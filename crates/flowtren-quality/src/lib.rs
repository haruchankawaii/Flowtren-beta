pub mod cardinality;
pub mod column_quality;
pub mod comparison;
pub mod consistency;
pub mod dataset_quality;
pub mod duplicates;
pub mod issue;
pub mod missing;
pub mod score;
pub mod semantic;
pub mod severity;

pub use cardinality::{
    CardinalityMetrics,
    analyze_cardinality,
};

pub use column_quality::{
    ColumnQuality,
    analyze_column_quality,
};

pub use comparison::{
    QualityComparison,
    compare_quality,
};

pub use consistency::{
    ConsistencyMetrics,
    analyze_consistency,
};

pub use dataset_quality::{
    DatasetQuality,
    analyze_dataset_quality,
    analyze_dataset_quality_with_semantics,
};

pub use duplicates::{
    DuplicateMetrics,
    analyze_duplicates,
};

pub use issue::{
    QualityIssue,
    QualityIssueKind,
};

pub use missing::{
    MissingMetrics,
    analyze_missing,
};

pub use semantic::{
    SemanticQuality,
    analyze_semantic_quality,
};

pub use severity::{
    QualitySeverity,
    severity_from_rate,
};