//! Bounded native inherited input discovery (T20.F09). Observation only.

mod evaluation;
mod experiment;
mod scenes;
pub mod verdict;
pub use experiment::{digest, lineage, seed, Arm, Frozen, Identity, Lineage, Proposal, Record};

pub use super::opportunity::controllers::Family;
pub use evaluation::{checkpoint, qualifies_score, Panel, Qualification, Reading};
pub use scenes::{learning_off, scene_config, Scene, SceneReading};

pub const VERSION: &str = "input-discovery-v1";
pub const FAMILIES: [Family; 3] = [Family::Scalar, Family::Vector, Family::Ring];

/// The frozen family start: canonical primary starts, existing area competence for ring.
pub fn family_start(
    founder: &crate::creature::genome::CreatureGenome,
    family: Family,
) -> crate::creature::genome::CreatureGenome {
    if family == Family::Ring {
        super::opportunity::controllers::controller(founder, Family::Vector, false).0
    } else {
        founder.clone()
    }
}

/// Labelled authored instrument control; never used as an inherited start.
pub fn instrument_control(
    founder: &crate::creature::genome::CreatureGenome,
    family: Family,
    inert: bool,
) -> crate::creature::genome::CreatureGenome {
    use crate::creature::genome::{
        cgp::OutputSinkKind,
        vote::{ActionParamField, VoteSink},
        BackendDef,
    };
    if family == Family::Ring {
        let config = scene_config();
        let canonical = crate::creature::founder::founder_genome_with_age_gate(
            config.population.founder_profile,
            &config.energy.lifecycle,
        );
        let overlay =
            super::opportunity::controllers::controller(&canonical, Family::Ring, inert).0;
        let mut control = founder.clone();
        let node = &mut control.nodes[super::opportunity::controllers::VOTE_NODE];
        let ref_idx =
            u16::try_from(node.input_refs.len()).expect("fixed controller reference index");
        node.input_refs.push(Family::Ring.reference());
        let BackendDef::Graph(graph) = &mut node.backend_def else {
            unreachable!()
        };
        let BackendDef::Graph(base) =
            &canonical.nodes[super::opportunity::controllers::VOTE_NODE].backend_def
        else {
            unreachable!()
        };
        let BackendDef::Graph(addition) =
            &overlay.nodes[super::opportunity::controllers::VOTE_NODE].backend_def
        else {
            unreachable!()
        };
        for ((sink, base), addition) in graph
            .output_sinks
            .iter_mut()
            .zip(&base.output_sinks)
            .zip(&addition.output_sinks)
        {
            for edge in addition.inputs.iter().skip(base.inputs.len()) {
                let mut edge = *edge;
                if let crate::creature::genome::cgp::GraphSource::InputLeaf {
                    ref_idx: index, ..
                } = &mut edge.source
                {
                    *index = ref_idx;
                }
                sink.inputs.push(edge);
            }
        }
        return control;
    }
    let (mut control, _) = super::opportunity::controllers::controller(founder, family, inert);
    if family == Family::Scalar {
        let BackendDef::Graph(graph) =
            &mut control.nodes[super::opportunity::controllers::VOTE_NODE].backend_def
        else {
            unreachable!()
        };
        let fruit_only = graph
            .output_sinks
            .iter()
            .find(|sink| sink.kind == OutputSinkKind::ActionVote(VoteSink::Eat))
            .unwrap()
            .inputs
            .last()
            .unwrap()
            .source;
        graph
            .sink_mut(OutputSinkKind::ActionParam(ActionParamField::EatFoodType))
            .unwrap()
            .inputs
            .last_mut()
            .unwrap()
            .source = fruit_only;
    }
    control
}

#[cfg(test)]
mod tests;
