# T10.F11 — Cross-Process Reproducibility of Seeded Runs

**Status**: In Progress
**Last updated**: 2026-09-04
**Feature**: T10.F11
**Track**: [T10 — Evolutionary Scale and Experiment Infrastructure](../../roadmaps/t10-evolutionary-scale-and-experiment-infrastructure.md)

## Goal

A seeded run reproduces its trajectory byte for byte in any process and at any
thread count. The two mutation operators that index a seeded draw into a list
built from a std `HashMap` are corrected so the candidate order is a function
of genome content only; every other hash-map iteration and RNG-indexed pick on
the simulation path is audited and recorded here; the four T01.F11 sweep
reports are regenerated at the fix commit and a second run reproduces each;
the master roadmap's persistence numbers are refreshed if they moved.

## Non-Goals

- No change to the randomness itself: every corrected operator makes the same
  number of RNG draws in the same order, over the same candidate set, and only
  the mapping from a draw to a candidate changes. No mutation rate, weight, or
  operator set changes.
- No replacement of std hash maps whose order is never observed (lookups,
  membership, `len()`, keyed counters). No global hasher swap and no new
  dependency: `BTreeMap` or a sort on the existing `Vec` is sufficient.
- No regeneration of the eight T10.F09 reports; they are wall-clock evidence
  and stay as measured, marked pre-fix in the T10 roadmap.
- No choice of a standard replicate world and no persistence gate (T01.F12);
  no checkpoint or resume (T10.F02); no throughput work.
- No change to `crates/v3-core/src/patterns/`: `generate_pattern_seeded` is
  used only by the `v3-server` HTTP pattern endpoint, not by seeding or the
  tick loop. Its hash-order dependence (`noise.rs` collects a `HashSet` into
  a `Vec` and trims it randomly) is recorded as a deferred finding below.

## Inputs and Invariants

- Contract: the T10 roadmap "Notes for AI Agents" paragraph on T10.F11 and its
  determinism-contract bullet; the master roadmap's dated baseline bullet
  ("one draw rather than a fixed fact until T10.F11 restores reproducibility
  and regenerates them").
- Dependency output (T10.F09, Complete): the finding that the `deterministic`
  block of a seeded run equals the T01.F11 report at 128² and 256² but not at
  512² (seed 11) or 1600² (all seeds), across processes, at any thread count;
  the two candidate sites it named; `v3-cli bench --threads`, whose
  in-process test `thread_count_changes_the_environment_but_not_the_deterministic_block`
  already proves thread-count independence at a tiny sweep.
- RNG sources on the simulation path: `SmallRng::seed_from_u64` in
  `simulation/seeding.rs`, `simulation/simulation.rs`, and the per-tick
  `reproduce_rng` in `simulation/tick.rs`. No `thread_rng`, `from_entropy`,
  `SystemTime`, or `OsRng` in non-test `v3-core` code; `Instant` is used only
  by the T10.F09 phase timers, which feed the `environment` block. The one
  `par_iter_mut` (Phase 1b) maps into a `Vec` in input order and performs no
  parallel reduction.
- Hash-order inventory, taken 2026-09-04 from every non-test `HashMap` and
  `HashSet` in `crates/v3-core/src` (the implementer re-runs the grep and
  records any addition in Verification):
  - Order-dependent and RNG-indexed (the defect):
    `mutation/vm/operators.rs` `apply_mutate_paired_slot_address` builds
    `groups: HashMap<u8, …>`, collects it into `eligible`, and indexes
    `eligible` with `rng.gen_range`; `mutation/topology/structural.rs`
    `clone_and_remap_slice` collects `id_map.values()` into `new_ids` and
    indexes it with `rng.gen_range` for the backlink target.
  - Lookup or membership only, order never observed: `simulation/tick.rs`
    `creature_refs` (removed by id, work order comes from `inputs`) and
    `successful_spawn_targets` (`insert`/`contains`);
    `mutation/graph/operators.rs` `old_to_new` (`get`);
    `creature/genome/analysis.rs` `node_id_to_idx` (`get`);
    `creature/genome/cgp_analysis.rs` `live_set` (`contains`);
    `kernel/paint.rs` `seen` (dedup; output order comes from the points loop);
    `creature/parseability.rs` `seen`.
  - Order-independent aggregation: `analysis.rs` `live`, `live_consts`,
    `consumed_refs` and `cgp_analysis.rs` `collect_consumed_input_refs`
    contribute only `len()` to a score; the keyed counters in
    `simulation/stats.rs` and `mutation/types/mod.rs` are read by key
    (`classify_mutation_outcome` uses `get`/`entry`) and serialize through
    `serde_json`'s sorted map. `creature/state.rs`
    `compute_live_vm_world_inputs` already uses a `BTreeMap`.
- RNG-indexed picks: fifteen `rng.gen_range(0..<list>.len())` sites in
  non-test `v3-core` code (`analysis.rs` 2, `graph/hebbian.rs` 2,
  `graph/operators.rs` 4, `reachability.rs` 1, `topology/birth.rs` 1,
  `topology/structural.rs` 2, `vm/operators.rs` 3). Each list must be built
  by iterating a `Vec`, slice, range, or ordered map. The implementer audits
  all fifteen and records the table in Verification; the two above are the
  only expected failures.
- Fix rule: a candidate list's order is a pure function of genome content.
  `eligible` is ordered by ascending slot index (a `BTreeMap<u8, …>` or a
  `sort_unstable_by_key` on the existing `Vec`); `new_ids` follows
  `gene_indices` order, which is also ascending `NodeId` because ids are
  assigned sequentially in that order. Draw count per call is unchanged.
- Why unit tests can catch this in one process: std `RandomState::new()`
  derives its keys from a thread-local pair that is incremented per map, so
  two maps built in one thread iterate the same keys in different orders.
- Invariants: telemetry derives from applied behavior; production code is
  never edited to kill a mutant; shell automation is POSIX `sh`; the
  `deterministic` block schema is unchanged; no stored baseline or threshold
  is edited to make a comparison pass. The epoch baseline is re-pinned only by
  the rule in Performance and Goal Impact.

## Implementation Tasks

- [x] Run `cargo test -p v3-core --test viability` first (mutation operators
      are birth-path mechanics), then TDD the two operators. In
      `mutation/vm/tests.rs` add `vm_mutate_paired_slot_address_is_reproducible_for_a_seed`:
      a VM program with at least four slot groups that each have a load and a
      store; apply the operator sixteen times to fresh clones of the genome,
      each with `SmallRng::seed_from_u64(same seed)`, and assert every result
      equals the first. In `mutation/topology/tests.rs` add the same shape
      for `apply_copy_mesh_backward_slice` (or forward) on a genome whose
      slice has at least four nodes, asserting the appended nodes and the
      backlink target are identical across the sixteen applications. Both
      tests must fail before the fix and pass after it; record the pre-fix
      failure in Verification. Then apply the fix rule to both sites.
- [x] Add a proptest in `mutation/vm/tests.rs`: for any program generated
      from slot instructions over slots `0..16` (with at least one paired
      group) and any seed, two applications with the same seed produce equal
      programs. Commit any `proptest-regressions/` file that appears.
- [x] Add one fast in-process reproducibility test in
      `crates/v3-core/tests/` (new file `reproducibility.rs`; add a
      `rust-test-reproducibility` target to the `Makefile`, list it in
      `.PHONY` and in the `rust-test-all` prerequisites, so `make check`
      runs it) that seeds two
      `Simulation`s with the same seed and config, runs both for the same
      horizon, and asserts (a) the two populations are byte-identical (compare
      the `serde_json` serialization, or `PartialEq`, of every creature's
      genome, position, and energy in `SlotMap` order plus the `SimStats`
      work counters) and (b) `mutation_events_applied_total_by_operator`
      shows `VmMutatePairedSlotAddress` and at least one of
      `TopologyCopyMeshBackwardSlice` / `TopologyCopyMeshForwardSlice` applied
      at least once, so the test is known to reach the corrected code. Size
      the world, founders, and horizon so the test runs in seconds in a debug
      build; the test may raise mutation rates through `MutationConfig` to
      reach the operators quickly, and records the configuration it uses in
      its doc comment. If the operators cannot be reached in seconds, record
      why in Verification and keep the test asserting (a) only; do not weaken
      (b) silently.
- [x] Commit the fix and tests as one commit; its hash is the fix commit
      every regenerated report must name in `environment.git_revision`.
      Commit `b8887e1348cfb0657b4f467f411a8d4ec0525ed0`
      ("fix(t10-f11): order the two RNG-indexed mutation candidate lists").
- [x] Regenerate the four T01.F11 reports in place with the exact T01.F11
      commands (`--feature t01-f11-baseline-persistence-characterization`,
      seeds 11,22,33, 2,000 ticks, density-matched founders 64/256/1024/10000,
      output `docs/progress/sweeps/t01-f11/wNNNN.json`), the two long ones
      detached and polled, in sequence. Then run each command a second time
      to a scratch path and compare the parsed `deterministic` objects with
      `python3` (`json.load`, whole-object equality); all four must be equal.
      Record commands, wall-clock, and the four equality results.
- [x] Refresh `docs/roadmap.md`'s dated baseline bullet: replace the sentence
      "Those runs also showed the simulation is not reproducible … until
      T10.F11 restores reproducibility and regenerates them" with one sentence
      stating that T10.F11 regenerated the four reports at the fix commit and
      a second run reproduced each; update the persistence numbers in that
      bullet only where the regenerated reports moved them. In the T10 track,
      append to the determinism-contract bullet one sentence that T10.F11
      restored the contract on 2026-09-04 and that the eight T10.F09 reports
      predate the fix and remain wall-clock evidence only. Add one dated line
      to the T01.F11 spec's "Notes for AI Agents" saying its reports were
      regenerated by T10.F11 and its tables describe the pre-fix draw. Run
      `make roadmap-check`.
- [x] Generate this feature's gate report with `make bench PROFILE=gate
      FEATURE=t10-f11-cross-process-reproducibility-of-seeded-runs`, append
      it to `closed` in `docs/progress/benchmark-series.json`, apply the
      re-pin rule below, and complete Performance and Goal Impact. The re-pin
      rule's second branch applied: the epoch baseline was not re-pinned.

## Verification

- [x] `cargo test -p v3-core --test viability` passes before other checks.
      Run first, before any edit: `ok. 25 passed; 0 failed` in 2.19 s; rerun
      after the fix: `ok. 25 passed; 0 failed` in 1.71 s.
- [x] The two operator tests fail before the fix (recorded output) and pass
      after it; `cargo test -p v3-core --lib` passes.

  Pre-fix, at `967bbe0f` with the tests added and the operators untouched,
  `cargo test -p v3-core --lib -- vm_mutate_paired_slot_address copy_mesh_backward_slice_is_reproducible`
  printed `test result: FAILED. 2 passed; 3 failed`, failing all three new
  tests:

  ```text
  ---- mutation::vm::tests::vm_mutate_paired_slot_address_is_reproducible_for_a_seed stdout ----
  assertion `left == right` failed: application 2 picked a different slot group
  than application 0 for the same seed: the candidate order is not a function of
  the genome
    left:  [... LoadSlotImm { dst: 0, slot_idx: 13 }, StoreSlotImm { slot_idx: 13, src: 0 }, ...]
    right: [... LoadSlotImm { dst: 0, slot_idx: 2 },  StoreSlotImm { slot_idx: 2,  src: 0 }, ...]

  ---- mutation::topology::tests::copy_mesh_backward_slice_is_reproducible_for_a_seed stdout ----
  assertion `left == right` failed: application 2 produced a different clone than
  application 0 for the same seed: the backlink candidate order is not a function
  of the genome
    left:  node 1 targets [... RouteTarget { target_id: NodeId(5), slot: 1 }]
    right: node 1 targets [... RouteTarget { target_id: NodeId(4), slot: 1 }]

  failures:
      mutation::topology::tests::copy_mesh_backward_slice_is_reproducible_for_a_seed
      mutation::vm::tests::vm_mutate_paired_slot_address_is_reproducible_for_a_seed
      mutation::vm::tests::vm_mutate_paired_slot_address_is_reproducible_for_any_slot_program
  ```

  The proptest failure produced
  `crates/v3-core/proptest-regressions/mutation/vm/tests.txt`, committed with
  the fix. After the fix, `cargo test -p v3-core --lib` reports
  `ok. 1016 passed; 0 failed` in 20.55 s.
- [x] The RNG-indexed pick audit table (fifteen sites, each with the list's
      source order) and the hash-map inventory re-check are recorded here.

  All fifteen `<list>[rng.gen_range(0..<list>.len())]` sites in non-test
  `v3-core` code, taken 2026-09-04 from
  `grep -rn "gen_range(0\.\." crates/v3-core/src --include "*.rs" | grep -v "/tests.rs"`
  and filtered to picks that index a named list:

  | # | Site | List | Source order | Verdict |
  | - | ---- | ---- | ------------ | ------- |
  | 1 | `creature/genome/analysis.rs:200` | `outputs` | `program.iter().enumerate()` filter | Program order — reproducible |
  | 2 | `creature/genome/analysis.rs:249` | `writers` | `program.iter().enumerate()` filter | Program order — reproducible |
  | 3 | `mutation/graph/hebbian.rs:201` | `others` | `ALL_RULES.iter()` filter | Const array order — reproducible |
  | 4 | `mutation/graph/hebbian.rs:310` | `others` | `ALL_CHANNELS.iter()` filter | Const array order — reproducible |
  | 5 | `mutation/graph/operators.rs:346` | `surfaces` | `0..compute_nodes.len()`, then sinks, then action bank, then `ExecuteGate` | Index-range order — reproducible |
  | 6 | `mutation/graph/operators.rs:491` | `cluster` | `Vec` grown by the random walk from `seed` | Draw order — reproducible |
  | 7 | `mutation/graph/operators.rs:514` | `neighbors` | `compute_nodes[current].inputs` then `compute_nodes.iter().enumerate()` | Vec/index order — reproducible |
  | 8 | `mutation/graph/operators.rs:662` | `eligible` | `compute_nodes.iter().enumerate()` filter | Vec order — reproducible |
  | 9 | `mutation/reachability.rs:68` | `eligible` | Caller's `Vec<usize>` (node indices) | Vec order — reproducible |
  | 10 | `mutation/topology/birth.rs:51` | `custom_sink_indices` | `output_sinks.iter().enumerate()` filter | Vec order — reproducible |
  | 11 | `mutation/topology/structural.rs:79` | `candidates` | `genome.nodes.iter()` filter | Genome order — reproducible |
  | 12 | `mutation/topology/structural.rs:231` | `new_ids` | **Was** `id_map.values()` (std `HashMap`); **now** pushed in `gene_indices` order | **Defect, fixed** |
  | 13 | `mutation/vm/operators.rs:744` | `terminal_positions` | `program.iter().enumerate()` filter | Program order — reproducible |
  | 14 | `mutation/vm/operators.rs:798` | `slot_indices` | `program.iter().enumerate()` filter | Program order — reproducible |
  | 15 | `mutation/vm/operators.rs:850` | `eligible` | **Was** `groups.into_iter()` (std `HashMap`); **now** a `BTreeMap`, ascending slot index | **Defect, fixed** |

  Two further RNG-indexed picks on the tick path use a count rather than
  `.len()` and so do not match the pattern above; both are audited here and
  are reproducible: `kernel/food_resource/growth.rs:116` and
  `kernel/ordinary_food/ecology.rs:429` index a `neighbors` buffer filled by
  iterating the fixed array `[Direction::N, E, S, W]`.

  Hash-map inventory re-check, from
  `grep -rn "HashMap\|HashSet" crates/v3-core/src --include "*.rs"` on
  2026-09-04 after the fix: no non-test `HashMap` or `HashSet` exists whose
  iteration order is observed, beyond the two corrected sites. The grep hits
  the plan did not list are all inside `#[cfg(test)]` modules — `sensors/visibility.rs:431`,
  `mutation/graph/operators.rs:1199`, `mutation/topology/routing.rs:333`,
  `creature/state.rs:541`, and `simulation/seeding.rs:166`/`:242` all sit
  after the `#[cfg(test)]` line in their file (288, 725, 143, 263, 140
  respectively). The `patterns/` module's `HashSet`s (`spiral.rs`, `maze.rs`,
  `star.rs`, `noise.rs`, `lines.rs`) are off the simulation path:
  `grep -rn "patterns::" crates/v3-core/src` outside `patterns/` returns
  nothing, and the only caller is `crates/v3-server/src/http/pattern.rs`. The
  `mutation/vm/operators.rs` `HashMap` is gone (now a `BTreeMap`), so the
  remaining non-test hash maps are exactly the lookup, membership, and keyed
  counter uses the plan classified.
- [x] `cargo test -p v3-core --test reproducibility` passes in under 30
      seconds in a debug build and reports the operator counts it observed.
      `ok. 1 passed; 0 failed` in 8.34 s (`make rust-test-reproducibility`
      runs the same target). With `-- --nocapture` it prints:
      `seed 20260904, 250 ticks: births=65, final_population=10, applied
      operators=45, Vm.MutatePairedSlotAddress=4,
      Topology.CopyMeshBackwardSlice=7, Topology.CopyMeshForwardSlice=13`, so
      requirement (b) holds without weakening. The test is a genuine detector,
      not a tautology: with the two operator fixes temporarily reverted in the
      working tree, the same command reported
      `test result: FAILED. 0 passed; 1 failed` on the work-counter assertion
      (the second run's counters were
      `[("mesh_hops", 176292), ("vm_steps", 499176), ("graph_relax_iters", 405058),
      ("plasticity_updates", 0), ("actions_applied", 16628), ("births", 65)]`
      and the first run's differed). The fixes were then restored and the test
      passes again.
- [x] `make rust-mutants` summary line, output path, and full survivor list,
      each survivor killed, equivalent, or deferred. Run once after the
      simplification pass, diffing against merge base `ba903c9d`:

  ```text
  rust-mutants: diff against ba903c9d27c736c989926453937c26f29f2d5708, output in /Users/istefanek/.local/share/petri-tools/mutants/t10-f11/mutants.out
  Found 2 mutants to test
  ok       Unmutated baseline in 18s build + 36s test
  2 mutants tested in 2m: 2 caught
  rust-mutants: no survivors
  ```

  Survivor list: empty — `missed.txt` and `timeout.txt` are both zero bytes,
  so there is nothing to kill, mark equivalent, or defer. The two mutants the
  diff generated were both caught (`caught.txt`):
  `structural.rs:183: replace clone_and_remap_slice with ()` and
  `vm/operators.rs:809: replace apply_mutate_paired_slot_address -> Result<(),
  MutationSkipReason> with Ok(())`. No `#[mutants::skip]` and no `exclude_re`
  entry was added.
- [x] Four regenerated reports name the fix commit; four second runs are
      equal; commands and wall-clock recorded.

  Every report's `environment.git_revision` is
  `b8887e1348cfb0657b4f467f411a8d4ec0525ed0`. Each command below was run once
  to `docs/progress/sweeps/t01-f11/wNNNN.json` and a second time to
  `<scratch>/wNNNN-rerun.json`, sequentially, never concurrently, from one
  detached POSIX `sh` script; the two parsed `deterministic` objects were then
  compared for whole-object equality with `python3` (`json.load`, `a == b`).

  ```sh
  make bench PROFILE=sweep OUT=docs/progress/sweeps/t01-f11/w0128.json BENCH_ARGS="--width 128 --height 128 --founders 64 --seeds 11,22,33 --ticks 2000 --feature t01-f11-baseline-persistence-characterization"
  make bench PROFILE=sweep OUT=docs/progress/sweeps/t01-f11/w0256.json BENCH_ARGS="--width 256 --height 256 --founders 256 --seeds 11,22,33 --ticks 2000 --feature t01-f11-baseline-persistence-characterization"
  make bench PROFILE=sweep OUT=docs/progress/sweeps/t01-f11/w0512.json BENCH_ARGS="--width 512 --height 512 --founders 1024 --seeds 11,22,33 --ticks 2000 --feature t01-f11-baseline-persistence-characterization"
  make bench PROFILE=sweep OUT=docs/progress/sweeps/t01-f11/w1600.json BENCH_ARGS="--width 1600 --height 1600 --founders 10000 --seeds 11,22,33 --ticks 2000 --feature t01-f11-baseline-persistence-characterization"
  ```

  | Report | In-place wall-clock | Second-run wall-clock | `deterministic` equal |
  | ------ | ------------------- | --------------------- | --------------------- |
  | `w0128.json` | 30 s (includes the release build) | 1 s | yes |
  | `w0256.json` | 8 s | 8 s | yes |
  | `w0512.json` | 120 s | 119 s | yes |
  | `w1600.json` | 588 s | 589 s | yes |

  What moved against the pre-fix reports at `d4d5005a`: 128 and 256 are
  unchanged in every field (totals `births=46` and `births=177`), because the
  corrected operators never reached a two-candidate list before those worlds
  went extinct. 512 seed 11 moved (peak 22,592 at tick 1,182 → 22,220 at
  1,275; tick-2,000 population 9,206 → 11,460; totals births 117,612 →
  138,536), and its other two seeds are unchanged. Every 1600 seed moved:
  peaks 35,327/36,265/35,363 → 35,279/36,369/35,433, minimum populations
  4,386/4,310/5,003 → 3,243/5,830/5,031, tick-2,000 populations
  6,790/7,291/7,015 → 5,291/10,997/8,130, totals births 438,189 → 479,652.
  This is trajectory drift from a changed pick, exactly as predeclared, not a
  change in the amount of randomness.
- [x] `make roadmap-check` passes after the document edits:
      `roadmap-check: validation passed`.
- [x] Benchmark report stored at
      `docs/progress/features/t10-f11-cross-process-reproducibility-of-seeded-runs.json`,
      generated with
      `make bench PROFILE=gate FEATURE=t10-f11-cross-process-reproducibility-of-seeded-runs`
      and appended to `closed` in `docs/progress/benchmark-series.json`.
- [x] `make check` passes (exit 0) at the closing commit: `MAKE_CHECK_EXIT=0`,
      18 `test result: ok` lines and no failure, including viability 25/25,
      v3-core 1,016 unit tests, the new `rust-test-reproducibility` target
      (1 passed in 8.57 s), and the v3-cli gate tests that compare the gate
      profile against both series references after the series append. It ran
      after the gate report was written and appended, so those comparisons
      used this feature's own report as the last closed reference.

## Performance and Goal Impact

Predeclared cost: none measurable. The fix sorts at most sixteen `u8` keys per
paired-slot application and collects at most eight ids per mesh-slice clone,
against ticks that cost milliseconds. Expected wall-clock delta: under 1
percent. This feature adds no diversity or cognition measure; every
`Undefined` indicator stays `Undefined`.

Predeclared trajectory effect: when a corrected operator has two or more
candidates, the same draw may now select a different candidate than the
pre-fix hash order did, so the gate profile's trajectory may change even
though it was already reproducible across processes at 128². Any work-counter
delta that results is trajectory drift from a changed pick, not compute cost;
the expected magnitude is within the 10 percent flag threshold on every
counter, with `births` the most sensitive because the gate records only about
25 births per seed.

Re-pin rule, from the T10 roadmap: if this feature's gate report has a
`deterministic` block different from the T10.F09 report's, the closing commit
sets `epoch_baseline` in `docs/progress/benchmark-series.json` to this
feature's report; if the blocks are identical, the epoch baseline is not
re-pinned. Either way the measured deltas against both references are
recorded here with their levels.

Measured 2026-09-04. The epoch baseline was **not** re-pinned: this feature's
gate `deterministic` block is equal, as a parsed object, to the T10.F09
report's (`json.load(a)['deterministic'] == json.load(b)['deterministic']`
is `True`), so the rule's second branch applies and `epoch_baseline` stays
`docs/progress/features/t10-f10-deterministic-benchmark-harness.json`. Only
the `closed` list gained this feature's report.

Cost, against both series references (`severe=false` for both):

| Counter (per creature-tick) | Current | T10.F10 epoch baseline | T10.F09 last closed | Level |
| --------------------------- | ------- | ---------------------- | ------------------- | ----- |
| `mesh_hops` | 1.998362 | 1.998362 (0.000000%) | 1.998362 (0.000000%) | ok |
| `vm_steps` | 28.028231 | 28.028231 (0.000000%) | 28.028231 (0.000000%) | ok |
| `graph_relax_iters` | 2.998624 | 2.998624 (0.000000%) | 2.998624 (0.000000%) | ok |
| `plasticity_updates` | 0.000000 | 0.000000 (n/a) | 0.000000 (n/a) | ok |
| `actions_applied` | 1.000000 | 1.000000 (0.000000%) | 1.000000 (0.000000%) | ok |
| `births` | 0.001212 | 0.001212 (0.000000%) | 0.001212 (0.000000%) | ok |

Wall-clock per creature-tick: 0.004577 ms, against 0.005448 ms for T10.F10
(−15.99%, level `ok`) and 0.004391 ms for T10.F09 (+4.22%, level `ok`); total
gate wall-clock 279.3 ms. Every deterministic counter is bit-identical to both
references, so the predeclared trajectory effect did not materialize at the
gate profile: the gate world is too small and too short for either corrected
operator to reach a candidate list with more than one entry, which is also why
the regenerated 128 and 256 sweeps are unchanged. The predeclared cost of
"none measurable" holds — the only measurable movement is wall-clock noise
well inside the 25 percent flag threshold.

Goal indicators are unchanged: population persistence and births per 100 ticks
are identical to the references, and every other indicator remains
`Undefined`, as predeclared.

## Success Criteria

- [x] Both operators pick from candidate lists ordered by genome content, and
      the per-operator tests and the in-process reproducibility test are in
      `make check`.
- [x] The audit records every hash-map iteration and RNG-indexed pick on the
      simulation path with its classification, and none other than the two
      corrected sites is order-dependent.
- [x] The four T01.F11 sweep reports name the fix commit and a second run of
      each reproduces its `deterministic` block.
- [x] The master roadmap's persistence numbers match the regenerated reports
      and its "one draw" caveat is gone; the T10.F09 reports are marked
      pre-fix.

## Notes for AI Agents

- Deferred finding (out of the simulation path):
  `crates/v3-core/src/patterns/noise.rs` collects a `HashSet` into a `Vec` in
  hash order and trims it randomly, so `generate_pattern_seeded` is not
  reproducible across processes. It serves only the `v3-server` pattern
  endpoint; the first feature that seeds barriers from patterns owns the fix.
