use rand::Rng;

use crate::types::NodeKind;

use super::node_utils::{clamp, is_hidden_node, is_input_node, is_output_node, random_hidden_node};
use super::*;

impl ComputationGraph {
    pub fn mutate_weights<R: Rng>(&mut self, rng: &mut R, rate: f32, magnitude: f32) -> bool {
        let rate = rate.clamp(0.0, 1.0);
        let magnitude = magnitude.abs();
        let mut changed = false;

        for edge in &mut self.edges {
            if rng.gen_bool(rate as f64) {
                let before = edge.weight;
                edge.weight = clamp(
                    edge.weight + rng.gen_range(-magnitude..=magnitude),
                    -8.0,
                    8.0,
                );
                changed |= (edge.weight - before).abs() > f32::EPSILON;
            }
        }

        for node in &mut self.nodes {
            if rng.gen_bool(rate as f64) {
                match node {
                    NodeKind::Constant(v) => {
                        let before = *v;
                        *v = clamp(*v + rng.gen_range(-magnitude..=magnitude), -8.0, 8.0);
                        changed |= (*v - before).abs() > f32::EPSILON;
                    }
                    NodeKind::Threshold(t) => {
                        let before = *t;
                        *t = clamp(*t + rng.gen_range(-magnitude..=magnitude), 0.0, 1.0);
                        changed |= (*t - before).abs() > f32::EPSILON;
                    }
                    _ => {}
                }
            }
        }

        changed
    }

    pub fn mutate_with_config<R: Rng>(&mut self, rng: &mut R, cfg: MutationConfig) -> bool {
        let mut changed = false;
        changed |=
            self.mutate_weights(rng, cfg.weight_mutation_rate, cfg.weight_mutation_magnitude);

        let structural_rate = cfg.structural_mutation_rate.clamp(0.0, 1.0);
        if rng.gen_bool(structural_rate as f64) {
            changed |= self.add_hidden_node_by_splicing_edge(rng);
        }
        if rng.gen_bool(structural_rate as f64) {
            changed |= self.add_edge_mutation(rng);
        }
        if rng.gen_bool(structural_rate as f64) {
            changed |= self.remove_edge_mutation(rng);
        }
        if rng.gen_bool(structural_rate as f64) {
            changed |= self.remove_disconnected_hidden_nodes() > 0;
        }

        let logic_rate = cfg.logic_node_mutation_rate.clamp(0.0, 1.0);
        if rng.gen_bool(logic_rate as f64) {
            changed |= self.change_hidden_node_type(rng);
        }

        changed
    }

    pub fn add_hidden_node_by_splicing_edge<R: Rng>(&mut self, rng: &mut R) -> bool {
        let candidate_indices = self
            .edges
            .iter()
            .enumerate()
            .filter_map(
                |(idx, edge)| {
                    if edge.from < edge.to {
                        Some(idx)
                    } else {
                        None
                    }
                },
            )
            .collect::<Vec<_>>();
        if candidate_indices.is_empty() {
            return false;
        }

        let edge_idx = candidate_indices[rng.gen_range(0..candidate_indices.len())];
        let edge = self.edges.remove(edge_idx);
        let insert_at = edge.to;

        for existing in &mut self.edges {
            if existing.from >= insert_at {
                existing.from += 1;
            }
            if existing.to >= insert_at {
                existing.to += 1;
            }
        }

        self.nodes.insert(insert_at, random_hidden_node(rng));
        let shifted_to = edge.to + 1;

        self.edges.push(Edge {
            from: edge.from,
            to: insert_at,
            weight: 1.0,
        });
        self.edges.push(Edge {
            from: insert_at,
            to: shifted_to,
            weight: edge.weight,
        });

        true
    }

    pub fn add_edge_mutation<R: Rng>(&mut self, rng: &mut R) -> bool {
        let mut candidates = Vec::new();

        for from in 0..self.nodes.len() {
            if is_output_node(&self.nodes[from]) {
                continue;
            }
            for to in (from + 1)..self.nodes.len() {
                if is_input_node(&self.nodes[to]) || self.has_edge(from, to) {
                    continue;
                }
                candidates.push((from, to));
            }
        }

        if candidates.is_empty() {
            return false;
        }

        let (from, to) = candidates[rng.gen_range(0..candidates.len())];
        self.edges.push(Edge {
            from,
            to,
            weight: rng.gen_range(-1.0..=1.0),
        });
        true
    }

    pub fn remove_edge_mutation<R: Rng>(&mut self, rng: &mut R) -> bool {
        if self.edges.is_empty() {
            return false;
        }
        let idx = rng.gen_range(0..self.edges.len());
        self.edges.swap_remove(idx);
        true
    }

    pub fn remove_disconnected_hidden_nodes(&mut self) -> usize {
        if self.nodes.is_empty() {
            return 0;
        }

        let mut incoming = vec![0_usize; self.nodes.len()];
        let mut outgoing = vec![0_usize; self.nodes.len()];
        for edge in &self.edges {
            if edge.to < incoming.len() {
                incoming[edge.to] += 1;
            }
            if edge.from < outgoing.len() {
                outgoing[edge.from] += 1;
            }
        }

        let mut removable = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                if is_hidden_node(node) && incoming[idx] == 0 && outgoing[idx] == 0 {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        removable.sort_unstable();
        let removed = removable.len();
        for idx in removable.into_iter().rev() {
            self.remove_node(idx);
        }
        removed
    }

    pub fn change_hidden_node_type<R: Rng>(&mut self, rng: &mut R) -> bool {
        let hidden_indices = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(idx, node)| {
                if is_hidden_node(node) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if hidden_indices.is_empty() {
            return false;
        }

        let idx = hidden_indices[rng.gen_range(0..hidden_indices.len())];
        let original = self.nodes[idx].clone();
        let mut replacement = random_hidden_node(rng);
        for _ in 0..8 {
            if replacement != original {
                break;
            }
            replacement = random_hidden_node(rng);
        }
        if replacement == original {
            return false;
        }
        self.nodes[idx] = replacement;
        true
    }
    fn has_edge(&self, from: usize, to: usize) -> bool {
        self.edges
            .iter()
            .any(|edge| edge.from == from && edge.to == to)
    }

    fn remove_node(&mut self, idx: usize) {
        self.nodes.remove(idx);
        self.edges.retain(|edge| edge.from != idx && edge.to != idx);
        for edge in &mut self.edges {
            if edge.from > idx {
                edge.from -= 1;
            }
            if edge.to > idx {
                edge.to -= 1;
            }
        }
    }
}
