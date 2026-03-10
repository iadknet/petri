//! CGP graph backend reproduction helpers.
//!
//! Ports `build_child_plasticity_weights` to work with `CgpGraphBackendDef`
//! (iterates `compute_nodes` instead of `internal_nodes`).

use crate::creature::genome::cgp::CgpGraphBackendDef;

/// Build child plasticity weights from parent state for a CGP graph backend.
///
/// For each compute node in the CGP graph:
/// - `plasticity.lamarckian == true`: copies parent's learned weights if available
/// - `plasticity.lamarckian == false` or `plasticity == None`: empty `Box<[f32]>`
///   (reinit from genome on first tick)
///
/// Returns a Vec indexed by compute_node_idx, containing learned weight slices.
#[must_use]
pub(crate) fn build_cgp_child_plasticity_weights(
    cgp_def: &CgpGraphBackendDef,
    parent_weights: &[Box<[f32]>],
) -> Vec<Box<[f32]>> {
    let node_count = cgp_def.compute_nodes.len();
    let mut result: Vec<Box<[f32]>> = Vec::with_capacity(node_count);

    for (idx, compute_node) in cgp_def.compute_nodes.iter().enumerate() {
        let should_copy = compute_node
            .plasticity
            .as_ref()
            .is_some_and(|cfg| cfg.lamarckian);

        if should_copy {
            let parent_w = parent_weights.get(idx).filter(|w| !w.is_empty());

            if let Some(pw) = parent_w {
                result.push(pw.clone());
            } else {
                result.push(Box::new([]));
            }
        } else {
            result.push(Box::new([]));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MutationConfig;
    use crate::creature::genome::cgp::{ComputeNode, ComputeNodeKind, GraphEdge, GraphSource};
    use crate::creature::genome::{HebbianRule, PlasticityConfig};

    fn simple_cgp_def() -> CgpGraphBackendDef {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        // CN0: no plasticity
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Constant(1.0),
            inputs: Vec::new(),
            plasticity: None,
        });

        // CN1: Lamarckian plasticity
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(0),
                weight: 0.5,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: true,
                modulation: None,
            }),
        });

        // CN2: Darwinian plasticity
        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Sigmoid,
            inputs: vec![GraphEdge {
                source: GraphSource::ComputeNode(1),
                weight: 0.3,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Oja,
                learning_rate: 0.05,
                weight_clamp: 3.0,
                lamarckian: false,
                modulation: None,
            }),
        });

        def
    }

    #[test]
    fn lamarckian_copies_parent_weights() {
        let def = simple_cgp_def();
        let parent_weights: Vec<Box<[f32]>> = vec![
            Box::new([]),    // CN0: no plasticity
            Box::new([0.9]), // CN1: learned weight
            Box::new([0.7]), // CN2: learned weight
        ];

        let child = build_cgp_child_plasticity_weights(&def, &parent_weights);

        assert_eq!(child.len(), 3);
        // CN0: no plasticity → empty
        assert!(child[0].is_empty());
        // CN1: Lamarckian → copied from parent
        assert_eq!(child[1].len(), 1);
        assert!((child[1][0] - 0.9).abs() < 1e-6);
        // CN2: Darwinian → empty (lazy-init)
        assert!(child[2].is_empty());
    }

    #[test]
    fn lamarckian_with_no_parent_weights() {
        let def = simple_cgp_def();
        let parent_weights: Vec<Box<[f32]>> = Vec::new();

        let child = build_cgp_child_plasticity_weights(&def, &parent_weights);

        assert_eq!(child.len(), 3);
        // All empty — parent had no weights yet
        for w in &child {
            assert!(w.is_empty());
        }
    }

    #[test]
    fn empty_graph_produces_empty_weights() {
        let config = MutationConfig::default();
        let def = CgpGraphBackendDef::new_with_fixed_outputs(&config);
        let parent_weights: Vec<Box<[f32]>> = Vec::new();

        let child = build_cgp_child_plasticity_weights(&def, &parent_weights);
        assert!(child.is_empty());
    }

    #[test]
    fn darwinian_always_resets() {
        let config = MutationConfig::default();
        let mut def = CgpGraphBackendDef::new_with_fixed_outputs(&config);

        def.compute_nodes.push(ComputeNode {
            kind: ComputeNodeKind::Add,
            inputs: vec![GraphEdge {
                source: GraphSource::InputLeaf {
                    ref_idx: 0,
                    sub_idx: 0,
                },
                weight: 0.5,
            }],
            plasticity: Some(PlasticityConfig {
                rule: HebbianRule::Classic,
                learning_rate: 0.1,
                weight_clamp: 5.0,
                lamarckian: false,
                modulation: None,
            }),
        });

        let parent_weights: Vec<Box<[f32]>> = vec![Box::new([0.9])];
        let child = build_cgp_child_plasticity_weights(&def, &parent_weights);

        assert_eq!(child.len(), 1);
        assert!(child[0].is_empty()); // Darwinian: reset to empty
    }
}
