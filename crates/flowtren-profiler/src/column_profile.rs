use flowtren_core::semantic_type::SemanticType;
use polars::prelude::*;

use crate::type_inference::infer_semantic_type;

#[derive(Debug, Clone)]
pub struct BasicColumnProfile {
    pub name: String,
    pub dtype: String,

    pub semantic_type: SemanticType,
    pub semantic_confidence: f32,

    pub null_count: usize,
    pub unique_count: usize,
}

pub fn profile_column(
    series: &Series,
) -> PolarsResult<BasicColumnProfile> {
    let inference =
        infer_semantic_type(series);

    Ok(BasicColumnProfile {
        name: series.name().to_string(),
        dtype: series.dtype().to_string(),

        semantic_type:
            inference.semantic_type,

        semantic_confidence:
            inference.confidence,

        null_count:
            series.null_count(),

        unique_count:
            series.n_unique()?,
    })
}