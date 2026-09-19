#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
pub enum QualitySeverity {
    Healthy,
    Low,
    Medium,
    High,
}

pub fn severity_from_rate(
    rate: f64,
) -> QualitySeverity {
    if rate <= 0.0 {
        QualitySeverity::Healthy
    } else if rate < 0.05 {
        QualitySeverity::Low
    } else if rate < 0.20 {
        QualitySeverity::Medium
    } else {
        QualitySeverity::High
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_healthy() {
        assert_eq!(
            severity_from_rate(0.0),
            QualitySeverity::Healthy
        );
    }

    #[test]
    fn classifies_low() {
        assert_eq!(
            severity_from_rate(0.01),
            QualitySeverity::Low
        );
    }

    #[test]
    fn classifies_medium() {
        assert_eq!(
            severity_from_rate(0.10),
            QualitySeverity::Medium
        );
    }

    #[test]
    fn classifies_high() {
        assert_eq!(
            severity_from_rate(0.25),
            QualitySeverity::High
        );
    }
}