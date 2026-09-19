use crate::suggestion::CleaningKind;

#[derive(Debug, Clone)]
pub struct CleaningAuditEntry {
    pub column: String,
    pub kind: CleaningKind,
    pub affected_rows: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CleaningAudit {
    pub entries: Vec<CleaningAuditEntry>,
}

impl CleaningAudit {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn record(
        &mut self,
        column: impl Into<String>,
        kind: CleaningKind,
        affected_rows: usize,
    ) {
        if affected_rows == 0 {
            return;
        }

        self.entries.push(
            CleaningAuditEntry {
                column: column.into(),
                kind,
                affected_rows,
            },
        );
    }

    pub fn total_changes(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| {
                entry.affected_rows
            })
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_audit_entries() {
        let mut audit =
            CleaningAudit::new();

        audit.record(
            "region",
            CleaningKind::TrimWhitespace,
            2,
        );

        audit.record(
            "email",
            CleaningKind::EmptyStringToNull,
            1,
        );

        assert_eq!(
            audit.entries.len(),
            2
        );

        assert_eq!(
            audit.total_changes(),
            3
        );

        assert!(!audit.is_empty());
    }

    #[test]
    fn ignores_zero_change_entries() {
        let mut audit =
            CleaningAudit::new();

        audit.record(
            "region",
            CleaningKind::TrimWhitespace,
            0,
        );

        assert!(
            audit.is_empty()
        );
    }
}