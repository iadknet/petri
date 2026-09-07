//! Per-operator treatment (T11.F01 Battery, "Per-operator treatment"): apply
//! each of the four mutation domains' `ALL` operators once to a fresh copy of
//! the subject, at the operator family's production reachable bias, and
//! classify the applied result against the subject's own base signature.

use rand::rngs::SmallRng;
use rand::SeedableRng;
use rayon::prelude::*;

use crate::config::{MutationConfig, ReachableBiasConfig};
use crate::creature::genome::analysis::mesh_reachable_nodes;
use crate::creature::genome::CreatureGenome;
use crate::mutation::graph::{GraphMutator, GraphOperator};
use crate::mutation::input_ref::{InputRefMutator, InputRefOperator};
use crate::mutation::reachability::TargetSets;
use crate::mutation::topology::{TopologyMutator, TopologyOperator};
use crate::mutation::types::{MutationSkipReason, TargetReachability};
use crate::mutation::vm::{VmMutator, VmOperator};
use crate::neighborhood::battery::{Battery, Signature};
use crate::neighborhood::classify::{classify, Tally};
use crate::neighborhood::EvalContext;

/// Seed base for VM-domain operator trials.
pub const VM_SEED_BASE: u64 = 1_000;
/// Seed base for graph-domain operator trials.
pub const GRAPH_SEED_BASE: u64 = 2_000;
/// Seed base for topology-domain operator trials.
pub const TOPOLOGY_SEED_BASE: u64 = 3_000;
/// Seed base for input-reference-domain operator trials.
pub const INPUT_REF_SEED_BASE: u64 = 4_000;

/// One operator's family, name (its `Debug` label, matching the audit's
/// report rows), and tally over the predeclared trial count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorRow {
    pub family: &'static str,
    pub operator: String,
    pub tally: Tally,
}

/// One operator across the four mutation domains, unified behind a single
/// `apply` so every domain's trials can share one flat parallel job instead
/// of one rayon call per operator.
#[derive(Debug, Clone, Copy)]
enum OperatorKind {
    Vm(VmOperator),
    Graph(GraphOperator),
    Topology(TopologyOperator),
    InputRef(InputRefOperator),
}

impl OperatorKind {
    fn all() -> Vec<Self> {
        VmOperator::ALL
            .into_iter()
            .map(Self::Vm)
            .chain(GraphOperator::ALL.into_iter().map(Self::Graph))
            .chain(TopologyOperator::ALL.into_iter().map(Self::Topology))
            .chain(InputRefOperator::ALL.into_iter().map(Self::InputRef))
            .collect()
    }

    const fn family(self) -> &'static str {
        match self {
            Self::Vm(_) => "vm",
            Self::Graph(_) => "graph",
            Self::Topology(_) => "topology",
            Self::InputRef(_) => "input_ref",
        }
    }

    fn name(self) -> String {
        match self {
            Self::Vm(op) => format!("{op:?}"),
            Self::Graph(op) => format!("{op:?}"),
            Self::Topology(op) => format!("{op:?}"),
            Self::InputRef(op) => format!("{op:?}"),
        }
    }

    const fn seed_base(self) -> u64 {
        match self {
            Self::Vm(_) => VM_SEED_BASE,
            Self::Graph(_) => GRAPH_SEED_BASE,
            Self::Topology(_) => TOPOLOGY_SEED_BASE,
            Self::InputRef(_) => INPUT_REF_SEED_BASE,
        }
    }

    const fn reachable_bias(self, config: &ReachableBiasConfig) -> f64 {
        match self {
            Self::Vm(_) => config.vm,
            Self::Graph(_) => config.graph,
            Self::Topology(_) => config.topology,
            Self::InputRef(_) => config.input_ref,
        }
    }

    fn apply(
        self,
        genome: &mut CreatureGenome,
        targets: &TargetSets<'_>,
        mutation_config: &MutationConfig,
        food_type_count: usize,
        rng: &mut SmallRng,
    ) -> Result<TargetReachability, MutationSkipReason> {
        let mut targets = targets.selector(
            self.reachable_bias(&mutation_config.reachable_bias),
            mutation_config.executed_bias,
        );
        match self {
            Self::Vm(op) => VmMutator::apply(genome, op, &mut targets, rng, mutation_config),
            Self::Graph(op) => GraphMutator::apply(genome, op, &mut targets, rng, mutation_config),
            Self::Topology(op) => TopologyMutator::apply_with_food_type_count(
                genome,
                op,
                &mut targets,
                rng,
                mutation_config,
                food_type_count,
            ),
            Self::InputRef(op) => InputRefMutator::apply_with_food_type_count(
                genome,
                op,
                &mut targets,
                rng,
                mutation_config,
                food_type_count,
            ),
        }
    }
}

/// The stable catalog of `(family, operator_name)` pairs every operator-row
/// list carries, in the four families' `ALL` order — independent of any
/// genome, so a caller pooling several genomes' rows (or reporting on an
/// empty evolved sample) always has the full row set to fill in.
#[must_use]
pub fn operator_catalog() -> Vec<(&'static str, String)> {
    OperatorKind::all()
        .into_iter()
        .map(|op| (op.family(), op.name()))
        .collect()
}

/// Apply every operator in the four `ALL` lists to `subject`, `trials` times
/// each, seeded by `seed_offset + <family base> + trial_index`, and classify
/// each applied result against `base` (the subject's own unmutated
/// signature). Returns one row per operator, in the four families' `ALL`
/// order (VM, graph, topology, input reference).
///
/// Every trial across every operator runs as one flat rayon job (one join
/// barrier, good load balance) rather than one rayon call per operator; the
/// per-operator, per-trial seed is unaffected, so tallies are identical to
/// running each operator's trials as a separate sequential or parallel batch.
#[must_use]
pub fn per_operator_rows(
    subject: &CreatureGenome,
    base: &Signature,
    battery: &Battery,
    mutation_config: &MutationConfig,
    context: &EvalContext,
    trials: u32,
    seed_offset: u64,
) -> Vec<OperatorRow> {
    let reachable = mesh_reachable_nodes(subject);
    // Observation stand-in for a live parent's dispatch record (T11.F17).
    let executed =
        battery.executed_indices(subject, context.runtime, context.shared_memory_decay_rate);
    let targets = TargetSets::new(&reachable, &executed);
    let ops = OperatorKind::all();

    let jobs: Vec<(usize, u32)> = (0..ops.len())
        .flat_map(|op_idx| (0..trials).map(move |trial| (op_idx, trial)))
        .collect();

    let tallies: Vec<Tally> = jobs
        .into_par_iter()
        .fold(
            || vec![Tally::default(); ops.len()],
            |mut acc, (op_idx, trial)| {
                let op = ops[op_idx];
                let seed = seed_offset + op.seed_base() + u64::from(trial);
                let mut rng = SmallRng::seed_from_u64(seed);
                let mut genome = subject.clone();
                let outcome = op
                    .apply(
                        &mut genome,
                        &targets,
                        mutation_config,
                        context.food_type_count,
                        &mut rng,
                    )
                    .map(|_| genome);
                let trial_tally = match outcome {
                    Err(_) => Tally::default().skip(),
                    Ok(genome) => {
                        let signature = battery.signature(
                            &genome,
                            context.runtime,
                            context.shared_memory_decay_rate,
                        );
                        Tally::default().record(classify(base, &signature))
                    }
                };
                acc[op_idx] = acc[op_idx].merge(trial_tally);
                acc
            },
        )
        .reduce(
            || vec![Tally::default(); ops.len()],
            |mut a, b| {
                for (slot, other) in a.iter_mut().zip(b) {
                    *slot = slot.merge(other);
                }
                a
            },
        );

    ops.into_iter()
        .zip(tallies)
        .map(|(op, tally)| OperatorRow {
            family: op.family(),
            operator: op.name(),
            tally,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FounderProfile;
    use crate::creature::founder::founder_genome;

    #[test]
    fn catalog_covers_every_operator_in_the_four_all_lists_once() {
        let catalog = operator_catalog();
        assert_eq!(
            catalog.len(),
            VmOperator::ALL.len()
                + GraphOperator::ALL.len()
                + TopologyOperator::ALL.len()
                + InputRefOperator::ALL.len()
        );
        let distinct: std::collections::BTreeSet<_> = catalog.iter().cloned().collect();
        assert_eq!(
            distinct.len(),
            catalog.len(),
            "every (family, operator) pair is unique"
        );
    }

    #[test]
    fn catalog_order_is_stable_across_calls() {
        assert_eq!(operator_catalog(), operator_catalog());
    }

    #[test]
    fn per_operator_rows_is_deterministic_and_covers_the_full_catalog() {
        let config = crate::config::SimulationConfig::default();
        let subject = founder_genome(FounderProfile::V3Alpha1);
        let battery = Battery::generate(config.world.food.types.len());
        let context = EvalContext::from_config(&config);
        let base = battery.signature(&subject, context.runtime, context.shared_memory_decay_rate);

        let a = per_operator_rows(&subject, &base, &battery, &config.mutation, &context, 5, 0);
        let b = per_operator_rows(&subject, &base, &battery, &config.mutation, &context, 5, 0);

        assert_eq!(
            a, b,
            "the same trial count and seed offset always yield the same rows"
        );
        assert_eq!(a.len(), operator_catalog().len());
        for row in &a {
            assert_eq!(row.tally.trials, 5);
        }
    }

    /// `operator_catalog`'s family labels come from `OperatorKind::family`,
    /// in the four `ALL` lists' order (VM, graph, topology, input
    /// reference): the family boundaries fall exactly at each list's length.
    /// A stub `family` returning a fixed string would collapse every
    /// boundary to one label.
    #[test]
    fn operator_catalog_family_labels_match_the_four_all_list_boundaries() {
        let catalog = operator_catalog();
        let vm_len = VmOperator::ALL.len();
        let graph_len = GraphOperator::ALL.len();
        let topology_len = TopologyOperator::ALL.len();
        let input_ref_len = InputRefOperator::ALL.len();

        let mut idx = 0;
        for _ in 0..vm_len {
            assert_eq!(catalog[idx].0, "vm");
            idx += 1;
        }
        for _ in 0..graph_len {
            assert_eq!(catalog[idx].0, "graph");
            idx += 1;
        }
        for _ in 0..topology_len {
            assert_eq!(catalog[idx].0, "topology");
            idx += 1;
        }
        for _ in 0..input_ref_len {
            assert_eq!(catalog[idx].0, "input_ref");
            idx += 1;
        }
        assert_eq!(idx, catalog.len());
    }

    /// The fixed pieces every family's reconstruction in
    /// `per_operator_rows_seeds_trials_by_offset_plus_family_base_plus_trial`
    /// shares, bundled so [`ReconstructFixture::tally`] stays under clippy's
    /// argument-count lint.
    struct ReconstructFixture<'a> {
        subject: &'a CreatureGenome,
        base: &'a Signature,
        battery: &'a Battery,
        context: &'a EvalContext<'a>,
    }

    impl ReconstructFixture<'_> {
        /// Run `trials` seeded by `seed_offset + seed_base + trial_index`,
        /// folding each outcome (via `apply`, which mutates a fresh clone and
        /// reports whether the operator applied) into a [`Tally`] exactly as
        /// [`per_operator_rows`] does. Shared by every family's case below so
        /// the seed arithmetic is written once per test rather than once per
        /// family.
        fn tally(
            &self,
            trials: u32,
            seed_offset: u64,
            seed_base: u64,
            mut apply: impl FnMut(&mut CreatureGenome, &mut SmallRng) -> Result<(), MutationSkipReason>,
        ) -> Tally {
            let mut tally = Tally::default();
            for trial in 0..trials {
                let mut rng = SmallRng::seed_from_u64(seed_offset + seed_base + u64::from(trial));
                let mut genome = self.subject.clone();
                tally = match apply(&mut genome, &mut rng) {
                    Err(_) => tally.skip(),
                    Ok(()) => {
                        let signature = self.battery.signature(
                            &genome,
                            self.context.runtime,
                            self.context.shared_memory_decay_rate,
                        );
                        tally.record(classify(self.base, &signature))
                    }
                };
            }
            tally
        }
    }

    /// `per_operator_rows` seeds trial `t` of an operator in family `f` by
    /// `seed_offset + f.seed_base() + t`. This reconstructs every operator of
    /// every family independently, using the public seed-base constants and
    /// the same production mutator entry points, and checks the whole
    /// reconstructed row list against `per_operator_rows`'s own output. A
    /// wrong `seed_base` (a fixed `0` or `1` for every family) or a `+`
    /// replaced by `-`/`*` in the seed arithmetic draws a different mutation
    /// per trial; reconstructing the full catalog (not just one operator per
    /// family) rules out a near-deterministic operator coincidentally
    /// matching under the wrong seed.
    #[test]
    fn per_operator_rows_seeds_trials_by_offset_plus_family_base_plus_trial() {
        let config = crate::config::SimulationConfig::default();
        let subject = founder_genome(FounderProfile::V3Alpha1);
        let battery = Battery::generate(config.world.food.types.len());
        let context = EvalContext::from_config(&config);
        let base = battery.signature(&subject, context.runtime, context.shared_memory_decay_rate);
        let reachable = mesh_reachable_nodes(&subject);
        let executed =
            battery.executed_indices(&subject, context.runtime, context.shared_memory_decay_rate);
        let sets = TargetSets::new(&reachable, &executed);
        let executed_bias = config.mutation.executed_bias;
        let rb = &config.mutation.reachable_bias;
        let trials = 3u32;
        let seed_offset = 777u64;

        let rows = per_operator_rows(
            &subject,
            &base,
            &battery,
            &config.mutation,
            &context,
            trials,
            seed_offset,
        );

        let fixture = ReconstructFixture {
            subject: &subject,
            base: &base,
            battery: &battery,
            context: &context,
        };
        let mut expected: Vec<(&'static str, String, Tally)> = Vec::new();

        for op in VmOperator::ALL {
            let tally = fixture.tally(trials, seed_offset, VM_SEED_BASE, |genome, rng| {
                VmMutator::apply(
                    genome,
                    op,
                    &mut sets.selector(rb.vm, executed_bias),
                    rng,
                    &config.mutation,
                )
                .map(|_| ())
            });
            expected.push(("vm", format!("{op:?}"), tally));
        }

        for op in GraphOperator::ALL {
            let tally = fixture.tally(trials, seed_offset, GRAPH_SEED_BASE, |genome, rng| {
                GraphMutator::apply(
                    genome,
                    op,
                    &mut sets.selector(rb.graph, executed_bias),
                    rng,
                    &config.mutation,
                )
                .map(|_| ())
            });
            expected.push(("graph", format!("{op:?}"), tally));
        }

        for op in TopologyOperator::ALL {
            let tally = fixture.tally(trials, seed_offset, TOPOLOGY_SEED_BASE, |genome, rng| {
                TopologyMutator::apply_with_food_type_count(
                    genome,
                    op,
                    &mut sets.selector(rb.topology, executed_bias),
                    rng,
                    &config.mutation,
                    context.food_type_count,
                )
                .map(|_| ())
            });
            expected.push(("topology", format!("{op:?}"), tally));
        }

        for op in InputRefOperator::ALL {
            let tally = fixture.tally(trials, seed_offset, INPUT_REF_SEED_BASE, |genome, rng| {
                InputRefMutator::apply_with_food_type_count(
                    genome,
                    op,
                    &mut sets.selector(rb.input_ref, executed_bias),
                    rng,
                    &config.mutation,
                    context.food_type_count,
                )
                .map(|_| ())
            });
            expected.push(("input_ref", format!("{op:?}"), tally));
        }

        assert_eq!(rows.len(), expected.len());
        for (row, (family, operator, tally)) in rows.iter().zip(expected.iter()) {
            assert_eq!(row.family, *family);
            assert_eq!(&row.operator, operator);
            assert_eq!(row.tally, *tally, "family {family} operator {operator}");
        }
    }
}
