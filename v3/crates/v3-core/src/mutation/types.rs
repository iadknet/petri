use std::collections::HashMap;

/// Reason a mutation event was skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MutationSkipReason {
    ParseabilityViolation,
    NoApplicableTarget,
    BudgetExhausted,
}

/// Summary returned by `MutationEngine` for every offspring.
///
/// Accounting invariant: `attempted_events == applied_events + skipped_events`.
#[derive(Debug, Clone)]
pub struct MutationSummary {
    pub attempted_events: u32,
    pub applied_events: u32,
    pub skipped_events: u32,
    pub skip_reasons: HashMap<MutationSkipReason, u32>,
}

impl MutationSummary {
    /// Return a summary with all counts zero and an empty skip-reason map.
    pub fn zero() -> Self {
        Self {
            attempted_events: 0,
            applied_events: 0,
            skipped_events: 0,
            skip_reasons: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_summary_zero_has_zero_counts() {
        let s = MutationSummary::zero();
        assert_eq!(s.attempted_events, 0);
        assert_eq!(s.applied_events, 0);
        assert_eq!(s.skipped_events, 0);
        assert!(s.skip_reasons.is_empty());
    }

    #[test]
    fn zero_summary_accounting_invariant() {
        let s = MutationSummary::zero();
        assert_eq!(s.attempted_events, s.applied_events + s.skipped_events);
    }
}
