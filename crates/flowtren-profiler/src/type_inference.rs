use chrono::NaiveDate;
use flowtren_core::semantic_type::SemanticType;
use polars::prelude::*;

#[derive(Debug, Clone)]
pub struct SemanticInference {
    pub semantic_type: SemanticType,
    pub confidence: f32,
}

impl SemanticInference {
    fn new(
        semantic_type: SemanticType,
        confidence: f32,
    ) -> Self {
        Self {
            semantic_type,
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

pub fn infer_semantic_type(
    series: &Series,
) -> SemanticInference {
    match series.dtype() {
        DataType::Boolean => SemanticInference::new(
            SemanticType::Boolean,
            1.0,
        ),

        DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64 => SemanticInference::new(
            SemanticType::Integer,
            1.0,
        ),

        DataType::Float32 | DataType::Float64 => {
            SemanticInference::new(
                SemanticType::Decimal,
                1.0,
            )
        }

        DataType::Date => SemanticInference::new(
            SemanticType::Date,
            1.0,
        ),

        DataType::Datetime(_, _) => {
            SemanticInference::new(
                SemanticType::DateTime,
                1.0,
            )
        }

        DataType::String => infer_string_type(series),

        _ => SemanticInference::new(
            SemanticType::Unknown,
            0.0,
        ),
    }
}

fn infer_string_type(
    series: &Series,
) -> SemanticInference {
    let Ok(strings) = series.str() else {
        return SemanticInference::new(
            SemanticType::Unknown,
            0.0,
        );
    };

    let values: Vec<&str> = (0..strings.len())
        .filter_map(|index| strings.get(index))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect();

    if values.is_empty() {
        return SemanticInference::new(
            SemanticType::Unknown,
            0.0,
        );
    }

    // -------------------------------------------------
    // Specific semantic types
    //
    // Urutan ini penting. Tipe yang lebih spesifik
    // harus diperiksa sebelum Category / Text.
    // -------------------------------------------------

    let date_ratio = ratio_matching(
        &values,
        looks_like_date,
    );

    if date_ratio >= 0.80 {
        return SemanticInference::new(
            SemanticType::Date,
            date_ratio as f32,
        );
    }

    let email_ratio = ratio_matching(
        &values,
        looks_like_email,
    );

    if email_ratio >= 0.80 {
        return SemanticInference::new(
            SemanticType::Email,
            email_ratio as f32,
        );
    }

    let phone_ratio = ratio_matching(
        &values,
        looks_like_phone,
    );

    if phone_ratio >= 0.80 {
        return SemanticInference::new(
            SemanticType::Phone,
            phone_ratio as f32,
        );
    }

    let currency_ratio = ratio_matching(
        &values,
        looks_like_currency,
    );

    if currency_ratio >= 0.80 {
        return SemanticInference::new(
            SemanticType::Currency,
            currency_ratio as f32,
        );
    }

    let percentage_ratio = ratio_matching(
        &values,
        looks_like_percentage,
    );

    if percentage_ratio >= 0.80 {
        return SemanticInference::new(
            SemanticType::Percentage,
            percentage_ratio as f32,
        );
    }

    let identifier_ratio = ratio_matching(
        &values,
        looks_like_identifier,
    );

    if identifier_ratio >= 0.80 {
        return SemanticInference::new(
            SemanticType::Identifier,
            identifier_ratio as f32,
        );
    }

    // -------------------------------------------------
    // Category / Text
    // -------------------------------------------------

    let unique_count = series
        .n_unique()
        .unwrap_or(values.len());

    let unique_ratio =
        unique_count as f64 / values.len() as f64;

    if unique_ratio <= 0.20 {
        // Semakin sedikit nilai unik dibanding jumlah row,
        // semakin yakin kolom tersebut adalah Category.
        //
        // Contoh:
        // 2 unique / 10 rows
        // unique_ratio = 0.20
        // confidence = 0.80

        let confidence =
            (1.0 - unique_ratio) as f32;

        return SemanticInference::new(
            SemanticType::Category,
            confidence,
        );
    }

    // Untuk Text, semakin tinggi unique ratio,
    // semakin besar indikasi bahwa nilainya bebas
    // dan bukan kategori.
    let text_confidence =
        unique_ratio.clamp(0.0, 1.0) as f32;

    SemanticInference::new(
        SemanticType::Text,
        text_confidence,
    )
}

fn ratio_matching(
    values: &[&str],
    matcher: fn(&str) -> bool,
) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let matched = values
        .iter()
        .copied()
        .filter(|value| matcher(value))
        .count();

    matched as f64 / values.len() as f64
}

fn looks_like_date(value: &str) -> bool {
    let value = value.trim();

    const DATE_FORMATS: &[&str] = &[
        "%Y-%m-%d",
        "%Y/%m/%d",
        "%d-%m-%Y",
        "%d/%m/%Y",
        "%m-%d-%Y",
        "%m/%d/%Y",
    ];

    DATE_FORMATS.iter().any(|format| {
        NaiveDate::parse_from_str(
            value,
            format,
        )
        .is_ok()
    })
}

fn looks_like_email(value: &str) -> bool {
    let value = value.trim();

    let Some((local, domain)) =
        value.split_once('@')
    else {
        return false;
    };

    if local.is_empty() || domain.is_empty() {
        return false;
    }

    if value.contains(' ') {
        return false;
    }

    if !domain.contains('.') {
        return false;
    }

    if domain.starts_with('.')
        || domain.ends_with('.')
    {
        return false;
    }

    true
}

fn looks_like_phone(value: &str) -> bool {
    let value = value.trim();

    if value.is_empty() {
        return false;
    }

    let normalized: String = value
        .chars()
        .filter(|c| {
            c.is_ascii_digit() || *c == '+'
        })
        .collect();

    let digit_count = normalized
        .chars()
        .filter(|c| c.is_ascii_digit())
        .count();

    if !(8..=15).contains(&digit_count) {
        return false;
    }

    normalized.starts_with("08")
        || normalized.starts_with("62")
        || normalized.starts_with("+62")
        || normalized.starts_with('0')
        || normalized.starts_with('+')
}

fn looks_like_currency(value: &str) -> bool {
    let value = value.trim();

    let number_part =
        if let Some(rest) =
            value.strip_prefix("Rp")
        {
            rest
        } else if let Some(rest) =
            value.strip_prefix("IDR")
        {
            rest
        } else if let Some(rest) =
            value.strip_prefix('$')
        {
            rest
        } else if let Some(rest) =
            value.strip_prefix('€')
        {
            rest
        } else if let Some(rest) =
            value.strip_prefix('£')
        {
            rest
        } else {
            return false;
        };

    parse_localized_number(
        number_part.trim(),
    )
    .is_some()
}

fn looks_like_percentage(
    value: &str,
) -> bool {
    let value = value.trim();

    if !value.ends_with('%') {
        return false;
    }

    let number = value
        .trim_end_matches('%')
        .trim();

    parse_localized_number(number).is_some()
}

fn looks_like_identifier(
    value: &str,
) -> bool {
    let value = value.trim();

    if value.len() < 3 {
        return false;
    }

    let has_alpha = value
        .chars()
        .any(|c| c.is_ascii_alphabetic());

    let has_digit = value
        .chars()
        .any(|c| c.is_ascii_digit());

    let valid_chars = value
        .chars()
        .all(|c| {
            c.is_ascii_alphanumeric()
                || c == '-'
                || c == '_'
        });

    has_alpha && has_digit && valid_chars
}

fn parse_localized_number(
    value: &str,
) -> Option<f64> {
    let value = value.trim();

    if value.is_empty() {
        return None;
    }

    // Normal number:
    // 1500000
    // 1500.50
    if let Ok(number) =
        value.parse::<f64>()
    {
        return Some(number);
    }

    let has_dot = value.contains('.');
    let has_comma = value.contains(',');

    // Examples:
    //
    // Indonesian:
    // 1.500.000,50
    //
    // US:
    // 1,500,000.50
    if has_dot && has_comma {
        let last_dot = value.rfind('.')?;
        let last_comma = value.rfind(',')?;

        let normalized =
            if last_comma > last_dot {
                value
                    .replace('.', "")
                    .replace(',', ".")
            } else {
                value.replace(',', "")
            };

        return normalized
            .parse::<f64>()
            .ok();
    }

    // Only comma.
    if has_comma {
        let comma_count =
            value.matches(',').count();

        let normalized =
            if comma_count > 1 {
                // 1,500,000
                value.replace(',', "")
            } else {
                let parts: Vec<&str> =
                    value.split(',').collect();

                if parts.len() == 2
                    && parts[1].len() <= 2
                {
                    // 1500,50
                    value.replace(',', ".")
                } else {
                    // 1,500
                    value.replace(',', "")
                }
            };

        return normalized
            .parse::<f64>()
            .ok();
    }

    // Only dot.
    if has_dot {
        let dot_count =
            value.matches('.').count();

        if dot_count > 1 {
            // 1.500.000
            return value
                .replace('.', "")
                .parse::<f64>()
                .ok();
        }
    }

    None
}