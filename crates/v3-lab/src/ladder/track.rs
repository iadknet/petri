//! The ladder's campaign bookkeeping: relevant sites, per-birth supply,
//! change signatures, per-generation child comparisons and retention.
//! Nothing here draws a random number.

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use rayon::prelude::*;
use serde::Serialize;
use v3_core::contracts::NodeId;
use v3_core::creature::genome::analysis::mesh_reachable_nodes;
use v3_core::creature::genome::cgp::OutputSinkKind;
use v3_core::creature::genome::{BackendDef, CreatureGenome, NodeGenome, VmInstruction, VoteSink};
use v3_core::mutation::{MutationEventOutcome, MutationSummary};
use v3_core::neighborhood::input_use::catalog::Family;
use v3_core::neighborhood::{classify, Class, Signature as BatterySignature};

use super::{Children, LadderRow, Pooled, RetentionRow, Supply, Targeted};
use crate::eval::{Boundary, SceneScore};
use crate::scene::Assay;

/// Whether `family` is one of the assay's relevant families.
#[must_use]
pub fn relevant_family(assay: Assay, family: Family) -> bool {
    let food = matches!(
        family,
        Family::FoodHere(_) | Family::NeighborFoodRing(_) | Family::AreaFoodSummary(_)
    );
    match assay {
        Assay::FoodSeeking => food,
        Assay::BarrierNavigation => {
            food || matches!(
                family,
                Family::NeighborBarrierRing | Family::AreaBarrierSummary
            )
        }
    }
}

/// The relevant families' labels, as `provenance.sizes` records them.
#[must_use]
pub fn relevant_family_labels(assay: Assay) -> Vec<String> {
    let mut labels = vec![
        "FoodHere(*)".to_owned(),
        "NeighborFoodRing(*)".to_owned(),
        "AreaFoodSummary(*)".to_owned(),
    ];
    if assay == Assay::BarrierNavigation {
        labels.push("NeighborBarrierRing".to_owned());
        labels.push("AreaBarrierSummary".to_owned());
    }
    labels
}

/// A sink the capability acts through: `Eat` or `Move(_)`.
fn motor_sink(sink: VoteSink) -> bool {
    matches!(sink, VoteSink::Eat | VoteSink::Move(_))
}

/// Whether `node` votes `Eat` or `Move(_)`: a graph `ActionVote` sink with
/// at least one input edge, or a VM `AddVote` naming such a sink.
#[must_use]
pub fn motor_node(node: &NodeGenome) -> bool {
    match &node.backend_def {
        BackendDef::Graph(graph) => graph.output_sinks.iter().any(|sink| {
            matches!(sink.kind, OutputSinkKind::ActionVote(vote) if motor_sink(vote))
                && !sink.inputs.is_empty()
        }),
        BackendDef::Vm(vm) => vm.program.iter().any(|instruction| {
            matches!(instruction, VmInstruction::AddVote { sink, .. }
                if VoteSink::from_index(usize::from(*sink)).is_some_and(motor_sink))
        }),
    }
}

/// A genome's relevant sites (reachable nodes that consume a relevant
/// family or vote `Eat`/`Move(_)`), by node id, and its reachable count.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sites {
    pub sites: BTreeSet<NodeId>,
    pub reachable: usize,
}

impl Sites {
    #[must_use]
    pub fn of(assay: Assay, genome: &CreatureGenome) -> Self {
        Self::with_reachable(assay, genome, &mesh_reachable_nodes(genome))
    }

    /// [`Sites::of`] with the reachable node indices already computed.
    #[must_use]
    pub fn with_reachable(assay: Assay, genome: &CreatureGenome, reachable: &[usize]) -> Self {
        let sensors: BTreeSet<usize> = crate::readings::consumers(genome)
            .into_iter()
            .filter(|(family, _)| relevant_family(assay, *family))
            .flat_map(|(_, nodes)| nodes)
            .collect();
        let sites = reachable
            .iter()
            .filter_map(|&index| genome.nodes.get(index).map(|node| (index, node)))
            .filter(|(index, node)| sensors.contains(index) || motor_node(node))
            .map(|(_, node)| node.node_id)
            .collect();
        Self {
            sites,
            reachable: reachable.len(),
        }
    }
}

/// The serde name of a key (`MutationOperator`, `MutationSkipReason`).
fn key_of<K: Serialize>(key: &K) -> String {
    match serde_json::to_value(key) {
        Ok(serde_json::Value::String(name)) => name,
        Ok(other) => other.to_string(),
        Err(_) => "unknown".into(),
    }
}

/// What distinguishes a child from its parent: the child's nodes that
/// differ from or are absent in the parent, the parent's nodes absent in the
/// child, and the child's entry node when it moved; each as `(id, content)`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Change {
    pub added: Vec<NodeGenome>,
    pub removed: Vec<NodeGenome>,
    pub entry: Option<NodeId>,
}

impl Change {
    #[must_use]
    pub fn between(parent: &CreatureGenome, child: &CreatureGenome) -> Self {
        let added = child
            .nodes
            .iter()
            .filter(|node| parent.find_node(node.node_id) != Some(*node))
            .cloned()
            .collect();
        let removed = parent
            .nodes
            .iter()
            .filter(|node| child.find_node(node.node_id).is_none())
            .cloned()
            .collect();
        let entry = (child.entry_node_id != parent.entry_node_id).then_some(child.entry_node_id);
        Self {
            added,
            removed,
            entry,
        }
    }

    /// Whether `genome` carries the change: every added pair present, no
    /// removed pair present (an unrelated node reusing the id is not the
    /// removed node), and the moved entry matching.
    #[must_use]
    pub fn carried_by(&self, genome: &CreatureGenome) -> bool {
        let present = |node: &NodeGenome| genome.find_node(node.node_id) == Some(node);
        self.added.iter().all(present)
            && !self.removed.iter().any(present)
            && self.entry.is_none_or(|entry| genome.entry_node_id == entry)
    }
}

/// One birth's ladder bookkeeping.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Birth {
    pub touching: bool,
    pub targeted: Targeted,
    pub discarded: u64,
    pub created: u64,
    pub removed: u64,
    /// The parent's relevant sites and reachable nodes.
    pub sites: u64,
    pub reachable: u64,
    /// Every applied event of the birth.
    pub applied: u64,
    pub change: Arc<Change>,
}

impl Birth {
    /// Classify a birth from its `MutationSummary`.
    #[must_use]
    pub fn of(
        assay: Assay,
        parent: &CreatureGenome,
        parent_sites: &Sites,
        child: &CreatureGenome,
        summary: &MutationSummary,
    ) -> Self {
        let on_site = |id: Option<NodeId>| id.is_some_and(|id| parent_sites.sites.contains(&id));
        let mut targeted = Targeted::default();
        let mut discarded = 0;
        for event in &summary.events {
            discarded += event
                .discarded
                .iter()
                .filter(|(_, target)| on_site(*target))
                .count() as u64;
            if !on_site(event.target) {
                continue;
            }
            let operator = event
                .operator
                .as_ref()
                .map_or_else(|| "none".to_owned(), key_of);
            *targeted.requested.entry(operator.clone()).or_default() += 1;
            match event.outcome {
                MutationEventOutcome::Applied(_) => {
                    *targeted.applied.entry(operator).or_default() += 1;
                }
                MutationEventOutcome::Skipped(reason) => {
                    *targeted.skipped.entry(operator).or_default() += 1;
                    *targeted
                        .skipped_by_reason
                        .entry(reason.as_key().to_owned())
                        .or_default() += 1;
                }
            }
        }
        let child_sites = Sites::of(assay, child);
        let created = child_sites.sites.difference(&parent_sites.sites).count() as u64;
        let removed = parent_sites.sites.difference(&child_sites.sites).count() as u64;
        let touching = targeted.applied.values().sum::<u64>() > 0 || created > 0 || removed > 0;
        Self {
            touching,
            targeted,
            discarded,
            created,
            removed,
            sites: parent_sites.sites.len() as u64,
            reachable: parent_sites.reachable as u64,
            applied: u64::from(summary.applied_events),
            change: Arc::new(Change::between(parent, child)),
        }
    }
}

impl Supply {
    /// Fold one birth in.
    pub fn add_birth(&mut self, birth: &Birth) {
        #[allow(clippy::cast_precision_loss)]
        let uniform_reference = (birth.reachable > 0)
            .then(|| birth.applied as f64 * birth.sites as f64 / birth.reachable as f64);
        self.add(&Self {
            births: 1,
            touching_births: u64::from(birth.touching),
            sites: birth.sites,
            reachable: birth.reachable,
            uniform_reference,
            targeted: birth.targeted.clone(),
            discarded: birth.discarded,
            created: birth.created,
            removed: birth.removed,
        });
    }
}

/// One evaluated member of an evolving lineage, as the ladder reads it.
#[derive(Debug, Clone, Copy)]
pub struct Member<'a> {
    pub id: u64,
    pub parent: Option<u64>,
    pub carried: bool,
    pub genome: &'a CreatureGenome,
    /// `Ancestry::applied`.
    pub applied: u64,
    /// `None` for founders and carried elites.
    pub birth: Option<&'a Birth>,
    pub identical: bool,
    pub scalar: f64,
    pub scenes: &'a [SceneScore],
    pub sequences: &'a [Vec<Boundary>],
}

impl Member<'_> {
    fn novel_child(&self) -> bool {
        self.birth.is_some() && !self.identical
    }
}

/// A selected improvement still followed.
#[derive(Debug, Clone)]
struct OpenChange {
    root_applied: u64,
    touching: bool,
    change: Arc<Change>,
    /// Ids in the last evaluated population descended from the change, its
    /// own carried copy included.
    lineage: BTreeSet<u64>,
    /// Per depth `1..=D`: still open.
    open: Vec<bool>,
}

/// A depth's resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    Retained,
    Deleted,
    LineageLoss,
}

impl Resolution {
    fn slot(self) -> usize {
        match self {
            Self::Retained => 0,
            Self::Deleted => 1,
            Self::LineageLoss => 2,
        }
    }
}

/// A generation's bookkeeping, applied only once its row is written.
#[derive(Debug, Clone)]
pub struct Pending {
    row: LadderRow,
    open: Vec<OpenChange>,
    signatures: HashMap<u64, BatterySignature>,
}

impl Pending {
    #[must_use]
    pub fn row(&self) -> &LadderRow {
        &self.row
    }
}

/// Per evolving lineage: the open changes, the parents' battery
/// signatures (carried genomes never change) and the pooled counts.
#[derive(Debug, Clone)]
pub struct Tracker {
    depth: usize,
    open: Vec<OpenChange>,
    signatures: HashMap<u64, BatterySignature>,
    pub pooled: Pooled,
}

/// Mean `progress` over the scenes.
fn mean_progress(scenes: &[SceneScore]) -> f64 {
    let values: Vec<f64> = scenes.iter().map(|s| s.progress).collect();
    crate::stats::mean(&values).unwrap_or(0.0)
}

/// Some training scene's score or `(position, energy)` sequence differs.
fn scene_changed(child: &Member<'_>, parent: &Member<'_>) -> bool {
    child.scenes != parent.scenes || child.sequences != parent.sequences
}

impl Tracker {
    #[must_use]
    pub fn new(depth: u32) -> Self {
        let depth = depth as usize;
        Self {
            depth,
            open: Vec::new(),
            signatures: HashMap::new(),
            pooled: Pooled::new(depth),
        }
    }

    /// One generation, after ranking and before the row. `survivors` are
    /// the ids breeding carries (`None` when no generation is bred);
    /// `signature` computes a genome's battery signature.
    #[must_use]
    pub fn step(
        &self,
        members: &[Member<'_>],
        survivors: Option<&[u64]>,
        signature: &(dyn Fn(&CreatureGenome) -> BatterySignature + Sync),
    ) -> Pending {
        let index: HashMap<u64, usize> =
            members.iter().enumerate().map(|(i, m)| (m.id, i)).collect();
        let parent_of = |child: &Member<'_>| -> Option<usize> {
            let parent = index.get(&child.parent?)?;
            members[*parent].carried.then_some(*parent)
        };

        let mut supply = Supply::default();
        for birth in members.iter().filter_map(|m| m.birth) {
            supply.add_birth(birth);
        }

        // Battery signatures: touching novel children and their parents.
        let touching: Vec<(usize, usize)> = members
            .iter()
            .enumerate()
            .filter(|(_, m)| m.novel_child() && m.birth.is_some_and(|b| b.touching))
            .filter_map(|(i, m)| parent_of(m).map(|p| (i, p)))
            .collect();
        // Parents not cached by an earlier generation are computed here.
        let missing: BTreeSet<usize> = touching
            .iter()
            .map(|&(_, p)| p)
            .filter(|&p| !self.signatures.contains_key(&members[p].id))
            .collect();
        let signatures: HashMap<u64, BatterySignature> = missing
            .par_iter()
            .map(|&p| (members[p].id, signature(members[p].genome)))
            .collect();
        let lookup = |id: &u64| signatures.get(id).or_else(|| self.signatures.get(id));
        let classes: Vec<Class> = touching
            .par_iter()
            .map(|&(c, p)| {
                let parent = lookup(&members[p].id).expect("every parent has a signature");
                classify(parent, &signature(members[c].genome)).class
            })
            .collect();

        let mut children = Children::default();
        let mut improved_ids = BTreeSet::new();
        let mut classes = classes.into_iter();
        for child in members.iter().filter(|m| m.novel_child()) {
            let Some(p) = parent_of(child) else {
                continue;
            };
            let parent = &members[p];
            let changed = scene_changed(child, parent);
            let improved = child.scalar > parent.scalar;
            let progress = mean_progress(child.scenes) > mean_progress(parent.scenes);
            if improved {
                improved_ids.insert(child.id);
            }
            if child.birth.is_some_and(|b| b.touching) {
                let class = classes.next().expect("one class per touching child");
                let t = &mut children.touching;
                t.children += 1;
                t.scene_changed += u64::from(changed);
                match class {
                    Class::Silent => t.silent += 1,
                    Class::Changed => t.changed += 1,
                    Class::Dead => t.dead += 1,
                }
                let viable = class != Class::Dead && (class == Class::Changed || changed);
                if viable {
                    t.viable += 1;
                    t.outcomes.add(child.scalar, parent.scalar, progress);
                }
            } else {
                let o = &mut children.other;
                o.children += 1;
                o.scene_changed += u64::from(changed);
                o.outcomes.add(child.scalar, parent.scalar, progress);
            }
        }

        // Retention: resolve the open changes on this population.
        let survivor_set: BTreeSet<u64> = survivors.unwrap_or(&[]).iter().copied().collect();
        let mut retention = RetentionRow::new(self.depth);
        let mut open = Vec::with_capacity(self.open.len());
        for change in &self.open {
            let mut next = OpenChange {
                root_applied: change.root_applied,
                touching: change.touching,
                change: Arc::clone(&change.change),
                lineage: members
                    .iter()
                    .filter(|m| {
                        change.lineage.contains(&m.id)
                            || m.parent
                                .is_some_and(|p| change.lineage.contains(&p) && !m.carried)
                    })
                    .map(|m| m.id)
                    .collect(),
                open: change.open.clone(),
            };
            let descendants: Vec<(&Member<'_>, u64, bool)> = members
                .iter()
                .filter(|m| next.lineage.contains(&m.id))
                .map(|m| {
                    (
                        m,
                        m.applied.saturating_sub(change.root_applied),
                        change.change.carried_by(m.genome),
                    )
                })
                .collect();
            for d in 0..self.depth {
                if !next.open[d] {
                    continue;
                }
                let depth = d as u64 + 1;
                let resolution = if descendants.iter().any(|(m, at, carries)| {
                    *carries && *at >= depth && survivor_set.contains(&m.id)
                }) {
                    Some(Resolution::Retained)
                } else if descendants.is_empty() {
                    Some(Resolution::LineageLoss)
                } else if descendants.iter().all(|(_, _, carries)| !carries) {
                    Some(Resolution::Deleted)
                } else {
                    None
                };
                if let Some(resolution) = resolution {
                    next.open[d] = false;
                    retention.record(d, resolution, change.touching);
                }
            }
            if next.open.iter().any(|&o| o) {
                open.push(next);
            }
        }
        // New selected improvements: improved novel children that survive.
        for child in members.iter().filter(|m| m.novel_child()) {
            if !(improved_ids.contains(&child.id) && survivor_set.contains(&child.id)) {
                continue;
            }
            let birth = child.birth.expect("novel children have births");
            retention.selected += 1;
            retention.selected_touching += u64::from(birth.touching);
            open.push(OpenChange {
                root_applied: child.applied,
                touching: birth.touching,
                change: Arc::clone(&birth.change),
                lineage: BTreeSet::from([child.id]),
                open: vec![true; self.depth],
            });
        }

        // Keep the signatures of the members breeding carries.
        let mut kept: HashMap<u64, BatterySignature> = HashMap::new();
        for id in &survivor_set {
            if let Some(found) = lookup(id) {
                kept.insert(*id, found.clone());
            }
        }
        Pending {
            row: LadderRow {
                supply,
                children,
                retention,
            },
            open,
            signatures: kept,
        }
    }

    /// Apply a written row's bookkeeping.
    pub fn commit(&mut self, pending: Pending) {
        self.pooled.add(&pending.row);
        self.open = pending.open;
        self.signatures = pending.signatures;
    }

    /// Censor every open depth at the lineage's stop.
    pub fn censor(&mut self) {
        for change in self.open.drain(..) {
            for (d, open) in change.open.iter().enumerate() {
                if *open {
                    self.pooled.retention.censored[d] += 1;
                    if change.touching {
                        self.pooled.retention.censored_touching[d] += 1;
                    }
                }
            }
        }
    }
}

impl RetentionRow {
    fn record(&mut self, d: usize, resolution: Resolution, touching: bool) {
        self.depths[d][resolution.slot()] += 1;
        if touching {
            self.depths_touching[d][resolution.slot()] += 1;
        }
    }
}

#[cfg(test)]
mod tests;
