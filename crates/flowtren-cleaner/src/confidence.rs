#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceLevel {
    Safe,
    Recommend,
    Review,
}

pub fn classify_confidence(
    confidence: f32,
) -> ConfidenceLevel {
    if confidence >= 0.95 {
        ConfidenceLevel::Safe
    } else if confidence >= 0.75 {
        ConfidenceLevel::Recommend
    } else {
        ConfidenceLevel::Review
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_safe_confidence() {
        assert_eq!(
            classify_confidence(1.0),
            ConfidenceLevel::Safe
        );

        assert_eq!(
            classify_confidence(0.95),
            ConfidenceLevel::Safe
        );
    }

    #[test]
    fn classifies_recommended_confidence() {
        assert_eq!(
            classify_confidence(0.90),
            ConfidenceLevel::Recommend
        );

        assert_eq!(
            classify_confidence(0.75),
            ConfidenceLevel::Recommend
        );
    }

    #[test]
    fn classifies_review_confidence() {
        assert_eq!(
            classify_confidence(0.74),
            ConfidenceLevel::Review
        );

        assert_eq!(
            classify_confidence(0.20),
            ConfidenceLevel::Review
        );
    }
}