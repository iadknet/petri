//! The pooled mesh-execution summary of a sample of genomes: node,
//! knockout, pass and execution sums and route-flag counts over each
//! genome's `mesh_execution` reading.
//!
//! The readings come from stored reports written by any earlier build, so
//! every field is optional: a reading lacking a count adds nothing to it, a
//! sum is absent until some reading carries its count, and the
//! within-snapshot flags (T19.F06) are counted only over readings that carry
//! them.

/// One genome's `mesh_execution` reading, each field as the reading carries
/// it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MeshRowReading {
    pub total_node_count: Option<u64>,
    pub reachable_node_count: Option<u64>,
    pub executed_node_count: Option<u64>,
    pub knockout_count: Option<u64>,
    pub pass_cap_hits: Option<u64>,
    pub passes: Option<u64>,
    pub executions_per_genome: Option<u64>,
    /// `false` where the reading carries no `true`.
    pub route_varies_with_input: bool,
    /// `false` where the reading carries no `true`.
    pub route_destination_varies: bool,
    pub route_varies_within_snapshot: Option<bool>,
    pub route_destination_varies_within_snapshot: Option<bool>,
}

/// The summary over every reading added, each sum `None` until a reading
/// carries its field.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MeshSummary {
    pub genomes: u64,
    pub total: Option<u64>,
    pub reachable: Option<u64>,
    pub executed: Option<u64>,
    pub knockout: Option<u64>,
    pub cap_hits: Option<u64>,
    pub passes: Option<u64>,
    pub execs: Option<u64>,
    pub route_varies: u64,
    pub route_destination_varies: u64,
    pub route_varies_within_snapshot: Option<u64>,
    pub route_destination_varies_within_snapshot: Option<u64>,
}

fn add(sum: &mut Option<u64>, value: Option<u64>) {
    if let Some(value) = value {
        *sum.get_or_insert(0) += value;
    }
}

impl MeshSummary {
    /// The summary of `rows`; `None` when there is none.
    pub fn of(rows: impl IntoIterator<Item = MeshRowReading>) -> Option<Self> {
        let mut summary = Self::default();
        for row in rows {
            summary.add(&row);
        }
        (summary.genomes > 0).then_some(summary)
    }

    pub fn add(&mut self, row: &MeshRowReading) {
        self.genomes += 1;
        add(&mut self.total, row.total_node_count);
        add(&mut self.reachable, row.reachable_node_count);
        add(&mut self.executed, row.executed_node_count);
        add(&mut self.knockout, row.knockout_count);
        add(&mut self.cap_hits, row.pass_cap_hits);
        add(&mut self.passes, row.passes);
        add(&mut self.execs, row.executions_per_genome);
        self.route_varies += u64::from(row.route_varies_with_input);
        self.route_destination_varies += u64::from(row.route_destination_varies);
        add(
            &mut self.route_varies_within_snapshot,
            row.route_varies_within_snapshot.map(u64::from),
        );
        add(
            &mut self.route_destination_varies_within_snapshot,
            row.route_destination_varies_within_snapshot.map(u64::from),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn no_reading_gives_no_summary() {
        assert_eq!(MeshSummary::of([]), None);
    }

    proptest! {
        /// Each sum is present exactly when some reading carries its field,
        /// and is then the sum over those readings; the route counts cover
        /// every reading.
        #[test]
        fn a_sum_covers_exactly_the_readings_carrying_its_field(
            rows in prop::collection::vec(
                (prop::option::of(0u64..100), any::<bool>(), prop::option::of(any::<bool>())),
                1..12,
            ),
        ) {
            let readings: Vec<MeshRowReading> = rows
                .iter()
                .map(|&(total, varies, within)| MeshRowReading {
                    total_node_count: total,
                    route_varies_with_input: varies,
                    route_varies_within_snapshot: within,
                    ..MeshRowReading::default()
                })
                .collect();
            let summary = MeshSummary::of(readings.iter().copied()).expect("a reading");
            prop_assert_eq!(summary.genomes, rows.len() as u64);
            let totals: Vec<u64> = rows.iter().filter_map(|row| row.0).collect();
            prop_assert_eq!(summary.total, (!totals.is_empty()).then(|| totals.iter().sum()));
            prop_assert_eq!(summary.reachable, None);
            prop_assert_eq!(summary.route_varies, rows.iter().filter(|row| row.1).count() as u64);
            let within: Vec<bool> = rows.iter().filter_map(|row| row.2).collect();
            prop_assert_eq!(
                summary.route_varies_within_snapshot,
                (!within.is_empty()).then(|| within.iter().filter(|&&v| v).count() as u64)
            );
        }
    }
}
