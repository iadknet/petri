# T11.F08 — Function-Preserving Duplication and Module Growth

**Status**: In Progress
**Last updated**: 2026-09-06
**Feature**: T11.F08
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Gene duplication. Copying a working module leaves the brain acting exactly as
its parent did when the copy is made, and the copy still works when a later
mutation activates it: a VM block or slice is duplicated into a span that
control flow cannot reach until a jump points at it, a graph node or subgraph
is duplicated at the position that keeps every input on the same world-tick
phase as its original, and a mesh copy reproduces its original when it is
selected in the original's place. A short trajectory on each backend shows
copy, silent divergence of the dormant copy, and activation, so complexity
can grow by copy and divergence.

## Non-Goals

- No label addressing (T11.F11), learned-weight correspondence or inheritance
  (T11.F09), memory-motif evolvability measurement (T11.F10), mutation
  probability, weight, supply, or reachability-bias change (T11.F04), and no
  new operator, node kind, sensor, or metabolic charge.
- No mesh routing or attachment change: T11.F15's `CopyNode`, slice
  attachment, `SwapNodeBackend`, and single-visit rule are unchanged.
- No claim that a copy is neutral in energy or steps, or at exhaustion: a
  guard `Halt` reached by fall-through costs its step, and a dormant graph
  copy still costs `graph_node_base_cost` per visit. Neutrality is behavioral
  and is proven by execution, never by reachability analysis alone.
- No change to `VmCopyInstructionBlockRemapped` (the inline register-renamed
  copy stays an ordinary behavior-changing macro and is measured as such),
  `VmCopyConstantBlock` (already append-only and silent), or `CopyEdgeBundle`.
- No founder change, benchmark threshold change, baseline re-pin, or second
  goal run. The T11.F03 spare-register `ReadInput` alternative for input
  references stays deferred: no reading yet shows references never go live.

## Inputs and Invariants

- Sources: the track's T11.F08 note ("uses the T11.F01 neighborhood and the
  corrected VM and graph semantics to qualify the existing copy operators")
  and node-type contract; [T11.F02](t11-f02-vm-structural-mutation-semantics.md)
  (copy-target rule, proven unreachable suffix, neutrality wording);
  [T11.F03](t11-f03-function-preserving-graph-growth.md) (faithful copy,
  insert-with-remap, deferred P2-2 split exception); [T11.F06](t11-f06-graph-memory-clock.md)
  (lower-index sources read the current visit, self and higher-index sources
  read frozen tick-start outputs); [T11.F15](t11-f15-mesh-routing-connection-semantics.md)
  (dormant paralog attachment, "F08 qualifies post-activation properties");
  the [mutation](../../reference/v3-mutation-spec.md),
  [graph backend](../../reference/v3-graph-backend-spec.md),
  [mesh execution](../../reference/v3-mesh-execution-spec.md), and
  [VM ISA](../../reference/v3-vm-isa-spec.md) references.
- Seams: `mutation/vm/operators.rs` (`apply_copy_instruction_block`,
  `apply_copy_gene_backward_slice`, `apply_copy_gene_forward_slice`,
  `splice_program_with_reference_repair`, `mutate_one_instruction_field`,
  `insert_new_instruction_with_reference_repair`), `mutation/graph/operators.rs`
  (`copy_compute_node`, `copy_cgp_subgraph`, `split_existing_edge`),
  `CgpGraphBackendDef::insert_compute_node_at`, `runtime/vm.rs` (pc past the
  end halts cleanly; `Halt` and `ExecuteActionQueue` are the terminals),
  `runtime/tests/vm_execution.rs` (`copied_unreachable_suffix_preserves_behavior_under_ample_budget`),
  `mutation/graph/tests/operators.rs` (`assert_neutral`, `run_scenarios`,
  `is_documented_split_exception`), `mutation/topology/structural.rs`
  (`copy_attached`), and the `neighborhood` battery. Extend these with `std`
  and existing crates; add no alternate mutation or execution path.
- Research basis, from the repository's recorded readings (audit of
  2026-09-04, mesh note of 2026-09-06): Koza's subroutine duplication yields
  "an offspring semantically equivalent to its parent"; Calabretta et al.
  2000 found duplication-derived modules specialize; the 2021 genotype-
  phenotype-map study found duplication raises robustness first and
  evolvability after diversification; Markov brains and SignalGP duplicate
  freely because position carries no meaning, while Brameier and Banzhaf's
  linear GP relies on introns (unreachable code) for neutral variation.
  Options: (1) place VM copies in an unreachable span and keep graph copies
  phase-faithful (selected: the track's stated route; smallest change; keeps
  ISA, founder, frontend); (2) label-addressed callable modules (T11.F11,
  conditional on the indicator after this feature); (3) leave copies inline
  and rely on supply (rejected: the F15 evolved reading is 483/720, 370/610,
  and 402/608 silent with 26 dead for the three VM copies, and inline copies
  are macro edits, not small steps). No new dependency is needed.
- Current readings (T11.F15 gate report, founder, silent/applied):
  `VmCopyInstructionBlock` 19/50, `VmCopyInstructionBlockRemapped` 15/50,
  `VmCopyGeneBackwardSlice` 27/50 with 6 dead, `VmCopyGeneForwardSlice`
  22/50, `CopyInternalNode` 50/50, `CopySubgraph` 50/50, topology `CopyNode`
  and both mesh slices 50/50. Evolved (goal report, pooled over 36 genomes):
  `CopyInternalNode` and `CopySubgraph` 689/689, `CopyNode` 720/720, slices
  717/720 and 718/720.

**VM dormant duplication.** `VmCopyInstructionBlock`,
`VmCopyGeneBackwardSlice`, and `VmCopyGeneForwardSlice` keep their source
draws (block size 2..=32 clamped to the program; the existing slice analyses)
and place the copy after the last terminal: the copy is spliced at the
program tail, and when the program's final instruction is neither `Halt` nor
`ExecuteActionQueue` a newly authored `Halt` guard (no source index) is
spliced immediately before the copied span in the same event. Every old jump
keeps its old resolved target through T11.F02 repair, so no surviving
instruction reaches the span; the guard makes fall-through halt where the old
program halted by running past its end. Copied jumps follow T11.F02's rule (a
copied selected target, else the surviving original). A later jump mutation
(a jump offset stepped by one unit, or an inserted or replaced jump) is the
only way the span becomes reachable. Property, over programs drawn by the
existing VM instruction generator with any offsets: when the original run
neither reaches the step cap nor exhausts energy (ample budget), the
`NodeResult`, action queue, output slots, and shared memory are identical
before and after the copy. Executing the guard costs one `Halt` step; this
is recorded, not hidden.

**VM copy-and-divergence trajectory.** One fixture on a small program ending
in a terminal: (1) copy the whole program to the tail, signature identical on
the battery scenarios; (2) apply the production single-field step to
instructions inside the dormant span, signature still identical; (3) insert a
newly authored `Jump` at pc 0 targeting the first copied instruction with
reference repair: for the exact copy the signature equals the original's
(the copy works in the original's place), and for the diverged copy it
differs on at least one scenario. Energy is compared only where the spec
predicts equality.

**Graph phase-faithful copies.** `CopyInternalNode` inserts the copy at
`source + 1` through `insert_compute_node_at`, applying the same index shift
to the copy's own inputs before insertion, and a self-edge on the copy reads
the copy. `CopySubgraph` inserts each cluster member's copy immediately after
that member (final index `c_i + i + 1` for the `i`-th sorted member), remaps
intra-cluster edges to the copies, and keeps external sources at their final
shifted indices (computed before any insertion). Then every copied edge
reads the same phase as its original: a source below the original stays
below the copy, a source above
stays above, and the copy's own state is its own. Copies remain read by
nothing at fire time (T11.F03 neutrality holds). Property, over the existing
fixtures and seeds: activating a copy by retargeting every edge that read a
cluster member from outside the cluster to read that member's copy, on all
five surfaces, produces identical outputs, actions, and shared memory to the
unmodified graph over a multi-tick sequence from fresh state. Index shifts
change learned-weight positions exactly as T11.F03's split does; the
correspondence rule is T11.F09's.

**Split exclusion (T11.F03 deferred P2-2).** `split_existing_edge` skips with
`NoApplicableTarget` when the graph carries plasticity, the picked edge's
consumer is a sink, action slot, or execute gate, and its source is an
`InputLeaf` that resolves to a `DynamicIntrospection` reference, because
that value can differ between evaluation and the post-plasticity effects
context; this is exactly the documented exception's scope. The documented exception and
`is_documented_split_exception` are removed; the split property holds
unconditionally, with the skip asserted for exactly those edges.

**Mesh post-activation qualification.** No topology code changes. A
`CopyNode` clone shares its original's shared-memory addresses, output slots,
and input references; activating it with `SwapRouteTargets` runs it in the
original's chain position, and from fresh runtime state its behavior equals
the original's on the battery. One fixture shows silent copy, silent
divergence of the dormant clone by one backend step, activation of the exact
clone identical on single-tick and sequence scenarios, and activation of the
diverged clone changed. Interference is accounted for, not remapped: when a
slice copy places an original and its copy in one chain, both write the same
addresses and the later node's write wins (mesh execution spec, chain
evaluation notes); record this rule beside the copy semantics.

**Neighborhood predeclaration.** Founder rows expected to move, against the
T11.F15 gate report: `VmCopyInstructionBlock` 19/50 → 50/50 silent,
`VmCopyGeneBackwardSlice` 27/50 (6 dead) → 50/50 silent and 0 dead,
`VmCopyGeneForwardSlice` 22/50 → 50/50 silent; `VmCopyInstructionBlockRemapped`
unchanged (same operator, same seeds); `CopyInternalNode` and `CopySubgraph`
stay 50/50; `AddInternalGraphNode` stays 50/50 or shows skips only where the
founder carries an excluded split edge. Single-event silent births should not
fall below 80/164 and founder dead births stay 0/208; evolved pooled dead
births should not rise above 21/3300. Evolved rows for the three converted VM
operators should read silent on every applied trial. No operator family is
disabled or down-weighted; the remapped inline copy keeps its weight.

## Implementation Tasks

- [ ] Write failing tests first: the VM tail-copy property over generated
      programs, the VM copy-and-divergence fixture, the graph activation
      property for `CopyInternalNode` and `CopySubgraph` (a backward external
      edge fixture must fail on appended copies), the split-exclusion skip,
      and the mesh `CopyNode` activation fixture; commit any
      `proptest-regressions/` file.
- [ ] Implement tail placement with the terminal guard for the three VM copy
      operators, phase-faithful insertion and self-edge rule for the two graph
      copy operators, and the split exclusion; remove the documented exception
      from the graph tests.
- [ ] Update `v3-mutation-spec.md` (VM copy placement and taxonomy: the three
      VM copies join the growth class, the remapped copy stays a macro; graph
      copy placement and self-edge rule; split exclusion replacing the
      documented exception; F08 ownership text), the graph backend spec's T11
      pointer, and the mesh execution spec's copy interference note.
- [ ] Self-review the diff (`simplify`), then fresh `make rust-mutants`;
      record the survivor triage below.
- [ ] Store gate and single goal reports at
      `docs/progress/features/t11-f08-function-preserving-duplication-and-module-growth.json`
      and `...-goal.json`; append both to `docs/progress/benchmark-series.json`;
      add the `docs/progress.md` row.

## Verification

- [ ] TDD evidence: record the initial failing commands and results for each
      test above, then the passing commands.
- [ ] Property tests: VM tail-copy neutrality over generated programs and any
      offsets; graph copy activation equivalence over fixtures and seeds; the
      split property unconditional with the skip asserted for excluded edges.
- [ ] `cargo test -p v3-core --test viability` ran first after the mutation
      semantics changed; result recorded.
- [ ] `make rust-mutants` (fresh, `MUTANTS_ITERATE=0`): summary line, output
      path, and every survivor resolved as killed, equivalent, or deferred.
- [ ] `make bench PROFILE=gate FEATURE=t11-f08-function-preserving-duplication-and-module-growth`
      and `make bench PROFILE=goal FEATURE=...` each exited 0 with reports
      stored as above; neighborhood readings recorded in Performance below.
- [ ] Second goal-profile determinism run: Not applicable per the 2026-09-05
      workflow decision; `crates/v3-core/tests/reproducibility.rs` in
      `make check` covers cross-process reproducibility.
- [ ] `make roadmap-check` on document edits and independently by the
      orchestrator; `make check` exited 0 at the tested commit.

## Performance and Goal Impact

Natural analog: gene duplication. A duplicated gene is carried silently until
regulation or divergence gives it a role; it reaches creatures only through
their inherited brain, with no feature-specific sensor or reward.

Predeclared cost: no work-counter definition change, no severe compute
allowance, and no epoch re-pin. Dormant VM tails cost no steps but grow
genomes, so memory, clone, and serialization work per birth may rise; graph
copies keep their per-visit node cost; populations differ, so every counter
may move. Compare all six gate counters and wall time per creature-tick
against previous T11.F15 and epoch T11.F04, and the goal counters likewise;
keep the +10%/+50% work flags and +25%/+100% wall flags. Founder observation
stays under 10 seconds and summed evolved observation under 180 seconds per
goal run. Record the founder and evolved neighborhood rows against the
predeclaration above, the persistence and lineage readings against T11.F15
without a cognition claim, and any threshold crossing with its cause.

## Success Criteria

- [ ] The three VM copy operators, the two graph copy operators, and the mesh
      copy are each shown neutral when they fire and equivalent when
      activated unchanged, by property tests or fixtures on every backend.
- [ ] Each backend's copy-and-divergence trajectory (silent copy, silent
      dormant divergence, activation) is a passing fixture.
- [ ] The split exclusion replaces the documented T11.F03 exception, the
      references record the changed placement and interference rules, and the
      stored reports meet the predeclared neighborhood expectations or record
      why not.

## Notes for AI Agents

- Planning base: main `de35c058` (T11.F15 closure), worktree
  `.claude/worktrees/t11-f08`, branch `worktree-t11-f08`. Roles per
  `docs/workflow.md`: Fable 5.1 orchestrator, Opus 5 implementer with Fable
  advisor, fresh Fable reviewer.
- Readiness review (orchestrator, 2026-09-06): checked against the template,
  the track's T11.F08 note, the node-type contract, the T11.F02/F03/F06/F15
  invariants, and the VM and graph runtime rules. One revision: the graph
  copy's own inputs take the same index shift before insertion, the split
  exclusion is scoped to graphs carrying plasticity, and the VM property's
  ample-budget condition is stated. Ready.
- At closure, evaluate track criterion 2 (structural VM edits preserve
  references, verified by property tests; one field per operand event)
  against T11.F02's and this feature's property tests and check it only if
  satisfied; criterion 6 stays open until T11.F09 lands learned-weight
  correspondence.
