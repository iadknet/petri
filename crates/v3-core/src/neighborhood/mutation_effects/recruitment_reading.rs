//! Fixed engineering denominators, independent offspring of one native parent.
use super::*;
use crate::config::{FounderProfile, NeutralInputRecruitment as Arm, SimulationConfig};
use crate::creature::founder::founder_genome;
use crate::creature::genome::cgp::{GraphSource, OutputSinkKind};
use crate::creature::genome::BackendDef;
use crate::neighborhood::input_use::catalog::{addressed, shared_memory, Addressed};
use std::time::Instant;

#[test]
#[ignore = "bounded T20.F04 release engineering reading; 6,144 births, not a discovery trial"]
fn neutral_recruitment_engineering_panel() {
    let started = Instant::now();
    let mut config = SimulationConfig::default();
    config.world.food.types = vec![config.world.food.types[0].clone(); 2];
    let parent = founder_genome(FounderProfile::V3Alpha1);
    let battery = Battery::generate(2);
    let reachable = mesh_reachable_nodes(&parent);
    let executed =
        battery.executed_indices(&parent, &config.runtime, config.shared_memory.decay_rate);
    let mut rows = Vec::new();
    for arm in [Arm::Off, Arm::SingleChannel, Arm::WholeFamily] {
        config.mutation.neutral_input_recruitment = arm;
        let mut requested = 0u64;
        let mut attempts = 0u64;
        let mut applicable = 0u64;
        let mut applied = 0u64;
        let mut skipped = 0u64;
        let mut exposed_births = 0u64;
        let mut declarations = 0usize;
        let mut edges = 0usize;
        let mut candidate_growth = 0u32;
        let mut total_growth = 0i64;
        let mut targets = BTreeMap::<String, u64>::new();
        let mut coverage = BTreeMap::<String, BTreeSet<u16>>::new();
        let mut sinks = BTreeMap::<String, u64>::new();
        let mut exposure = BTreeMap::<String, u64>::new();
        for i in 0..2048 {
            let mut child = parent.clone();
            let summary = MutationEngine::apply_mutations_with_food_type_count(
                &mut child,
                &config.mutation,
                &reachable,
                ParentExecuted::Indices(&executed),
                &mut SmallRng::seed_from_u64(29_004_000 + i),
                2,
            );
            requested += u64::from(summary.attempted_events);
            total_growth += i64::from(child.genome_size()) - i64::from(parent.genome_size());
            let funnel = summary
                .operator_funnel_by_operator
                .get(&MutationOperator::GraphRecruitNeutralInput)
                .copied()
                .unwrap_or_default();
            attempts += funnel.attempted;
            applicable += funnel.applicable;
            applied += funnel.applied;
            skipped += funnel.skipped;
            exposed_births += u64::from(!summary.recruitment_deltas.is_empty());
            assert_eq!(funnel.applied as usize, summary.recruitment_deltas.len());
            for (before, after) in summary.recruitment_deltas {
                candidate_growth += after.genome_size() - before.genome_size();
                for (old, new) in before.nodes.iter().zip(&after.nodes) {
                    if old == new {
                        continue;
                    }
                    *targets.entry(format!("{:?}", new.node_id)).or_default() += 1;
                    declarations += new.input_refs.len() - old.input_refs.len();
                    let (BackendDef::Graph(old_graph), BackendDef::Graph(new_graph)) =
                        (&old.backend_def, &new.backend_def)
                    else {
                        panic!("recruitment changed backend");
                    };
                    for (old_sink, new_sink) in
                        old_graph.output_sinks.iter().zip(&new_graph.output_sinks)
                    {
                        if old_sink.inputs.len() == new_sink.inputs.len() {
                            continue;
                        }
                        let OutputSinkKind::ActionVote(vote) = new_sink.kind else {
                            panic!("non-vote target")
                        };
                        *sinks.entry(format!("{vote:?}")).or_default() += 1;
                        for edge in &new_sink.inputs[old_sink.inputs.len()..] {
                            assert_eq!(edge.weight, 0.0);
                            edges += 1;
                            let channel = match edge.source {
                                GraphSource::InputLeaf { ref_idx, sub_idx } => {
                                    let Addressed::Channel(channel) =
                                        addressed(&new.input_refs[usize::from(ref_idx)], sub_idx)
                                    else {
                                        panic!("noncanonical input")
                                    };
                                    channel
                                }
                                GraphSource::SharedMemory { slot, previous } => {
                                    shared_memory(usize::from(slot), previous)
                                }
                                GraphSource::ComputeNode(_) => panic!("recruitment source"),
                            };
                            coverage
                                .entry(channel.family.label())
                                .or_default()
                                .insert(channel.channel);
                            *exposure
                                .entry(format!(
                                    "{}:{} -> {vote:?}",
                                    channel.family.label(),
                                    channel.channel
                                ))
                                .or_default() += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(attempts, applied + skipped);
        rows.push(serde_json::json!({"arm":format!("{arm:?}"), "births":2048, "requested_events":requested,
            "candidate_attempts":attempts, "candidate_applicable":applicable,"candidate_applied":applied,
            "candidate_skipped":skipped,"births_with_exposure":exposed_births,"births_without_exposure":2048-exposed_births,
            "declarations_added":declarations,"edges_added":edges,"candidate_size_growth":candidate_growth,
            "all_operator_net_size_growth":total_growth,"targets":targets,"coverage":coverage,"sink_events":sinks,"exposure":exposure}));
    }
    let evidence = serde_json::to_string_pretty(&serde_json::json!({"parent_units":parent.genome_size(),"reachable":reachable,"executed":executed,"food_types":2,"seed_start":29_004_000,"seed_end_exclusive":29_006_048,"rows":rows,"elapsed_seconds":started.elapsed().as_secs_f64()})).unwrap();
    assert!(evidence.len() <= 10 * 1024 * 1024);
    println!("{evidence}");
    assert!(
        started.elapsed().as_secs_f64() < 60.0,
        "engineering reading exceeded release cap"
    );
}
