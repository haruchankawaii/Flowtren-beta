pub fn format_percentage(
    rate: f64,
) -> String {
    format!(
        "{:.1}%",
        rate * 100.0
    )
}

pub fn format_decimal(
    value: f64,
) -> String {
    format!(
        "{value:.2}"
    )
}

pub fn human_join_columns(
    columns: &[String],
) -> String {
    match columns {
        [] => {
            String::new()
        }

        [only] => {
            only.clone()
        }

        [left, right] => {
            format!(
                "{left} and {right}"
            )
        }

        _ => {
            columns.join(", ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_percentage() {
        assert_eq!(
            format_percentage(
                0.125,
            ),
            "12.5%"
        );
    }

    #[test]
    fn formats_decimal() {
        assert_eq!(
            format_decimal(
                0.91234,
            ),
            "0.91"
        );
    }

    #[test]
    fn joins_two_columns() {
        let columns =
            vec![
                "sales".to_string(),
                "marketing".to_string(),
            ];

        assert_eq!(
            human_join_columns(
                &columns,
            ),
            "sales and marketing"
        );
    }
}