use super::{Arm, Lineage};
use crate::neighborhood::recruitment_paths::{estimate, Estimate};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Positive,
    Negative,
    Inconclusive,
}

#[derive(Debug, Clone, Serialize)]
pub struct Comparison {
    pub panel: u32,
    pub family: usize,
    pub arm: Arm,
    pub control: Arm,
    pub candidate: Estimate,
    pub baseline: Estimate,
    pub contrast: Option<[f64; 2]>,
    pub exposed: Estimate,
    pub complete: bool,
    pub verdict: Verdict,
}

pub fn compare(rows: &[Lineage], panel: u32, family: usize, arm: Arm, control: Arm) -> Comparison {
    let expected = if panel == 0 { 32 } else { 16 };
    let selected = |wanted| {
        rows.iter().filter(move |row| {
            row.identity.panel == panel
                && row.identity.family == family
                && row.identity.arm == wanted
                && row.complete
        })
    };
    let candidate_rows: Vec<_> = selected(arm).collect();
    let baseline_rows: Vec<_> = selected(control).collect();
    let candidate = estimate(
        candidate_rows
            .iter()
            .filter(|row| row.primary_graph_discovery)
            .count() as u32,
        candidate_rows.len() as u32,
    );
    let baseline = estimate(
        baseline_rows
            .iter()
            .filter(|row| row.primary_graph_discovery)
            .count() as u32,
        baseline_rows.len() as u32,
    );
    let exposed = estimate(
        candidate_rows
            .iter()
            .filter(|row| {
                if matches!(arm, Arm::C | Arm::M) {
                    row.focal_structured_events > 0
                } else {
                    row.focal_access_events > 0
                }
            })
            .count() as u32,
        candidate_rows.len() as u32,
    );
    comparison(
        panel, family, arm, control, candidate, baseline, exposed, expected,
    )
}

#[allow(clippy::too_many_arguments)]
fn comparison(
    panel: u32,
    family: usize,
    arm: Arm,
    control: Arm,
    candidate: Estimate,
    baseline: Estimate,
    exposed: Estimate,
    expected: u32,
) -> Comparison {
    let complete = candidate.denominator == expected && baseline.denominator == expected;
    let contrast = candidate
        .wilson_95
        .zip(baseline.wilson_95)
        .map(|(candidate, control)| [candidate[0] - control[1], candidate[1] - control[0]]);
    let verdict = match (
        complete,
        candidate.fraction.zip(baseline.fraction),
        contrast,
    ) {
        (true, Some((candidate, baseline)), Some(interval))
            if candidate >= 0.25 && candidate - baseline >= 0.10 && interval[0] > 0.0 =>
        {
            Verdict::Positive
        }
        (true, _, Some(interval))
            if interval[1] < 0.10 && exposed.fraction.is_some_and(|fraction| fraction >= 0.75) =>
        {
            Verdict::Negative
        }
        _ => Verdict::Inconclusive,
    };
    Comparison {
        panel,
        family,
        arm,
        control,
        candidate,
        baseline,
        contrast,
        exposed,
        complete,
        verdict,
    }
}

pub fn overall(panels: &[Comparison]) -> Verdict {
    if panels.len() == 2
        && panels
            .iter()
            .all(|panel| panel.verdict == Verdict::Positive)
    {
        Verdict::Positive
    } else if panels
        .iter()
        .any(|panel| panel.verdict == Verdict::Negative)
    {
        Verdict::Negative
    } else {
        Verdict::Inconclusive
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn missing_lineages_cannot_qualify(successes in 0..32u32) {
            let estimate = estimate(successes, 31.max(successes));
            let row = comparison(0, 0, Arm::S, Arm::B, estimate, super::estimate(0, 32), super::estimate(31, 31), 32);
            prop_assert_ne!(row.verdict, Verdict::Positive);
        }
        #[test]
        fn identical_rates_never_positive(successes in 0..=32u32) {
            let row = comparison(0, 0, Arm::S, Arm::B, estimate(successes, 32), estimate(successes, 32), estimate(32, 32), 32);
            prop_assert_ne!(row.verdict, Verdict::Positive);
        }
    }
}
