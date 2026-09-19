pub mod correlation;
pub mod descriptive;
pub mod distribution;
pub mod outlier;
pub mod percentile;
pub mod trend;

pub use correlation::{
    CorrelationStats,
    analyze_correlation,
};

pub use descriptive::{
    DescriptiveStats,
    analyze_descriptive,
};

pub use distribution::{
    DistributionStats,
    HistogramBin,
    analyze_distribution,
    analyze_distribution_with_bins,
};

pub use outlier::{
    OutlierStats,
    analyze_outliers,
    analyze_outliers_with_multiplier,
    is_outlier,
};

pub use percentile::{
    PercentileStats,
    analyze_percentiles,
    percentile,
};

pub use trend::{
    TrendDirection,
    TrendStats,
    analyze_index_trend,
    analyze_trend,
};