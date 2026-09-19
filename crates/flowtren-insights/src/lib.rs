pub mod engine;
pub mod explanation;
pub mod insight;
pub mod ranking;
pub mod rules;

pub use engine::{
    InsightEngine,
};

pub use insight::{
    Insight,
    InsightConfidence,
    InsightEvidence,
    InsightKind,
    InsightSeverity,
};

pub use ranking::{
    rank_insights,
    ranked_insights,
};

pub use rules::{
    concentration::{
        calculate_concentration,
        generate_concentration_insight,
    },
    correlation::generate_correlation_insight,
    decline::generate_decline_insight,
    dominance::generate_dominance_insight,
    duplicates::generate_duplicate_insight,
    growth::generate_growth_insight,
    missing::generate_missing_insight,
    outliers::generate_outlier_insight,
    trend::generate_trend_insight,
    volatility::{
        calculate_volatility,
        generate_volatility_insight,
    },
};