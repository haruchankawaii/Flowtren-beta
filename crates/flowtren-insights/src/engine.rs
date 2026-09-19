use flowtren_quality::{
    ColumnQuality,
    DatasetQuality,
};

use flowtren_stats::{
    CorrelationStats,
    DescriptiveStats,
    OutlierStats,
    TrendStats,
};

use crate::{
    insight::Insight,
    ranking::rank_insights,
    rules::{
        concentration::generate_concentration_insight,
        correlation::generate_correlation_insight,
        decline::generate_decline_insight,
        dominance::generate_dominance_insight,
        duplicates::generate_duplicate_insight,
        growth::generate_growth_insight,
        missing::generate_missing_insight,
        outliers::generate_outlier_insight,
        trend::generate_trend_insight,
        volatility::generate_volatility_insight,
    },
};

#[derive(
    Debug,
    Clone,
    Default,
)]
pub struct InsightEngine {
    insights:
        Vec<Insight>,
}

impl InsightEngine {
    pub fn new() -> Self {
        Self {
            insights:
                Vec::new(),
        }
    }

    pub fn add_outlier_analysis(
        &mut self,
        column_name: &str,
        stats: &OutlierStats,
    ) {
        if let Some(insight) =
            generate_outlier_insight(
                column_name,
                stats,
            )
        {
            self.insights.push(
                insight,
            );
        }
    }

    pub fn add_correlation_analysis(
        &mut self,
        left_column: &str,
        right_column: &str,
        stats: &CorrelationStats,
    ) {
        if let Some(insight) =
            generate_correlation_insight(
                left_column,
                right_column,
                stats,
            )
        {
            self.insights.push(
                insight,
            );
        }
    }

    pub fn add_trend_analysis(
        &mut self,
        column_name: &str,
        stats: &TrendStats,
    ) {
        if let Some(insight) =
            generate_trend_insight(
                column_name,
                stats,
            )
        {
            self.insights.push(
                insight,
            );
        }
    }

    pub fn add_missing_analysis(
        &mut self,
        quality: &ColumnQuality,
    ) {
        if let Some(insight) =
            generate_missing_insight(
                quality,
            )
        {
            self.insights.push(
                insight,
            );
        }
    }

    pub fn add_duplicate_analysis(
        &mut self,
        quality: &DatasetQuality,
    ) {
        if let Some(insight) =
            generate_duplicate_insight(
                quality,
            )
        {
            self.insights.push(
                insight,
            );
        }
    }

    pub fn add_growth_analysis(
        &mut self,
        column_name: &str,
        start_value: f64,
        end_value: f64,
        observation_count: usize,
    ) {
        if let Some(insight) =
            generate_growth_insight(
                column_name,
                start_value,
                end_value,
                observation_count,
            )
        {
            self.insights.push(
                insight,
            );
        }
    }

    pub fn add_decline_analysis(
        &mut self,
        column_name: &str,
        start_value: f64,
        end_value: f64,
        observation_count: usize,
    ) {
        if let Some(insight) =
            generate_decline_insight(
                column_name,
                start_value,
                end_value,
                observation_count,
            )
        {
            self.insights.push(
                insight,
            );
        }
    }

    pub fn add_dominance_analysis(
        &mut self,
        column_name: &str,
        dominant_value: &str,
        dominant_count: usize,
        total_count: usize,
    ) {
        if let Some(insight) =
            generate_dominance_insight(
                column_name,
                dominant_value,
                dominant_count,
                total_count,
            )
        {
            self.insights.push(
                insight,
            );
        }
    }

    pub fn add_concentration_analysis(
        &mut self,
        column_name: &str,
        counts: &[usize],
    ) {
        if let Some(insight) =
            generate_concentration_insight(
                column_name,
                counts,
            )
        {
            self.insights.push(
                insight,
            );
        }
    }

    pub fn add_volatility_analysis(
        &mut self,
        column_name: &str,
        stats: &DescriptiveStats,
    ) {
        if let Some(insight) =
            generate_volatility_insight(
                column_name,
                stats,
            )
        {
            self.insights.push(
                insight,
            );
        }
    }

    pub fn add_insight(
        &mut self,
        insight: Insight,
    ) {
        self.insights.push(
            insight,
        );
    }

    pub fn len(&self) -> usize {
        self.insights.len()
    }

    pub fn is_empty(&self) -> bool {
        self.insights.is_empty()
    }

    pub fn clear(&mut self) {
        self.insights.clear();
    }

    pub fn finish(
        mut self,
    ) -> Vec<Insight> {
        rank_insights(
            &mut self.insights,
        );

        self.insights
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::insight::InsightKind;

    #[test]
    fn collects_new_insight_types() {
        let mut engine =
            InsightEngine::new();

        engine.add_dominance_analysis(
            "city",
            "Jakarta",
            80,
            100,
        );

        engine.add_concentration_analysis(
            "category",
            &[
                80,
                10,
                5,
                5,
            ],
        );

        engine.add_volatility_analysis(
            "revenue",
            &DescriptiveStats {
                row_count:
                    100,

                count:
                    100,

                null_count:
                    0,

                sum:
                    Some(10_000.0),

                mean:
                    Some(100.0),

                min:
                    Some(1.0),

                max:
                    Some(300.0),

                std_dev:
                    Some(80.0),
            },
        );

        let insights =
            engine.finish();

        assert_eq!(
            insights.len(),
            3
        );

        assert!(
            insights.iter().any(
                |insight| {
                    insight.kind
                        == InsightKind::Dominance
                }
            )
        );

        assert!(
            insights.iter().any(
                |insight| {
                    insight.kind
                        == InsightKind::Concentration
                }
            )
        );

        assert!(
            insights.iter().any(
                |insight| {
                    insight.kind
                        == InsightKind::Volatility
                }
            )
        );
    }
}