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

- [ ] Run `cargo test -p v3-core --test viability` first (mutation operators
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
- [ ] Add a proptest in `mutation/vm/tests.rs`: for any program generated
      from slot instructions over slots `0..16` (with at least one paired
      group) and any seed, two applications with the same seed produce equal
      programs. Commit any `proptest-regressions/` file that appears.
- [ ] Add one fast in-process reproducibility test in
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
- [ ] Commit the fix and tests as one commit; its hash is the fix commit
      every regenerated report must name in `environment.git_revision`.
- [ ] Regenerate the four T01.F11 reports in place with the exact T01.F11
      commands (`--feature t01-f11-baseline-persistence-characterization`,
      seeds 11,22,33, 2,000 ticks, density-matched founders 64/256/1024/10000,
      output `docs/progress/sweeps/t01-f11/wNNNN.json`), the two long ones
      detached and polled, in sequence. Then run each command a second time
      to a scratch path and compare the parsed `deterministic` objects with
      `python3` (`json.load`, whole-object equality); all four must be equal.
      Record commands, wall-clock, and the four equality results.
- [ ] Refresh `docs/roadmap.md`'s dated baseline bullet: replace the sentence
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
- [ ] Generate this feature's gate report with `make bench PROFILE=gate
      FEATURE=t10-f11-cross-process-reproducibility-of-seeded-runs`, append
      it to `closed` in `docs/progress/benchmark-series.json`, apply the
      re-pin rule below, and complete Performance and Goal Impact.

## Verification

- [ ] `cargo test -p v3-core --test viability` passes before other checks.
- [ ] The two operator tests fail before the fix (recorded output) and pass
      after it; `cargo test -p v3-core --lib` passes.
- [ ] The RNG-indexed pick audit table (fifteen sites, each with the list's
      source order) and the hash-map inventory re-check are recorded here.
- [ ] `cargo test -p v3-core --test reproducibility` passes in under 30
      seconds in a debug build and reports the operator counts it observed.
- [ ] `make rust-mutants` summary line, output path, and full survivor list,
      each survivor killed, equivalent, or deferred.
- [ ] Four regenerated reports name the fix commit; four second runs are
      equal; commands and wall-clock recorded.
- [ ] `make roadmap-check` passes after the document edits.
- [ ] Benchmark report stored at
      `docs/progress/features/t10-f11-cross-process-reproducibility-of-seeded-runs.json`.
- [ ] `make check` passes (exit 0) at the closing commit.

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

## Success Criteria

- [ ] Both operators pick from candidate lists ordered by genome content, and
      the per-operator tests and the in-process reproducibility test are in
      `make check`.
- [ ] The audit records every hash-map iteration and RNG-indexed pick on the
      simulation path with its classification, and none other than the two
      corrected sites is order-dependent.
- [ ] The four T01.F11 sweep reports name the fix commit and a second run of
      each reproduces its `deterministic` block.
- [ ] The master roadmap's persistence numbers match the regenerated reports
      and its "one draw" caveat is gone; the T10.F09 reports are marked
      pre-fix.

## Notes for AI Agents

- Deferred finding (out of the simulation path):
  `crates/v3-core/src/patterns/noise.rs` collects a `HashSet` into a `Vec` in
  hash order and trims it randomly, so `generate_pattern_seeded` is not
  reproducible across processes. It serves only the `v3-server` pattern
  endpoint; the first feature that seeds barriers from patterns owns the fix.
