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

- [x] Write failing tests first: the VM tail-copy property over generated
      programs, the VM copy-and-divergence fixture, the graph activation
      property for `CopyInternalNode` and `CopySubgraph` (a backward external
      edge fixture must fail on appended copies), the split-exclusion skip,
      and the mesh `CopyNode` activation fixture; commit any
      `proptest-regressions/` file.
      (`7dd8d7c3`; regressions committed for both new proptests.)
- [x] Implement tail placement with the terminal guard for the three VM copy
      operators, phase-faithful insertion and self-edge rule for the two graph
      copy operators, and the split exclusion; remove the documented exception
      from the graph tests. (`3672367a`.)
- [x] Update `v3-mutation-spec.md` (VM copy placement and taxonomy: the three
      VM copies join the growth class, the remapped copy stays a macro; graph
      copy placement and self-edge rule; split exclusion replacing the
      documented exception; F08 ownership text), the graph backend spec's T11
      pointer, and the mesh execution spec's copy interference note.
      (`8ca76bb0`.)
- [x] Self-review the diff (`simplify`), then fresh `make rust-mutants`;
      record the survivor triage below. (`005b0778`, `eebacbbf`.)
- [x] Store gate and single goal reports at
      `docs/progress/features/t11-f08-function-preserving-duplication-and-module-growth.json`
      and `...-goal.json`; append both to `docs/progress/benchmark-series.json`;
      add the `docs/progress.md` row.

## Verification

- [x] TDD evidence: record the initial failing commands and results for each
      test above, then the passing commands.

  Red, at `7dd8d7c3` (tests plus the extracted seams, placement and exclusion
  semantics unchanged):

  - `cargo test -p v3-core --lib mutation::vm::f08_tests` — **FAILED**, 1
    passed / 4 failed: `tail_copy_is_neutral_under_ample_budget`
    (`[NoOp, NoOp]` vs `[NoOp]`, minimal input `len = 1, op_index = 0`),
    `tail_copy_preserves_the_program_prefix_and_guards_a_non_terminal_program`,
    `tail_copy_authors_no_guard_when_the_program_already_ends_in_a_terminal`
    (copy landed at index 1, not the tail),
    `the_guard_stops_fall_through_into_the_copied_span` (no guard authored,
    `[PushAction, PushAction]`).
  - `cargo test -p v3-core --lib mutation::graph::tests::f08` — **FAILED**, 2
    passed / 6 failed: `activating_a_copy_read_by_a_backward_edge_reproduces_the_original`,
    `copy_activation_is_equivalent_over_fixtures_and_seeds` (minimal input
    `seed = 13425958788970458832`, `activate[base][0, 1]` actions `Move(N)` vs
    `Move(NE)`), `copy_internal_node_inserts_the_copy_directly_after_its_source`,
    `copy_internal_node_shifts_the_copied_inputs_and_follows_a_self_edge`,
    `copy_subgraph_inserts_every_copy_directly_after_its_member`,
    `split_skips_a_live_introspection_edge_into_a_non_compute_consumer_under_plasticity`.
  - `cargo test -p v3-core --lib mutation::topology::f08_tests` — **passed on
    the first run**, and truthfully so: this feature changes no topology code,
    and T11.F15's `CopyNode` already produces a clone that reproduces its
    original once `SwapRouteTargets` runs it in the original's chain position.
    Both fixtures are qualification, not red-then-green.
  - Two `proptest-regressions` files were created by those failures and are
    committed: `crates/v3-core/proptest-regressions/mutation/vm/f08_tests.txt`
    and `.../mutation/graph/tests/f08.txt`.

  Green, at `3672367a` and after: `cargo test -p v3-core --lib mutation::` —
  **ok, 339 passed / 0 failed** (one pre-existing assertion updated for the
  new semantics: `copy_instruction_block_respects_max_32`'s bound is 7, not 6,
  because a three-`Noop` program now also gains the guard `Halt`).
  `cargo test -p v3-core --lib` — **ok, 1197 passed / 0 failed / 1 ignored**.

- [x] Property tests: VM tail-copy neutrality over generated programs and any
      offsets; graph copy activation equivalence over fixtures and seeds; the
      split property unconditional with the skip asserted for excluded edges.

  `tail_copy_is_neutral_under_ample_budget` (96 cases) draws programs from the
  production `random_vm_instruction` generator, optionally redrawing every jump
  offset across the whole `i32` range, applies one of the three copy operators
  through `VmMutator::apply`, and compares `NodeResult`, action queue, output
  slots, and shared memory; the ample-budget precondition is
  `!energy_exhausted && steps + 1 < max_vm_steps`.
  `copy_activation_is_equivalent_over_fixtures_and_seeds` (48 cases) runs over
  `base_def`, `plasticity_def`, and a new `phase_def`, duplicating one drawn
  node and one drawn ascending cluster, asserting fire-time neutrality and then
  activation equivalence across a four-tick sequence per scenario from fresh
  state with `GraphRuntimeState::begin_tick` at each boundary.
  `split_existing_edge_is_neutral_at_fire_time` now holds with no exception
  branch, and `split_skips_...` / `split_exclusion_is_scoped_to_all_three_of_its_conditions`
  pin the skip to exactly the excluded edges.

- [x] `cargo test -p v3-core --test viability` ran after the mutation
      semantics changed; result recorded: **ok, 25 passed / 0 failed**, run
      twice (after the semantics change and again after the simplify pass).
      Deviation from the brief, recorded rather than glossed: the actual order
      was the focused `mutation::` suite (which surfaced two test updates),
      then the topology fixtures, then viability, then the full lib suite and
      `make check`. Viability ran before the full suite and `make check` but
      not before the focused suite, so it was not literally first.
- [x] `make rust-mutants` (fresh, `MUTANTS_ITERATE=0`): summary line, output
      path, and every survivor resolved as killed, equivalent, or deferred.

  First fresh run (`MUTANTS_ITERATE=0 make rust-mutants`, mode `fresh`, diff
  against `de35c058`), output
  `/Users/istefanek/.local/share/petri-tools/mutants/t11-f08/mutants.out`:

  > `41 mutants tested in 5m: 1 missed, 40 caught`

  Survivor list (`missed.txt`), one entry:

  - `crates/v3-core/src/mutation/graph/operators.rs:572:40: replace > with >= in check_compute_node_capacity`
    — **killed** (`eebacbbf`): `copy_operators_skip_when_the_copy_would_exceed_the_index_space`
    now starts one node below the limit and asserts the copy still applies at
    the highest non-sentinel index before asserting the skip at the limit. No
    production code changed.

  Second fresh run after that test, same command and output path:

  > `41 mutants tested in 4m: 41 caught`

  and `rust-mutants: no survivors`.

- [ ] `make bench PROFILE=gate FEATURE=t11-f08-function-preserving-duplication-and-module-growth`
      and `make bench PROFILE=goal FEATURE=...` each exited 0 with reports
      stored as above; neighborhood readings recorded in Performance below.

  Gate: **exit 0**, `severe=false` against both references, report stored.
  Goal: the run completed and the report was written, but `compare_against`
  reported `severe=true` for `plasticity_updates` versus the previous closure
  and `make bench` therefore **exited 3**. The crossing, its cause, and the
  epoch comparison are recorded in Performance below; no threshold was
  weakened and no stored baseline was edited. Accepting or rejecting the
  crossing is the orchestrator's and reviewer's decision, so this item stays
  unchecked.

- [x] Second goal-profile determinism run: Not applicable per the 2026-09-05
      workflow decision; `crates/v3-core/tests/reproducibility.rs` in
      `make check` covers cross-process reproducibility.
- [x] `make roadmap-check` on document edits and independently by the
      orchestrator; `make check` exited 0 at the tested commit.

  `make roadmap-check` — `roadmap-check: validation passed`.
  `make check` — **exit 0** at `3672367a` and again after the simplify pass at
  `005b0778`.

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

Reports: [gate](../../progress/features/t11-f08-function-preserving-duplication-and-module-growth.json),
[goal](../../progress/features/t11-f08-function-preserving-duplication-and-module-growth-goal.json).
The tables below name the readings the predeclaration asked for; the reports
are the source for everything else.

**Gate counters** (per creature-tick; previous T11.F15 / epoch T11.F04),
`severe=false` against both:

| counter | current | vs T11.F15 | vs T11.F04 |
| --- | --- | --- | --- |
| `mesh_hops` | 2.000000 | 0.000000% | +0.022955% |
| `vm_steps` | 28.044355 | 0.000000% | -0.001248% |
| `graph_relax_iters` | 1.000000 | 0.000000% | -66.650826% (T11.F06 definition change) |
| `plasticity_updates` | 0.000899 | 0.000000% | -0.221976% |
| `actions_applied` | 1.000000 | 0.000000% | 0.000000% |
| `births` | 0.001225 | 0.000000% | +5.331040% |

Gate wall 0.004238436 ms/creature-tick: +1.791398% versus T11.F15,
+0.929275% versus T11.F04, both `ok`. Every gate counter is byte-identical to
T11.F15: the gate profile is 225 ticks with 75 births, and a dormant tail copy
costs no VM step and changes no action, so the simulation trajectory it
produces is the same one.

**Goal counters** (per creature-tick), `severe=true` for one counter versus
the previous closure:

| counter | current | vs T11.F15 | vs T11.F04 (epoch) |
| --- | --- | --- | --- |
| `mesh_hops` | 2.181179 | +0.034718% | -33.515052% |
| `vm_steps` | 41.169837 | -32.044401% | -96.673252% |
| `graph_relax_iters` | 1.005334 | +0.073462% | -83.251507% |
| `plasticity_updates` | 0.104370 | **+63.738195% (severe)** | -25.448402% |
| `actions_applied` | 1.020685 | -6.190920% | -9.637509% |
| `births` | 0.007635 | -3.464408% | -6.330512% |

Goal wall 0.006062250 ms/creature-tick: **-6.353186%** versus T11.F15 and
**-15.275737%** versus T11.F04, both `ok`; total world wall time fell from
636,657 ms to 603,463 ms over 1.2% more creature-ticks.

**The `plasticity_updates` crossing.** Cause, stated as measured rather than
excused.

*Series context.* This counter has ranged 0.100–0.154 per creature-tick across
every goal report from T01.F12 through T11.F14 (0.100492, 0.100492, 0.124060,
0.115330, 0.139997, 0.153630, 0.151239, 0.151239); T11.F15's 0.063742 is the
one low outlier in the series, and this feature's 0.104370 is back inside the
historical band and 25.4% *below* the T11.F04 epoch.

*Measured composition change, not an inference.* `graph_relax_iters` is flat
(+0.073462%), so the rise is updates per graph visit, and the evolved
neighborhood shows why: every plasticity-only operator doubled its applied
trials against T11.F15 — `DisableHebbian` 40/40 applied with 680 skipped →
80/80 with 640, and the same 40 → 80 shift for `EnableRewardModulation`,
`MutateHebbianRate`, `MutateHebbianRule`, and `ToggleHebbianLamarckian`. At 20
trials per genome that is 2 of 36 sampled evolved genomes carrying plasticity
under T11.F15 versus 4 under this feature. Mean reachable structure also rose
(128.824696 versus 125.564576, p75 157 versus 153) while VM work fell 32%.

*Direct mechanism: the feature working.* Before this change a copied plasticity
node was appended, so it read its inputs in the wrong evaluation phase; a
mutation that wired something to it produced a different signal, and selection
removed it. The copy is now phase-faithful, so a duplicated plasticity module
reproduces its original when something reads it and can then diverge —
retained duplicated plasticity structure is exactly what this feature's Goal
asks for ("complexity can grow by copy and divergence"), and more retained
plasticity structure means more Hebbian updates per visit. Nothing in the diff
adds an update to a fixed genome: the split exclusion in fact *removes* split
opportunities on plasticity-carrying graphs.

*Status.* The spec predeclared that populations differ and every counter may
move, but did not predeclare a severe allowance, so this is an unpredeclared
cost of the intended effect. The run is deterministic, so re-running produces
the same flag; the decision is acceptance or rejection of the observed cost,
as with T11.F04, not remediation. No threshold was weakened and no baseline
was re-pinned; an epoch re-pin is not recommended, since the counter sits 25%
below the T11.F04 epoch and inside the historical band.

**Founder neighborhood versus the predeclaration** (gate report, silent /
applied, T11.F15 → T11.F08):

| operator | predeclared | T11.F15 | T11.F08 | met |
| --- | --- | --- | --- | --- |
| `VmCopyInstructionBlock` | 19/50 → 50/50 | 19/50 | 50/50 | yes |
| `VmCopyGeneBackwardSlice` | 27/50, 6 dead → 50/50, 0 dead | 27/50, 6 dead | 50/50, 0 dead | yes |
| `VmCopyGeneForwardSlice` | 22/50 → 50/50 | 22/50 | 50/50 | yes |
| `VmCopyInstructionBlockRemapped` | unchanged | 15/50 | 15/50 | yes |
| `CopyInternalNode` | stays 50/50 | 50/50 | 50/50 | yes |
| `CopySubgraph` | stays 50/50 | 50/50 | 50/50 | yes |
| `AddInternalGraphNode` | 50/50 or skips only on an excluded edge | 50/50, 0 skips | 50/50, 0 skips | yes |

Every other founder operator row is byte-identical to T11.F15. Single-event
silent births 84/164 (predeclared floor: not below 80/164; T11.F15 read
80/164). Founder dead births 0/208, unchanged. Founder observation 63.415 ms,
far under the 10-second cap.

**Evolved neighborhood** (goal report, pooled over 36 genomes, T11.F15 →
T11.F08): `VmCopyInstructionBlock` 483/720 with 2 dead → **720/720, 0 dead**;
`VmCopyGeneBackwardSlice` 370/610 with 24 dead → **604/604, 0 dead**;
`VmCopyGeneForwardSlice` 402/608 → **622/622, 0 dead** — silent on every
applied trial, as predeclared. `VmCopyInstructionBlockRemapped` stays a
behavior-changing macro (446/720 → 449/720). `CopyInternalNode` 689/689,
`CopySubgraph` 680/680, `CopyNode` 720/720; the two mesh slices rose from
717/720 and 718/720 to 720/720. `AddInternalGraphNode` reads 701/701 silent
with 19 skips (T11.F15: 707/707 with 13 skips) — the six extra skips are the
split exclusion declining edges it must decline; the silent fraction stays
1.00. Evolved pooled dead births **19/3300** (predeclared: not above
21/3300); evolved single-event silence 1862/2736 versus 1823/2736. Evolved
observation 632.437 ms total (200.6 / 243.9 / 187.9 ms per seed), far under
the 180-second cap; whole-run goal wall 603,463 ms.

**Persistence and lineage versus T11.F15** (no cognition claim). Final
populations 12,298 / 11,704 / 11,405 (T11.F15: 10,786 / 11,669 / 11,714), no
extinction on any seed, plateau 12,431.702 / 12,398.210 / 12,402.022 versus
10,596.510 / 12,308.016 / 12,191.246, mean energy 67.121 / 66.347 / 61.506
versus 64.769 / 61.161 / 60.371. Births per 100 ticks 12,667.366667 versus
12,963.483333. Reachable structure min/p25/median/p75/max/mean
1/97/107/157/519/128.824696 versus 3/97/104/153/723/125.564576. Lineage
clades/entropy 208/4.329333, 189/4.112589, 194/4.326013 versus 173/4.038022,
212/4.284456, 225/4.251236. Current-memory either counts 2/2/0 versus 3/2/0;
temporal operator-state either counts 79/24/120 versus 37/11/17, persisted
outputs 7/10/16 versus 8/13/25, previous slots 0/3/0 versus 0/0/0. Generation
median/max 21/45, 23/42, 21/46 versus 23/45, 21/52, 22/44. These are one run
per side on unpaired populations; nothing here is a claim about cognition.

## Success Criteria

- [x] The three VM copy operators, the two graph copy operators, and the mesh
      copy are each shown neutral when they fire and equivalent when
      activated unchanged, by property tests or fixtures on every backend.
      (`tail_copy_is_neutral_under_ample_budget`,
      `copy_activation_is_equivalent_over_fixtures_and_seeds`,
      `activating_a_copy_read_by_a_backward_edge_reproduces_the_original`,
      `copy_node_clone_shares_its_original_addresses_slots_and_references`.)
- [x] Each backend's copy-and-divergence trajectory (silent copy, silent
      dormant divergence, activation) is a passing fixture.
      (`vm_copy_diverge_and_activate_trajectory`,
      `mesh_copy_diverge_and_activate_trajectory`; the graph half is the
      activation property above, whose dormant-divergence step is the copy's
      own independent state and wiring.)
- [ ] The split exclusion replaces the documented T11.F03 exception, the
      references record the changed placement and interference rules, and the
      stored reports meet the predeclared neighborhood expectations or record
      why not. (Exclusion and reference updates landed; every predeclared
      neighborhood expectation is met; the goal profile's
      `plasticity_updates` crossing is recorded with its cause above and is
      the one open decision.)

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
- Implementation seams, for the reviewer and for T11.F09:
  `CgpGraphBackendDef::duplicate_compute_nodes_in_place` is the single
  placement primitive both graph copy operators use, and
  `copy_span_to_dormant_tail` is the single placement helper the three VM
  copy operators use. Learned-weight correspondence through either is
  T11.F09's: the duplication primitive shifts genome indices without touching
  `GraphRuntimeState::plasticity_weights`, exactly as T11.F03's split does.
- Deferred findings: none from the mutation run (no survivors on the second
  fresh run). One measurement item is open rather than deferred: the goal
  profile's `plasticity_updates` severe flag versus T11.F15, recorded with
  its cause in Performance and Goal Impact. `make bench PROFILE=goal` exited
  3 because of it; the report itself was written and stored.
- Judgment call beyond the literal brief, flagged for the reviewer: the
  reference update also places `VmCopyConstantBlock` in the growth class of
  the `v3-mutation-spec.md` taxonomy. It is factually silent when it fires
  (founder 50/50, evolved 617/617) and append-only, but the brief named only
  the three tail copies, so strike it if unwanted. Two crate-internal
  signature changes support the tests: `mutate_one_instruction_field` widened
  from `pub(super)` to `pub(crate)` so the mesh fixture can apply the
  production single-field step to a clone, and `split_existing_edge` gained an
  `input_refs` parameter (all call sites updated) because the exclusion has to
  resolve an `InputLeaf` to its reference kind.
- The mesh half of this feature is qualification only, as the spec's
  Non-Goals require: `crates/v3-core/src/mutation/topology/f08_tests.rs`
  passed on its first run against unchanged T11.F15 topology code, and no
  topology file is in the diff.
