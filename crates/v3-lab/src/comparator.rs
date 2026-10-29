//! Built-in comparators: competence witnesses composed from the authored
//! controllers, never tuned on assay outcomes.

use v3_core::genome::CreatureGenome;

use crate::controllers::{append_controller, controller, Family, AUTHORED_REF, VOTE_NODE};
use crate::scene::Assay;

/// The Ring reference's index on the vote node, after the Vector
/// controller's `AreaFoodSummary` at [`AUTHORED_REF`].
pub const RING_REF: u16 = AUTHORED_REF + 1;

/// The area-food controller (F01's comparator).
#[must_use]
pub fn area_food(founder: &CreatureGenome) -> CreatureGenome {
    controller(founder, Family::Vector)
}

/// `barrier-comparator`: the area-food controller plus the Ring controller
/// — `NeighborBarrierRing` at [`RING_REF`] and `barrier[d] → Move(d)` at
/// `RING_INHIBITION` for the four cardinal moves.
///
/// # Panics
///
/// If `founder`'s vote node is not the founder's seven-reference Graph node.
#[must_use]
pub fn barrier_comparator(founder: &CreatureGenome) -> CreatureGenome {
    let mut genome = area_food(founder);
    assert_eq!(
        genome.nodes[VOTE_NODE].input_refs().len(),
        usize::from(RING_REF),
        "the area-food controller's vote node holds eight references"
    );
    append_controller(&mut genome, Family::Ring);
    genome
}

/// The assay's built-in comparator.
#[must_use]
pub fn built_in(assay: Assay, founder: &CreatureGenome) -> CreatureGenome {
    match assay {
        Assay::FoodSeeking => area_food(founder),
        Assay::BarrierNavigation => barrier_comparator(founder),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arena::arena_config;
    use crate::controllers::{RING_INHIBITION, RING_MOVES};
    use crate::eval::{evaluate_genome, Setup};
    use crate::scene::{Geometry, SceneSpec};
    use rand::SeedableRng;
    use v3_core::contracts::votes::{DirectedVoteKind, VoteSink};
    use v3_core::contracts::InputReference;
    use v3_core::creature::founder::founder_genome_with_age_gate;
    use v3_core::genome::cgp::{GraphEdge, GraphSource, OutputSinkKind};
    use v3_core::genome::BackendDef;

    /// The reference the comparator reads at `index` on the vote node.
    fn vote_reference(genome: &CreatureGenome, index: usize) -> Option<&InputReference> {
        genome.nodes.get(VOTE_NODE)?.input_refs().get(index)
    }

    fn founder() -> CreatureGenome {
        let config = arena_config(64);
        founder_genome_with_age_gate(
            config.population.founder_profile,
            &config.energy.lifecycle,
            v3_core::creature::founder::default_founder_body(),
        )
    }

    #[test]
    fn the_barrier_comparator_appends_the_ring_to_the_area_food_controller() {
        let founder = founder();
        let genome = barrier_comparator(&founder);
        let refs = genome.nodes[VOTE_NODE].input_refs();
        assert_eq!(refs.len(), 9);
        assert_eq!(refs[..7], founder.nodes[VOTE_NODE].input_refs()[..]);
        assert_eq!(
            vote_reference(&genome, 7),
            Some(&Family::Vector.reference())
        );
        assert_eq!(vote_reference(&genome, 8), Some(&Family::Ring.reference()));
        let BackendDef::Graph(graph) = genome.nodes[VOTE_NODE].backend_def() else {
            panic!("graph vote node");
        };
        let area = area_food(&founder);
        let BackendDef::Graph(vector) = area.nodes[VOTE_NODE].backend_def() else {
            panic!("graph vote node");
        };
        assert_eq!(graph.compute_nodes(), vector.compute_nodes());
        for direction in 0..8u8 {
            let sink =
                OutputSinkKind::ActionVote(VoteSink::Directed(DirectedVoteKind::Move, direction));
            let edges = &graph.sink(sink).inputs;
            let before = &vector.sink(sink).inputs;
            let ring: Vec<&GraphEdge> = edges
                .iter()
                .filter(|e| matches!(e.source(), GraphSource::InputLeaf { ref_idx: 8, .. }))
                .collect();
            if RING_MOVES.contains(&direction) {
                assert_eq!(edges.len(), before.len() + 1);
                assert_eq!(
                    ring,
                    [&GraphEdge::new(
                        GraphSource::InputLeaf {
                            ref_idx: 8,
                            sub_idx: u16::from(direction),
                        },
                        RING_INHIBITION
                    )]
                );
            } else {
                assert_eq!(edges, before);
            }
        }
        assert_eq!(built_in(Assay::FoodSeeking, &founder), area_food(&founder));
        assert_eq!(built_in(Assay::BarrierNavigation, &founder), genome);
    }

    #[test]
    fn the_barrier_comparator_evaluates_on_a_barrier_scene() {
        let founder = founder();
        let spec = SceneSpec {
            geometry: Geometry::Ring { scale: 1 },
            size: 64,
            vision_radius: 5,
            assay: Assay::BarrierNavigation,
        };
        let scene = spec
            .draw(&mut rand::rngs::SmallRng::seed_from_u64(4))
            .unwrap();
        // Effort-cost defaults, 2026-10-10: start 200 and lifetime 50 so the comparator
        // lives the horizon (at 100 and 100 it died at tick 53).
        let setup = Setup::new(arena_config(64), 200.0, 50)
            .with_scoring(Assay::BarrierNavigation.scoring());
        let (score, _) = evaluate_genome(&setup, &barrier_comparator(&founder), &scene);
        assert_eq!(score.ticks, 50);
        assert!(score.moves_attempted > 0);
    }
}
