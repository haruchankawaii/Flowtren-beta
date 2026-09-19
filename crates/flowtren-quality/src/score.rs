pub fn score_from_rate(
    bad_rate: f64,
) -> f64 {
    let normalized =
        bad_rate.clamp(
            0.0,
            1.0,
        );

    clamp_score(
        100.0
            * (1.0 - normalized),
    )
}

pub fn score_from_missing_rate(
    missing_rate: f64,
) -> f64 {
    score_from_rate(
        missing_rate,
    )
}

pub fn score_from_duplicate_rate(
    duplicate_rate: f64,
) -> f64 {
    score_from_rate(
        duplicate_rate,
    )
}

pub fn score_from_inconsistency_rate(
    inconsistency_rate: f64,
) -> f64 {
    score_from_rate(
        inconsistency_rate,
    )
}

pub fn combine_scores(
    scores: &[f64],
) -> f64 {
    if scores.is_empty() {
        return 100.0;
    }

    let total:
        f64 =
        scores
            .iter()
            .sum();

    clamp_score(
        total
            / scores.len() as f64,
    )
}

pub fn weighted_score(
    scores: &[(f64, f64)],
) -> f64 {
    if scores.is_empty() {
        return 100.0;
    }

    let total_weight:
        f64 =
        scores
            .iter()
            .map(
                |(_, weight)| {
                    weight.max(0.0)
                },
            )
            .sum();

    if total_weight == 0.0 {
        return 100.0;
    }

    let weighted_sum:
        f64 =
        scores
            .iter()
            .map(
                |(score, weight)| {
                    clamp_score(
                        *score,
                    ) * weight.max(0.0)
                },
            )
            .sum();

    clamp_score(
        weighted_sum
            / total_weight,
    )
}

fn clamp_score(
    score: f64,
) -> f64 {
    score.clamp(
        0.0,
        100.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_problem_rate_scores_100() {
        assert_eq!(
            score_from_rate(0.0),
            100.0
        );
    }

    #[test]
    fn half_problem_rate_scores_50() {
        assert_eq!(
            score_from_rate(0.5),
            50.0
        );
    }

    #[test]
    fn combines_scores() {
        let score =
            combine_scores(
                &[
                    100.0,
                    80.0,
                    60.0,
                ],
            );

        assert_eq!(
            score,
            80.0
        );
    }

    #[test]
    fn weighted_scores_work() {
        let score =
            weighted_score(
                &[
                    (100.0, 0.75),
                    (0.0, 0.25),
                ],
            );

        assert_eq!(
            score,
            75.0
        );
    }

    #[test]
    fn empty_scores_are_perfect() {
        assert_eq!(
            combine_scores(&[]),
            100.0
        );

        assert_eq!(
            weighted_score(&[]),
            100.0
        );
    }
}