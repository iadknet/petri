//! Reachability-aware mutation target selection.
//!
//! Provides biased node selection that preferentially targets mesh nodes reachable
//! from the entry node while preserving drift on unreachable structure.
//!
//! The parent's cached reachable set is used to bias mutations on the offspring.
//! After mutations, the offspring's actual reachable set may differ. This is
//! intentional — mutations are biased toward what was functional in the parent.
//! The offspring gets a fresh BFS via `CreatureState::new()`. Within a birth,
//! [`BirthMembership`] maps the parent's sets onto the child's current nodes,
//! so earlier removals and additions never move membership between nodes.

use std::borrow::Cow;
use std::cmp::Ordering;

use rand::Rng;

use super::types::TargetReachability;
use crate::contracts::NodeId;
use crate::creature::genome::NodeGenome;
use crate::creature::state::DispatchRecord;

/// Where a birth's executed node set comes from.
///
/// The set is resolved only once the birth draws at least one mutation event,
/// so zero-event births derive nothing (T11.F17).
#[derive(Debug, Clone, Copy)]
pub enum ParentExecuted<'a> {
    /// An explicit sorted ascending index set, as the observation harnesses
    /// derive it from battery hop records.
    Indices(&'a [usize]),
    /// The live parent's dispatch record, read at the parent's current age.
    Record(&'a DispatchRecord, u64),
}

impl ParentExecuted<'_> {
    /// No dispatch information: the executed layer never fires.
    pub const NONE: Self = Self::Indices(&[]);

    /// Resolve to the sorted indices executed within `window` ticks.
    #[must_use]
    pub fn resolve(&self, window: u64) -> Cow<'_, [usize]> {
        match *self {
            Self::Indices(indices) => Cow::Borrowed(indices),
            Self::Record(record, age) => Cow::Owned(record.executed_indices(age, window)),
        }
    }
}

/// Select a node index from `eligible` with probabilistic bias toward reachable nodes.
///
/// Both `eligible` and `reachable` must be sorted ascending.
///
/// - `bias = 0.0`: uniform selection from `eligible` (no reachability preference).
/// - `bias = 1.0`: always select from reachable nodes when any are eligible.
/// - `bias = -1.0`: always select from unreachable nodes when any are eligible.
///
/// Returns `(selected_index, classification)` or `None` if `eligible` is empty.
#[must_use]
pub fn biased_select_from(
    eligible: &[usize],
    reachable: &[usize],
    bias: f64,
    rng: &mut impl Rng,
) -> Option<(usize, TargetReachability)> {
    if eligible.is_empty() {
        return None;
    }

    let prefer_reachable = bias >= 0.0;
    let bias_clamped = if bias.is_finite() {
        bias.abs().clamp(0.0, 1.0)
    } else {
        0.0
    };

    // Attempt biased selection toward the preferred class
    if bias_clamped > 0.0 && rng.gen_bool(bias_clamped) {
        let reachable_count = intersection_count(eligible, reachable);
        let count = if prefer_reachable {
            reachable_count
        } else {
            eligible.len() - reachable_count
        };
        if count > 0 {
            let pick = rng.gen_range(0..count);
            let idx = if prefer_reachable {
                nth_intersection(eligible, reachable, pick)
            } else {
                nth_difference(eligible, reachable, pick)
            };
            let class = if prefer_reachable {
                TargetReachability::Reachable
            } else {
                TargetReachability::Unreachable
            };
            return Some((idx, class));
        }
        // No preferred-class nodes in eligible set — fall through to uniform
    }

    // Uniform selection from all eligible nodes
    let picked = eligible[rng.gen_range(0..eligible.len())];
    let classification = classify_target(picked, reachable);
    Some((picked, classification))
}

#[cfg(test)]
thread_local! {
    /// Test-only oracle switch: while set on a thread, membership never
    /// follows removals, reproducing the pre-T11.F24 index-set targeting so
    /// equivalence can be checked against it.
    pub(crate) static FROZEN_MEMBERSHIP: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

/// Which of a child's current mesh nodes are parent-set members, maintained
/// across the events of one birth (T11.F24).
///
/// A child node is a member of the parent's reachable (executed) set exactly
/// when it has been present since the birth began, no event of this birth
/// created it, and its parent index is in the parent's set. Until the birth's
/// first applied mesh-node removal the child's first nodes sit at their parent
/// indices and every added node sits after them, so the parent's index sets
/// are borrowed as they are. The first removal copies their entries below the
/// parent's node count into owned sets; each removal then drops the removed
/// index and shifts every later member down by one. Carried nodes always
/// precede the nodes created in this birth (events only append or remove), so
/// a member index never lands on a created node, even one on a removed node's
/// `NodeId` or former index. Maintaining this consumes no RNG and costs
/// O(member count) per removal.
#[derive(Debug)]
pub struct BirthMembership<'a> {
    parent_reachable: &'a [usize],
    parent_executed: &'a [usize],
    /// The child's node count when the birth began: its parent's node count.
    parent_len: usize,
    /// The current reachable and executed members from the birth's first
    /// applied mesh-node removal on; `None` before it.
    repaired: Option<(Vec<usize>, Vec<usize>)>,
}

impl<'a> BirthMembership<'a> {
    /// Membership at the start of a birth whose child copies a parent of
    /// `parent_len` mesh nodes. Both sets are sorted ascending parent indices.
    #[must_use]
    pub const fn new(
        parent_reachable: &'a [usize],
        parent_executed: &'a [usize],
        parent_len: usize,
    ) -> Self {
        Self {
            parent_reachable,
            parent_executed,
            parent_len,
            repaired: None,
        }
    }

    /// The member sets as sorted ascending current child indices.
    #[must_use]
    pub fn sets(&self) -> TargetSets<'_> {
        match &self.repaired {
            Some((reachable, executed)) => TargetSets::new(reachable, executed),
            None => TargetSets::new(self.parent_reachable, self.parent_executed),
        }
    }

    /// Follow one mutation event from the node ids the child carried before
    /// it to the nodes it carries after it.
    ///
    /// Relies on the within-event identity of every mesh-node-changing
    /// operator: one event either removes a single node, keeping the others
    /// in order, or appends nodes on ids the child did not carry. A discarded
    /// or rejected attempt restores the genome, so it arrives here unchanged.
    /// Appends need no handling: every member index is below the child's
    /// length before the event.
    pub fn observe_event(&mut self, before: &[NodeId], after: &[NodeGenome]) {
        #[cfg(test)]
        if FROZEN_MEMBERSHIP.with(std::cell::Cell::get) {
            return;
        }
        if after.len() >= before.len() {
            return;
        }
        debug_assert_eq!(after.len() + 1, before.len(), "one event removes one node");
        let removed = before
            .iter()
            .zip(after)
            .position(|(id, node)| *id != node.node_id)
            .unwrap_or(after.len());
        // Only parent indices name carried nodes; a set entry at or past
        // `parent_len` never had a node to follow.
        let carried = |set: &[usize]| -> Vec<usize> {
            set[..set.partition_point(|&index| index < self.parent_len)].to_vec()
        };
        let (reachable, executed) = self.repaired.get_or_insert_with(|| {
            (
                carried(self.parent_reachable),
                carried(self.parent_executed),
            )
        });
        for set in [reachable, executed] {
            set.retain_mut(|index| match (*index).cmp(&removed) {
                Ordering::Less => true,
                Ordering::Equal => false,
                Ordering::Greater => {
                    *index -= 1;
                    true
                }
            });
        }
    }
}

/// The child nodes that are members of the parent's reachable and recently
/// executed sets, from which one [`TargetSelector`] is built per mutation
/// event.
#[derive(Debug, Clone, Copy)]
pub struct TargetSets<'a> {
    reachable: &'a [usize],
    executed: &'a [usize],
}

impl<'a> TargetSets<'a> {
    /// Both sets are sorted ascending mesh node indices in the same genome.
    #[must_use]
    pub const fn new(reachable: &'a [usize], executed: &'a [usize]) -> Self {
        Self {
            reachable,
            executed,
        }
    }

    /// A selector over these sets at the given domain biases.
    #[must_use]
    pub fn selector(&self, reachable_bias: f64, executed_bias: f64) -> TargetSelector<'a> {
        TargetSelector::new(self.reachable, self.executed, reachable_bias, executed_bias)
    }
}

/// The parent-derived inputs to one mutation event's target draws, and the
/// tally of picks that landed on a node the parent recently executed.
///
/// Both node sets are sorted ascending mesh node indices into the child as it
/// stands before the event: the current members of the parent's `reachable`
/// set (entry-node BFS) and `executed` set (the parent's dispatch record
/// within the configured window, T11.F17), as [`BirthMembership`] maps them.
/// One selector is built per mutation event and passed to the domain mutator,
/// which draws every target of that event through [`TargetSelector::select`].
#[derive(Debug)]
pub struct TargetSelector<'a> {
    reachable: &'a [usize],
    executed: &'a [usize],
    reachable_bias: f64,
    executed_bias: f64,
    executed_hits: u32,
    first_pick: Option<usize>,
}

impl<'a> TargetSelector<'a> {
    /// Build a selector over the parent's node sets and the domain's biases.
    const fn new(
        reachable: &'a [usize],
        executed: &'a [usize],
        reachable_bias: f64,
        executed_bias: f64,
    ) -> Self {
        Self {
            reachable,
            executed,
            reachable_bias,
            executed_bias,
            executed_hits: 0,
            first_pick: None,
        }
    }

    /// A selector with no executed layer, for fixtures that hold no dispatch
    /// information.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn reachable_only(reachable: &'a [usize], reachable_bias: f64) -> Self {
        Self::new(reachable, &[], reachable_bias, 0.0)
    }

    /// The first node index this selector returned, or `None` when it never
    /// returned one. Read after the operator runs to record which node a
    /// mutation event selected (T13.F01); purely observational.
    #[must_use]
    pub const fn first_pick(&self) -> Option<usize> {
        self.first_pick
    }

    /// How many targets this selector picked from the executed set.
    #[must_use]
    pub const fn executed_hits(&self) -> u32 {
        self.executed_hits
    }

    /// Select a node index from `eligible`, preferring nodes the parent
    /// executed recently and falling back to the reachability-biased draw.
    ///
    /// `eligible` must be sorted ascending. When every eligible node is
    /// executed the executed layer cannot change the outcome, so it is skipped
    /// entirely and no RNG is consumed; a zero executed bias behaves the same.
    /// Otherwise one roll decides whether to draw uniformly from
    /// `eligible ∩ executed`, falling through to [`biased_select_from`] on a
    /// failed roll or an empty intersection.
    pub fn select(
        &mut self,
        eligible: &[usize],
        rng: &mut impl Rng,
    ) -> Option<(usize, TargetReachability)> {
        if eligible.is_empty() {
            return None;
        }
        if self.executed_bias > 0.0 {
            let executed_count = intersection_count(eligible, self.executed);
            // The roll sits between the two guards deliberately: an eligible
            // set that is entirely executed consumes no RNG (short-circuit),
            // while an empty intersection still spends the one roll before
            // falling through.
            if executed_count < eligible.len()
                && rng.gen_bool(self.executed_bias)
                && executed_count > 0
            {
                let pick = rng.gen_range(0..executed_count);
                let index = nth_intersection(eligible, self.executed, pick);
                self.executed_hits += 1;
                self.remember(index);
                return Some((index, classify_target(index, self.reachable)));
            }
        }
        let picked = biased_select_from(eligible, self.reachable, self.reachable_bias, rng)?;
        if self.executed.binary_search(&picked.0).is_ok() {
            self.executed_hits += 1;
        }
        self.remember(picked.0);
        Some(picked)
    }

    /// Keep the first returned index, ignoring every later one.
    fn remember(&mut self, index: usize) {
        self.first_pick.get_or_insert(index);
    }
}

/// Classify whether a node index is reachable.
#[must_use]
pub fn classify_target(node_idx: usize, reachable: &[usize]) -> TargetReachability {
    if reachable.binary_search(&node_idx).is_ok() {
        TargetReachability::Reachable
    } else {
        TargetReachability::Unreachable
    }
}

/// Count elements in the intersection of two sorted slices (two-pointer merge).
fn intersection_count(a: &[usize], b: &[usize]) -> usize {
    let (mut i, mut j, mut count) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                count += 1;
                i += 1;
                j += 1;
            }
        }
    }
    count
}

/// Find the k-th element (0-based) in the intersection of two sorted slices.
///
/// Panics if `k >= intersection_count(a, b)`.
fn nth_intersection(a: &[usize], b: &[usize], k: usize) -> usize {
    let (mut i, mut j, mut found) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                if found == k {
                    return a[i];
                }
                found += 1;
                i += 1;
                j += 1;
            }
        }
    }
    panic!("k={k} out of range for intersection");
}

/// Find the k-th element (0-based) in `a \ b` for sorted slices.
///
/// Panics if `k` is out of range for the difference.
fn nth_difference(a: &[usize], b: &[usize], k: usize) -> usize {
    let (mut i, mut j, mut found) = (0usize, 0usize, 0usize);
    while i < a.len() {
        while j < b.len() && b[j] < a[i] {
            j += 1;
        }
        let in_b = j < b.len() && b[j] == a[i];
        if !in_b {
            if found == k {
                return a[i];
            }
            found += 1;
        }
        i += 1;
    }
    panic!("k={k} out of range for difference");
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    fn seeded_rng(seed: u64) -> SmallRng {
        SmallRng::seed_from_u64(seed)
    }

    /// Sorted ascending indices selected by `flags` from `0..flags.len()`.
    fn subset(flags: &[bool]) -> Vec<usize> {
        flags
            .iter()
            .enumerate()
            .filter(|(_, &keep)| keep)
            .map(|(index, _)| index)
            .collect()
    }

    /// The draw and the RNG stream after it, for equality against the
    /// pre-T11.F17 draw.
    fn draw_and_rng_tail(
        mut selector: TargetSelector<'_>,
        eligible: &[usize],
        rng: &mut SmallRng,
    ) -> (Option<(usize, TargetReachability)>, u64, u32) {
        let picked = selector.select(eligible, rng);
        (picked, rng.gen::<u64>(), selector.executed_hits())
    }

    proptest! {
        /// With the executed layer fully on, every pick lands on an executed
        /// node whenever the eligible set contains one.
        #[test]
        fn executed_bias_one_always_picks_an_executed_eligible_node(
            flags in prop::collection::vec((any::<bool>(), any::<bool>(), any::<bool>()), 1..12),
            seed in any::<u64>(),
            reachable_bias in 0.0f64..=1.0,
        ) {
            let eligible = subset(&flags.iter().map(|f| f.0).collect::<Vec<_>>());
            let reachable = subset(&flags.iter().map(|f| f.1).collect::<Vec<_>>());
            let executed = subset(&flags.iter().map(|f| f.2).collect::<Vec<_>>());
            let mut rng = seeded_rng(seed);
            let mut selector =
                TargetSelector::new(&reachable, &executed, reachable_bias, 1.0);
            let picked = selector.select(&eligible, &mut rng);
            match picked {
                None => prop_assert!(eligible.is_empty()),
                Some((index, class)) => {
                    prop_assert!(eligible.contains(&index));
                    prop_assert_eq!(class, classify_target(index, &reachable));
                    if eligible.iter().any(|node| executed.contains(node)) {
                        prop_assert!(executed.contains(&index));
                    }
                    prop_assert_eq!(
                        selector.executed_hits(),
                        u32::from(executed.contains(&index))
                    );
                }
            }
        }

        /// A zero executed bias, and an eligible set entirely inside the
        /// executed set, both leave the existing draw and its RNG untouched.
        #[test]
        fn executed_layer_is_transparent_at_zero_bias_and_under_the_short_circuit(
            flags in prop::collection::vec((any::<bool>(), any::<bool>()), 1..12),
            seed in any::<u64>(),
            reachable_bias in 0.0f64..=1.0,
            executed_bias in 0.0f64..=1.0,
        ) {
            let eligible = subset(&flags.iter().map(|f| f.0).collect::<Vec<_>>());
            let reachable = subset(&flags.iter().map(|f| f.1).collect::<Vec<_>>());
            let mut expected_rng = seeded_rng(seed);
            let expected = (
                biased_select_from(&eligible, &reachable, reachable_bias, &mut expected_rng),
                expected_rng.gen::<u64>(),
            );

            // Zero bias: the executed set is irrelevant and no roll happens.
            let mut zero_rng = seeded_rng(seed);
            let (picked, tail, hits) = draw_and_rng_tail(
                TargetSelector::new(&reachable, &eligible, reachable_bias, 0.0),
                &eligible,
                &mut zero_rng,
            );
            prop_assert_eq!(picked, expected.0);
            prop_assert_eq!(tail, expected.1);
            prop_assert_eq!(hits, u32::from(picked.is_some()));

            // Short-circuit: every eligible node is executed.
            let mut short_rng = seeded_rng(seed);
            let (picked, tail, _) = draw_and_rng_tail(
                TargetSelector::new(&reachable, &eligible, reachable_bias, executed_bias),
                &eligible,
                &mut short_rng,
            );
            prop_assert_eq!(picked, expected.0);
            prop_assert_eq!(tail, expected.1);
        }

        /// An empty intersection consumes one roll and then draws exactly as
        /// the reachable-bias layer would.
        #[test]
        fn an_empty_executed_intersection_falls_through_after_one_roll(
            flags in prop::collection::vec((any::<bool>(), any::<bool>()), 1..12),
            seed in any::<u64>(),
            reachable_bias in 0.0f64..=1.0,
        ) {
            let eligible = subset(&flags.iter().map(|f| f.0).collect::<Vec<_>>());
            prop_assume!(!eligible.is_empty());
            let reachable = subset(&flags.iter().map(|f| f.1).collect::<Vec<_>>());
            let mut expected_rng = seeded_rng(seed);
            let _rolled = expected_rng.gen_bool(0.75);
            let expected = biased_select_from(&eligible, &reachable, reachable_bias, &mut expected_rng);

            let mut rng = seeded_rng(seed);
            let mut selector = TargetSelector::new(&reachable, &[], reachable_bias, 0.75);
            prop_assert_eq!(selector.select(&eligible, &mut rng), expected);
            prop_assert_eq!(selector.executed_hits(), 0);
            prop_assert_eq!(rng.gen::<u64>(), expected_rng.gen::<u64>());
        }
    }

    /// One scripted change to a child's node vector, as one applied event
    /// makes it: remove the node at an index, append nodes on fresh ids the
    /// way `next_node_id` allocates them, or leave the vector alone (a
    /// node-internal event, or an attempt that was rolled back).
    #[derive(Debug, Clone)]
    enum Step {
        Remove(usize),
        Append(u8),
        Unchanged,
    }

    fn step() -> impl Strategy<Value = Step> {
        prop_oneof![
            any::<usize>().prop_map(Step::Remove),
            (1u8..4).prop_map(Step::Append),
            Just(Step::Unchanged),
        ]
    }

    fn bare_node(node_id: NodeId) -> NodeGenome {
        NodeGenome {
            node_id,
            input_refs: Vec::new(),
            backend_def: crate::creature::genome::BackendDef::Vm(
                crate::creature::genome::VmBackendDef {
                    register_count: 1,
                    constants: Vec::new(),
                    program: vec![crate::creature::genome::VmInstruction::Halt],
                },
            ),
            targets: Vec::new(),
        }
    }

    /// The next free id at or above the current maximum plus one, as
    /// `topology::structural::next_node_id` allocates it: removing the
    /// maximum id makes it free again.
    fn next_free_id(nodes: &[(NodeId, Option<usize>)]) -> NodeId {
        let mut candidate = nodes
            .iter()
            .map(|(id, _)| id.0)
            .max()
            .unwrap_or(0)
            .wrapping_add(1);
        while nodes.iter().any(|(id, _)| id.0 == candidate) {
            candidate = candidate.wrapping_add(1);
        }
        NodeId::new(candidate)
    }

    proptest! {
        /// Over sparse, shuffled parent ids and any script of removals,
        /// appends, and unchanged events, a current child index is a member
        /// exactly when its node was carried from a parent index in the set.
        /// The oracle tags each node with its provenance directly; the
        /// membership sees only ids. Until the first removal the parent's
        /// slices are handed out as they are.
        #[test]
        fn birth_membership_follows_provenance_not_position(
            parent_ids in prop::collection::btree_set(0u32..48, 1..10)
                .prop_map(|ids| ids.into_iter().collect::<Vec<_>>())
                .prop_shuffle(),
            reachable_flags in prop::collection::vec(any::<bool>(), 10),
            executed_flags in prop::collection::vec(any::<bool>(), 10),
            steps in prop::collection::vec(step(), 0..16),
        ) {
            let parent_len = parent_ids.len();
            let reachable = subset(&reachable_flags[..parent_len]);
            let executed = subset(&executed_flags[..parent_len]);
            let mut membership = BirthMembership::new(&reachable, &executed, parent_len);
            let mut nodes: Vec<(NodeId, Option<usize>)> = parent_ids
                .iter()
                .enumerate()
                .map(|(index, &id)| (NodeId::new(id), Some(index)))
                .collect();
            let mut removed_any = false;
            for step in steps {
                let before: Vec<NodeId> = nodes.iter().map(|(id, _)| *id).collect();
                match step {
                    Step::Remove(at) if nodes.len() > 1 => {
                        nodes.remove(at % nodes.len());
                        removed_any = true;
                    }
                    Step::Append(count) => {
                        for _ in 0..count {
                            let id = next_free_id(&nodes);
                            nodes.push((id, None));
                        }
                    }
                    Step::Remove(_) | Step::Unchanged => {}
                }
                let after: Vec<NodeGenome> = nodes.iter().map(|(id, _)| bare_node(*id)).collect();
                membership.observe_event(&before, &after);

                let sets = membership.sets();
                if !removed_any {
                    prop_assert!(std::ptr::eq(sets.reachable, reachable.as_slice()));
                    prop_assert!(std::ptr::eq(sets.executed, executed.as_slice()));
                }
                for (set, parent_set) in [(sets.reachable, &reachable), (sets.executed, &executed)] {
                    let expected: Vec<usize> = nodes
                        .iter()
                        .enumerate()
                        .filter(|(_, (_, origin))| origin.is_some_and(|p| parent_set.contains(&p)))
                        .map(|(index, _)| index)
                        .collect();
                    prop_assert_eq!(set, expected.as_slice());
                }
            }
        }
    }

    /// A parent-set entry at the parent's node count names no carried node,
    /// so the first removal drops it instead of shifting it onto a node this
    /// birth created at that index.
    #[test]
    fn birth_membership_drops_entries_at_the_parent_node_count_on_removal() {
        let reachable = vec![0, 2];
        let executed = vec![1, 2];
        let mut membership = BirthMembership::new(&reachable, &executed, 2);
        let carried = [NodeId::new(4), NodeId::new(7)];
        let created = NodeId::new(8);

        let mut after: Vec<NodeGenome> = carried.iter().copied().map(bare_node).collect();
        after.push(bare_node(created));
        membership.observe_event(&carried, &after);
        membership.observe_event(&[carried[0], carried[1], created], &after[1..]);

        let sets = membership.sets();
        assert!(sets.reachable.is_empty(), "reachable: {:?}", sets.reachable);
        assert_eq!(sets.executed, [0]);
    }

    #[test]
    fn empty_eligible_returns_none() {
        let mut rng = seeded_rng(0);
        assert!(biased_select_from(&[], &[0, 1], 0.5, &mut rng).is_none());
    }

    #[test]
    fn bias_zero_selects_uniformly() {
        let eligible = vec![0, 1, 2, 3, 4];
        let reachable = vec![0, 1]; // only 0, 1 reachable
        let mut rng = seeded_rng(42);
        let mut selected_unreachable = false;
        for _ in 0..200 {
            let (idx, _) = biased_select_from(&eligible, &reachable, 0.0, &mut rng).unwrap();
            if idx >= 2 {
                selected_unreachable = true;
                break;
            }
        }
        assert!(
            selected_unreachable,
            "bias=0.0 should eventually select unreachable nodes"
        );
    }

    #[test]
    fn production_bias_gives_each_eligible_live_or_inactive_node_equal_opportunity() {
        let bias = crate::config::SimulationConfig::default()
            .mutation
            .reachable_bias;
        for value in [bias.topology, bias.vm, bias.graph, bias.input_ref] {
            let mut random = seeded_rng(42);
            let mut counts = [0u32; 5];
            for _ in 0..20_000 {
                let (index, class) =
                    biased_select_from(&[0, 1, 2, 3, 4], &[0, 1], value, &mut random).unwrap();
                counts[index] += 1;
                assert_eq!(
                    class,
                    if index < 2 {
                        TargetReachability::Reachable
                    } else {
                        TargetReachability::Unreachable
                    }
                );
            }
            // Each node expects 4,000 draws; 300 is over five standard deviations.
            // Three inactive nodes therefore receive 60%, not a class quota of 50%.
            assert!(
                counts
                    .into_iter()
                    .all(|count| (3700..=4300).contains(&count)),
                "{counts:?}"
            );
        }
    }

    #[test]
    fn bias_one_always_selects_reachable_when_available() {
        let eligible = vec![0, 1, 2, 3, 4];
        let reachable = vec![1, 3]; // only 1, 3 reachable
        let mut rng = seeded_rng(99);
        for _ in 0..500 {
            let (idx, class) = biased_select_from(&eligible, &reachable, 1.0, &mut rng).unwrap();
            assert!(
                idx == 1 || idx == 3,
                "bias=1.0 must select reachable, got {idx}"
            );
            assert_eq!(class, TargetReachability::Reachable);
        }
    }

    #[test]
    fn bias_one_falls_back_when_no_reachable_eligible() {
        let eligible = vec![2, 4, 6];
        let reachable = vec![1, 3, 5]; // no overlap
        let mut rng = seeded_rng(7);
        let (idx, class) = biased_select_from(&eligible, &reachable, 1.0, &mut rng).unwrap();
        assert!(eligible.contains(&idx));
        assert_eq!(class, TargetReachability::Unreachable);
    }

    #[test]
    fn negative_bias_one_always_selects_unreachable_when_available() {
        let eligible = vec![0, 1, 2, 3, 4];
        let reachable = vec![1, 3]; // 0, 2, 4 are unreachable
        let mut rng = seeded_rng(123);
        for _ in 0..500 {
            let (idx, class) = biased_select_from(&eligible, &reachable, -1.0, &mut rng).unwrap();
            assert!(
                idx == 0 || idx == 2 || idx == 4,
                "bias=-1.0 must select unreachable, got {idx}"
            );
            assert_eq!(class, TargetReachability::Unreachable);
        }
    }

    #[test]
    fn negative_bias_one_falls_back_when_no_unreachable_eligible() {
        let eligible = vec![1, 3, 5];
        let reachable = vec![1, 3, 5]; // all reachable
        let mut rng = seeded_rng(77);
        let (idx, class) = biased_select_from(&eligible, &reachable, -1.0, &mut rng).unwrap();
        assert!(eligible.contains(&idx));
        assert_eq!(class, TargetReachability::Reachable);
    }

    #[test]
    fn classify_reachable_node() {
        assert_eq!(
            classify_target(3, &[1, 3, 5]),
            TargetReachability::Reachable
        );
    }

    #[test]
    fn classify_unreachable_node() {
        assert_eq!(
            classify_target(4, &[1, 3, 5]),
            TargetReachability::Unreachable
        );
    }

    #[test]
    fn classify_empty_reachable() {
        assert_eq!(classify_target(0, &[]), TargetReachability::Unreachable);
    }

    #[test]
    fn statistical_bias_0_7() {
        let eligible: Vec<usize> = (0..10).collect();
        let reachable = vec![0, 1, 2, 3, 4]; // 5 of 10 reachable
        let mut rng = seeded_rng(12345);
        let n = 5000;
        let mut reachable_count = 0u32;
        for _ in 0..n {
            let (_, class) = biased_select_from(&eligible, &reachable, 0.7, &mut rng).unwrap();
            if class == TargetReachability::Reachable {
                reachable_count += 1;
            }
        }
        let rate = f64::from(reachable_count) / n as f64;
        // Expected: 0.7 * 1.0 + 0.3 * 0.5 = 0.85 (70% biased always-reachable + 30% uniform 50/50)
        assert!(
            (0.80..=0.90).contains(&rate),
            "expected reachable rate in [0.80, 0.90], got {rate:.3}"
        );
    }

    #[test]
    fn intersection_count_basic() {
        assert_eq!(intersection_count(&[1, 3, 5], &[2, 3, 5, 7]), 2);
        assert_eq!(intersection_count(&[], &[1, 2]), 0);
        assert_eq!(intersection_count(&[1, 2], &[]), 0);
        assert_eq!(intersection_count(&[1, 2, 3], &[1, 2, 3]), 3);
    }

    #[test]
    fn nth_intersection_basic() {
        assert_eq!(nth_intersection(&[1, 3, 5, 7], &[3, 5, 9], 0), 3);
        assert_eq!(nth_intersection(&[1, 3, 5, 7], &[3, 5, 9], 1), 5);
    }

    #[test]
    fn nth_difference_basic() {
        assert_eq!(nth_difference(&[1, 3, 5, 7], &[3, 5, 9], 0), 1);
        assert_eq!(nth_difference(&[1, 3, 5, 7], &[3, 5, 9], 1), 7);
    }
}
