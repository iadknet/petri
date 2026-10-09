# Output-wiring experiment run: results (2026-10)

Date: 2026-10-09. Contract: the user's goal command of 2026-10-09 (one
predeclared row, W1, in the form of the
[output-wiring experiment plan](output-wiring-experiment-plan-2026-10-09.md)'s
row W1 with its gates W0 and W0.2, its readings and its draft validation rule,
taken as written), in the form of the
[born-connected run](born-connected-run-2026-10.md) (its gates, Advice table
and reviews are the precedent), extending the
[diagnosis follow-up run](evolvability-diagnosis-run-2026-10-followup.md)'s
row N3 vote-delta instrument, under the
[diagnosis plan](evolvability-diagnosis-plan-2026-10-05.md)'s rules 1 to 9 as
far as they apply to a one-row run and the
[run 1 plan](evolvability-exploration-plan-2026-10-01.md) (the five
not-forcing checks, matched controls, commit kinds, the Codex review command,
the close order), inside the
[T22 exploration contract](../roadmaps/t22-capability-assays-and-evolvability-lab.md).
Status: **predeclared** (row W1 not yet run).

## Question

The born edge (T20.F16, production since `e6538e10`) wires a new sensor into
an action vote only. The plan's local evidence says the other 67 sinks are not
under-wired but **wired and inert**: at tick 2,000 memory writes reach a slot
on 14 to 88 % of genomes, reads are common, and no creature's final action
depends on what memory holds (`memory_sensitivity` 0 of 18,053 over three
worlds), while no reading has ever covered custom outputs, router gates or
action parameters. Two questions, one row: (1) on the selected cohort of a
natural world, which output surfaces are wired, executed and causal, and
does a memory loop (a write to slot k and a read of slot k on one genome)
ever become causal; does memory use appear by 10,000 or 20,000 ticks (the
horizon question, read on `default` alone)? (2) If the born edge may land on
any of the 78 sinks that use their input (`wide`), do sensors born into
memory, the bus or a gate become causal there, and at what cost to the
vote-side gain T20.F16 bought?

## Run setup

- **Roles.** Session on Fable 5.1 (`claude-fable-5-1`), the implementer: it
  writes the instrument and the prototype, runs the gates and the row, reads
  the results and writes this note. The Fable advisor is enabled
  (`advisorModel: fable` in the project settings) and consulted through the
  `advisor` tool at every rule 8 point (the Advice table). No subagent
  spawned. Reviews: Codex `gpt-6.1-sol` `high`, read-only, through
  `codex exec` (`codex-cli 0.159.1`, the run 1 plan's command, wrapped in a
  POSIX `sh` script): (a) this predeclaration before any gate or row runs,
  its findings disposed and the amendments committed first; (b) the note's
  readings before closing.
- **Launch revision.** `main` `e847ec8b` (clean; T20.F16 closed at
  `b82d1924`, so the production engine carries the born vote edge, T03.F12's
  gestation and T24's food repairs). Worktree
  `.claude/worktrees/output-wiring` on branch `worktree-output-wiring`,
  created with `EnterWorktree` (`baseRef: head`); its head checked equal to
  `e847ec8b` after creation. `npm ci` in `frontend/`. Disk free at launch
  58 GiB (rule 9's stop line is 20 GiB).
- **Host.** At launch the host was on battery (43 %, discharging, 1:20
  reported); the user was told once in the session's first message. The
  runner records the power state at every heavy launch and wraps every heavy
  job in `caffeinate -i` with the awake-time killer of the born run's
  `diagrun.sh` (copied). Builds and the minute-scale gates run first so the
  world arms start as late as possible. Telemetry off on every `v3-cli`
  invocation; the census probe and the E4 probe are test binaries. One heavy
  job at a time; `pgrep -fl 'v3-lab|v3-cli|census-'` before each.
- **Evidence.** Raw output under `.bench-artifacts/lab/wiring/` in the
  worktree, copied to the main checkout at closing; compact summaries beside
  this note under [`output-wiring-run-2026-10/`](output-wiring-run-2026-10/)
  (each under the 1 MiB ceiling). Recipes are jq overlays of
  `experiments/worlds/canyon-country.json` stored under the evidence tree,
  never under `experiments/worlds/`.
- **Commit kinds.** `docs:` (this note and its folder), `instr:` (the census
  probe with the sink-use funnel, the memory-loop census and the memory
  sensitivities; observation only), `lab:` (the E4 `transition_rates`
  probe), `proto:` (the one default-`votes` mechanism prototype). `instr:`,
  `lab:` and `proto:` stay on the branch; only `docs:` lands on `main`.
- **Code taken from `worktree-born-connected`** (file state, no cherry-pick;
  sources cited in the commit messages): the census probe
  `crates/v3-cli/src/bench/diagnosis_census.rs` with its `#[cfg(test)]`
  module line (`24d0f553`: the funnel block, the vote-delta reading, the
  `births` block), the vote-delta reading
  `crates/v3-core/src/neighborhood/input_use/vote_delta.rs` with its module
  line (`24d0f553`), and the E4 probe `crates/v3-lab/tests/diagnosis_probe.rs`
  (`3abc1b70`, with its single-event-`InputRefAdd` tallies). **Nothing else
  of that branch's diff against `main` is taken**: the branch is based on
  `b0f47dd5`, before T20.F16 landed, so its `proto:` commits (`765e429c`,
  `a172c734`: `mutation.born_connected`) are superseded by the production
  engine and its other differences are `main` having moved. In the taken
  files the flag semantics are reworked: the census reads
  `mutation.born_connected_destinations` through the serialized
  `MutationConfig` (absent means `votes`), `frozen_mutation` drops that key,
  the header's `prototypes` entry carries it; the probe's `PETRI_E4_BORN`
  becomes `PETRI_E4_WIDE=1`, which sets `born_connected_destinations:
  "non_clear"` through the serialized config (the frozen probe, built
  without the prototype, rejects it, which is correct). The instrument
  therefore builds without the prototype and the frozen instrument of gate
  W0.2 is that build.
- **Draw-path drift since the born run.** `git diff --stat a172c734 main`
  over `v3-core/src/mutation`, `creature/genome`, `creature/founder`,
  `neighborhood` and `v3-lab` is not empty on the draw path (T20.F16 rewrote
  `input_ref/mod.rs`, 101 lines, and re-pinned `neighborhood/drift.rs`), so
  the born run's `b0/e4-on*.json` is **not** a reference for W0: the
  reference for the `votes` path is the frozen probe built from `main`
  without the prototype (below), and no comparison to the born run's raw
  output is made.
- **Instrument extension (`instr:`), three new checkpoint blocks** on the
  census row, each with its own wall time:
  1. **Sink-use funnel** (`sink_use`, `sink-use-v1`,
     `neighborhood::input_use::sink_use`), the output analog of the input
     funnel, on the selected cohort (20 parents) at every checkpoint. Stages
     per parent and sink: **wired**: a sink with at least one edge on a
     reachable Graph node (`mesh_reachable_nodes`); **executed**: that node
     ran in at least one scene execution on the funnel's panels (the
     `neighborhood-v1` battery plus the T11.F26 extension: recorded and
     authored contexts, the long sequences), recorded by an execution mode
     that keeps every `execute_node` call's node index; **causal**: clearing
     the sink's edges (the sink becomes unwired: a `WriteSlot` stops writing,
     a `CustomOutput` passes the bus value through, a `RouterGate` score
     stays 0, an `ActionParam` is not overwritten, a `ClearSlot` stops
     clearing, a vote sink contributes 0) on every reachable node and
     re-running every scene changes the complete committed action queue of
     at least one scene execution (the vote-delta reading's comparison; the
     ablation is **not** the input funnel's `ablated`, which rewrites read
     sources, since a zero-weight edge would still write 0.0 into memory).
     The ablation runs **per sink kind** (all six kinds, every sink of the
     kind on every reachable node cleared together) and **per wired (kind,
     slot)** (only sinks the parent carries wired). Reported per kind and
     per (kind, slot): parents wired, executed, causal (distinct parents),
     with `ActionVote` as the descriptive comparison row and `ClearSlot`
     reported but excluded from the non-vote clause (it cannot carry a
     sensor's value). `parents_evaluated` must be 20.
  2. **Memory-loop census** (`memory_loop`, `memory-loop-v1`, same module),
     per parent and slot k in 0..16: **write wired** (a `WriteSlot(k)` sink
     with edges on a reachable Graph node, or a VM `StoreSlotImm { slot_idx:
     k }` on a reachable VM node; register-indexed `StoreSlot` counted apart
     as dynamic writes), **read wired** (a `GraphSource::SharedMemory { slot:
     k, .. }` edge, current or previous tick, on a reachable Graph node, or a
     VM `LoadSlotImm`/`LoadSlotPrev` of slot k; register-indexed `LoadSlot`
     counted apart as dynamic reads); a **loop** is write wired and read
     wired on the same genome; **executed**: the writing and a reading node
     both ran on the panels; **causal** (predeclared definition, the cheaper
     of the two the advisor named): the `WriteSlot(k)` sink's per-slot
     ablation above changes an action queue and a read of slot k is wired
     (the read side has no causal instrument of its own; a VM write cannot
     be ablated by the sink rewrite and is counted in the structural stages
     only). Reported per parent and pooled: slots written, slots read,
     loops, loops executed, loops causal, dynamic reads and writes, and
     `ClearSlot` sinks wired.
  3. **Memory sensitivity at the checkpoint** (`memory_sensitivity`,
     `temporal_memory_sensitivity`): the goal profile's own readings
     (`observe_final_actions`, `observe_temporal_actions` in v3-core;
     `memory_sensitivity` and `temporal_memory_sensitivity` in
     `v3-cli::bench::indicators`) over every living creature at both
     checkpoints: creatures whose final action differs with memory zeroed,
     scrambled (a fixed rotation), or either, and the three temporal
     components (previous slots, persisted outputs, operator state). The
     plan's limit stands: a final-action comparison misses a loop whose
     effect shows one tick later.
  The funnel's retention proposals run on the arm's config with the
  prototype key dropped (frozen one-event meaning); the `births` block and
  the new blocks run under the arm's actual policy (the sink-use funnel and
  the loop census draw no mutation; the births block uses the arm's
  destinations); the vote-delta reading draws no mutation.
- **Branch verification.** TDD for the prototype; `cargo test -p v3-core
  --test viability` first (the prototype touches births). The `votes`
  config serializes without the new field (`skip_serializing_if`), so its
  `config_digest` is `main`'s and the `v3-cli` recipe-digest pin test is
  expected to stay green; `make check` on the branch is recorded under Gate
  results with every exit code and gates the driver.
- **Reference binary.** `v3-cli` built in release from the untouched
  worktree at `e847ec8b` before any source change, copied to
  `.bench-artifacts/lab/wiring/v3-cli-main` and hashed: the pinned
  production binary of the launch revision (gate W0.2 (i)'s smoke).
- **Frozen instrument.** The census test binary and the probe built from the
  `instr:` + `lab:` state without the `proto:` commit, copied to
  `.bench-artifacts/lab/wiring/census-frozen` and `probe-frozen` and hashed:
  the frozen probe is W0's reference for the `votes` path; the frozen census
  runs the control recipes (`default`, `default-mutoff`, both seeds, 20,000
  ticks) and the prototype binary's `votes` arms must reproduce its samples
  and timing-free checkpoint blocks (gate W0.2 (ii)).
- **Branch and commits.** Recorded as they land (predeclaration, amendment
  A1, `instr:`, `lab:`, `proto:`, the run's `docs:` commits, the binary
  hashes, the viability result).
- **Landing scope.** Only `docs:` commits land on `main`; `proto:`, `instr:`
  and `lab:` stay on the branch, which is kept. T22 instrument candidates
  from this run (on the branch): the sink-use funnel and the memory-loop
  census beside the input funnel, the vote-delta reading and the births
  block.

## The prototype (W1's mechanism, `proto:`)

`mutation.born_connected_destinations: votes | non_clear` (default `votes`;
`policy-deviation` when `non_clear`; serialized only when `non_clear`, so the
`votes` config is byte-identical to `main`'s). `votes` **is production**
(T20.F16): on a Graph node `InputRef.Add` appends the reference and takes
three draws in order, the sub-value uniform under the reference's width, the
sink `gen_range(0..27)` over `VoteSink` by index, the weight uniform on
[−1, 1], and pushes one edge onto that `ActionVote` sink; the `votes` path is
draw for draw that code. `non_clear`: the same event, the same three draws
in the same order, with the second draw `gen_range(0..78)` mapped by index
over the 78 sinks that use their input, in catalog order with the 16
`ClearSlot` sinks skipped: 0..24 `CustomOutput(0..24)`, 24..32
`RouterGate(0..8)`, 32..48 `WriteSlot(0..16)`, 48..75 `ActionVote` in
`VoteSink` index order, 75..78 `ActionParam` in `ActionParamField::ALL`
order. `ClearSlot` is excluded because it ignores its input (whatever is
wired to it, the slot is zeroed when the node runs; a sensor cannot be
carried), not for safety. The edge is pushed only when the index fits `u16`
and the node carries the drawn sink (a production node always does; a test
genome with a truncated catalog may not, and the event then stays a plain
declaration with the same draw stream). The event stays one applied
`InputRefAdd`; the operator definitions and weights, the requested-supply
law and the birth summary's accounting are unchanged; within a fired birth
the later events can differ between the two destination classes (a
different sink joins the edge pools, so later draws shift), so such births'
applied and skip records are not each other's; the `votes` path draws
nothing extra and reproduces `main` birth for birth. VM nodes are unchanged
under both values. The added edge is an ordinary edge afterwards.

**Natural analog** (the plan's): duplicated opsins came to couple to
alternative G-protein cascades, and a receptor's downstream pathway is
re-coupled over evolution; "expressed into an existing cascade" does not
single out the motor end, a memory or bus surface is as legitimate a cascade
as a vote. The prototype approximates integration by a fresh random
connection into any input-using surface and models neither duplication nor
pathway inheritance.

**The five not-forcing checks.** (1) A general rule over one operator and
one placement class (any input-using sink of the same node); it names no
sensor family, no arena, no sink by meaning (uniform by index over all 78)
and no scene. (2) The analog and its mechanism are above. (3) No
correspondence: sink, sub-value and sign are uniform draws. (4) No score
enters any draw; the row runs in a natural world with births and no scorer.
(5) What variation still has to discover: the reference, the right surface
among 78, the sign and the weight. What `non_clear` changes beyond the
destination class: nothing else (same event, same supply, same gestation
unit); what the mutation-off pairs cannot separate: the destination class
from the rest of the born mechanism, which is production on both arms.

**Contracts it deviates from, stated.** The T20.F16 spec's Non-Goal "No
destination other than a vote sink of the same node, no compute-node edge",
its spec-owner decision "destination one uniform `ActionVote` sink", and the
[mutation spec](../reference/v3-mutation-spec.md)'s InputRef-domain `Add`
paragraph, which names the vote sink draw `gen_range(0..27)`; the T20.F02
[neutral-connection contract](../specs/roadmap/t20-f02-graph-neutral-connection-contract.md)'s
unsupported-destinations row (`ClearSlot`, `WriteSlot`, `CustomOutput`,
`ActionParam`, `RouterGate` outside the eligible set because a wired empty
sink can clear or overwrite incumbent data), which the born edge at a live
weight already sits outside on the vote surface and `non_clear` extends to
the overwrite surfaces: an overwrite sink replaces incumbent data with the
sensor's value, so harm per fired birth may rise (the births block reads it
descriptively). On this branch the prototype is a `policy-deviation` arm
behind a default-`votes` field; a feature would amend the three sentences
explicitly. The lab's recruitment-path qualification fixtures
(`neighborhood/recruitment_paths/qualification.rs`) pin accepted born steps
into `Move(E)` under pinned seeds; the census does not call them and every
test runs at the default `votes`, so they are unaffected; under `non_clear`
they would not hold and are not run with it.

## Gates (predeclared, blocking)

Every gate must pass, after any disposition and rerun, before the step it
blocks runs: W0 before any world arm, W0.2's smoke before the pairs, W0.3 and
W0.2's full comparison before the `wide` arms are read; the row's agreement
check blocks the interpretation of its causal and vote-delta readings.

| ID | What | Rule |
| --- | --- | --- |
| W0 | Mechanism identity and effect on the founder, through the E4 probe (`PETRI_E4_BIRTHS=1000000`; parents `founder` (a), `founder+declared` (b), `founder+declared+zero-edge` (c); no elites): the **frozen probe** (built from `main` without the prototype) once, the **prototype probe** with `votes` once and with `non_clear` (`PETRI_E4_WIDE=1`) once, plus a supplementary `non_clear` run at 10⁷ births for the destination-spread criterion | **`votes` path:** the prototype probe's `votes` output equals the frozen probe's in the **deterministic projection** (every field but `wall_secs` and the probe's config metadata) on all three parents, `genome_size`, `executed_resolved` and `frozen_age` compared first: `default` is byte-identical to T20.F16's engine. **`non_clear` path:** (1) every single-event target declaration arrives connected: on parent (a), among the single-event `InputRefAdd` births that declared the target on the vote node, `connect_any_given_declared` is exactly 1 and every `single_add_births.connect_by_location` key is `sink:<kind>` with no `ClearSlot` and no `compute:` key, every `connect_by_sub` key in 0..6; over all births `connect_any`'s CP interval contains `declared_vote_node_target`'s point. (2) Destinations spread over the 78 sinks by index: on the 10⁷ run, the single-event destinations hit **all 78** non-clear sink keys, and the share of `sink:ActionVote(…)` keys among them has a 95 % CP interval containing 27/78 = 0.3462 (about 1,400 declarations: an interval about ±2.5 % wide); reported beside it, descriptively, the useful-signed share among single-`Add` declarations against 2/189 × 27/78 = 0.00366 and the per-kind shares against 24/78, 8/78, 16/78, 3/78. (3) Requested supply unchanged: `attempted_events_per_birth` on every parent equal between `votes` and `non_clear` (expected identical: the requested law is untouched). (4) `InputRefAdd` applied per birth within 5 % of the `votes` path's on every parent (not expected identical: later events of a fired birth can differ). (5) Integrated-loss equivalence: `lost_declaration` on (b) and (c) within 5 % of the `votes` path's, routes reported beside, with the born run's value-versus-occurrence limit. A failure of the `votes` identity or of (1) or (2) means the mechanism does not do what it says; a failure of (3), (4) or (5) beyond tolerance is investigated and explained; in every case W0 must pass after the disposition and a rerun before any world arm runs |
| W0.2 | Instrument identity: the census probe on the prototype binary at `votes` against (i) the launch-revision production binary `v3-cli-main` and (ii) the frozen instrument built without the prototype | (i) **Smoke:** a 2,000-tick run of the census probe (`default` recipe, seed 1022, checkpoints 1,000 and 2,000, every reading on) against `v3-cli-main --telemetry off run --ticks 2000 --sample-every 500 --seed 1022 --config default.json`: the sample tick sets equal (500, 1,000, 1,500, 2,000, both sides), every compared field present on both sides, `population`, `mean_genome_size`, `mean_generation` and `reproduction_actions_spawned_total` equal at every tick, both runs completed to 2,000, and the census header's `config_digest` equal to the run's `run_started` digest; `b02check.py` fails on any missing tick, field or completion record. The smoke blocks the pairs and the main arms. (ii) **Full comparison:** the frozen instrument runs `default` and `default-mutoff` on both seeds (20,000 ticks; the reference runs); the prototype binary's `default` and `default-mutoff` runs must equal them on every sample row (the fields above plus `mutation_events_applied_total` and `mean_energy`), the extinction tick, the footer's `ticks` and `extinct`, and every checkpoint block (`input_use`, `e6`, `vote_deltas`, `births`, **`sink_use`, `memory_loop`, `memory_sensitivity`, `temporal_memory_sensitivity`**, `living`, `selected_parents`, `selected_generations`, `age_deciles`, `energy_deciles`), each serialized once with sorted keys and compact separators and hashed; the header's `prototypes` entry, `config_digest` and the timing fields are excluded and listed. A full-run inequality blocks the `wide` arms from being read until explained and fixed |
| W0.3 | The mutation-off pair is identical by construction: with `per_unit_rate` 0 no event is requested, so no `Add` fires and the destination field is never read | `wide-mutoff` and `default-mutoff` on each seed: the same sample fields, extinction tick, footer and timing-free checkpoint blocks as W0.2 (ii) equal. The header's `config_digest` **differs** (the field serializes when `non_clear`) and `prototypes.born_connected_destinations` differs (`votes` against `non_clear`); both differences are verified, and nothing else in the header but the recipe path and the run label may differ. The births block's `mutation_config` entry is compared apart (every key but the destination field equal). A pair extinct before its first checkpoint has no block comparison, and the checker says so instead of failing (the born run's amendment A3). An inequality is a defect in the prototype's `votes` path and blocks the `wide` arms |

**Gate results.** Recorded as they land.

## Ledger

One row. No diagnostic (authored-genome) arm is used, and no authored genome
enters the natural-world arms; W0's prepared parents are measurements, not
mechanisms. A killed run is `incomplete`, never a biological negative; an
extinction is a finding.

| ID | Question | Method, instrument, sizes | Prediction / decision rule | Status | Result |
| --- | --- | --- | --- | --- | --- |
| W1 | Which output surfaces are wired, executed and causal on a natural world's selected cohort, does a memory loop become causal, does memory use appear by 10,000 or 20,000 ticks, and do sensors born into any input-using surface (`non_clear`) become causal there without costing persistence or more than two thirds of the vote-side gain? | The prototype above (`proto:`, `mutation.born_connected_destinations`, default `votes`, `policy-deviation` at `non_clear`). **Tested on the founder at unit level before any gate:** `votes`, `Add` on a Graph node is draw for draw production (one reference, one `ActionVote` edge, same draws); `non_clear`, `Add` on a Graph node appends one reference and exactly one edge on a non-`ClearSlot` sink, sourced from the appended index at a sub-value under the reference's width, weight in [−1, 1], compute nodes and the other node untouched; `non_clear` on a VM node appends one reference and leaves the backend unchanged; the field reaches `Add` through the config, `votes` is the default, absent from the serialized config at `votes` and round-trips at `non_clear`; over 4,096 seeds the `non_clear` edge reaches all 78 sinks with the vote share near 27/78; a proptest over generated genomes holds the one-reference, at-most-one-edge, never-`ClearSlot`, source and untouched-node invariants. **Design as B1:** Canyon goal recipe (`experiments/worlds/canyon-country.json`, jq overlays), seeds 1022 and 2022, 20,000 ticks, samples every 500, every arm on both seeds, through the census probe with checkpoints 10,000 and 20,000 (selected cohort of 20 parents, founder cohort, drift empty; the input funnel, the vote-delta reading, the births block, the sink-use funnel, the memory-loop census and the memory sensitivities at both; E6 at 20,000), 3 h awake kill per run, one job at a time. Arms: `default` (`votes`, production), `wide` (`non_clear`), `default-mutoff` and `wide-mutoff` (`per_unit_rate` 0: identical by construction, gate W0.3). **Readings:** window means over ticks 15,000 to 20,000 (population, genome size in founder units (97), births per creature-tick, extinction interval); the typed target-family funnel rows (`AreaFoodSummary:0`, `AreaFoodSummary:1`, `NeighborBarrierRing`: declared / connected / executed / causal, distinct parents, `parents_evaluated` 20) at both checkpoints; the vote-delta block at both (executed target channels, channels whose ablation changes an action queue, channels whose ablation moves any vote); the **agreement check** (per typed family, the funnel's `family_causal` against the distinct parents with an action-changing vote-delta channel of that family; per channel row likewise), which blocks the interpretation of the causal and vote-delta readings until it holds; the **sink-use funnel** per kind and per (kind, slot) (wired / executed / causal parents) at both; the **memory-loop census** (slots written, read, loops, loops executed, loops causal, dynamic reads and writes) at both; **`memory_sensitivity`** (zeroed, scrambled, either, over all living) and the three temporal components at both; dead / silent / changed per birth from the births block, descriptive | **Prediction, with a direction or its absence for every reading.** `wide` persists on both seeds; its window-mean population **below** `default`'s but above 0.5 × (an overwrite sink replaces incumbent data with the sensor's value in every fired `Add` on a Graph node, E5 found whole-family wiring harmful 4 of 5; structural connection does not guarantee a behavior change); genome size **not below** `default`'s; births per creature-tick: **no direction**; extinction: none predicted; dead, silent, changed per birth: **no direction** (underpowered and battery-limited, as B1 found). Input funnel typed rows at 20,000, descriptively: `family_declared` **not reliably different** (both arms protect a connected declaration from pruning), `family_connected` tracks `family_declared` on both arms, `family_executed` and `family_causal` on the vote surface **down** under `wide` (27 of 78 born edges reach a vote), about one third of `default`'s, with the vote-delta's action-changing target channels **down** in step. **Sink-use funnel** at 20,000: under `default`, `WriteSlot` and `CustomOutput` wired on most parents, executed on most, causal on **few or none**; `RouterGate` and `ActionParam` wired on fewer, causal on none; whether any non-vote kind becomes causal by 20,000 where it is not at 10,000 is the horizon reading; under `wide`, **at least one causal incidence on `WriteSlot` or `CustomOutput` where `default` has none** (the row's clause). **Memory-loop census:** loops present on both arms at both checkpoints, loops causal **0 at 10,000** on both arms; at 20,000 `wide` **not below** `default`. **`memory_sensitivity` on `default`:** the plan leaves it open; this run predicts **0 of all living on both seeds at both checkpoints** (the live survey's quarter came at 281,000 ticks on an older substrate), with the temporal components near the T20.F16 goal's levels (operator state under 1 %, persisted outputs about 1 %, previous slots 0); `wide`: not predicted. E6: **no prediction**. The validation reads 20,000; the 10,000 checkpoint is descriptive. **Validation rule, verbatim from the plan's draft rule, which the goal command fixes as written:** "`wide` is **validated as a lead** if on both seeds at 20,000: `wide` persists; its window-mean population is at least 0.5 × `default`'s; the sink-use funnel shows at least one causal incidence on a non-vote sink where `default` has none; and the vote-side target causal count is not below one third of `default`'s. **The horizon question is answered apart**, on `default` alone: `memory_sensitivity` above 0 at 20,000 on either seed means memory use is a matter of horizon and T11.F10 should read later checkpoints; 0 on both seeds with loops present means the block is selection or closure, and option C or a temporal assay comes next. A 20-parent cohort is a lead-selection screen; one lineage can carry a count." **Resolutions, predeclared:** (i) "on both seeds" governs every clause after it; each clause is read on each seed. (ii) "A causal incidence on a non-vote sink where `default` has none" is read **per sink kind** among `CustomOutput`, `RouterGate`, `WriteSlot` and `ActionParam` (`ClearSlot` cannot carry a sensor and is reported only) on the selected cohort at 20,000: some such kind with causal parents ≥ 1 under `wide` and 0 under `default` on that seed. If `default` has at least one causal parent on **every** one of the four kinds on a seed, the clause fails on that seed by construction; that is recorded as a finding about the rule, with the per-kind and per-slot comparison (`wide` against `default`, numerators and distinct parents) beside it, and the row is not validated by this rule. (iii) "The vote-side target causal count" is Σ `family_causal` (distinct parents) over the three typed target rows on the selected cohort at 20,000; the clause holds when `wide`'s is at least one third of `default`'s, and trivially when `default`'s is 0. (iv) If `default` is extinct before 20,000 or has no living sample in the window on a seed, the population ratio and the 20,000 comparison are undefined on that seed and the row is not validated by this rule, with `wide` persisting where `default` died recorded as a finding (the born run's `default` had 795 and 815 living at its trough on these seeds, and its `born` arm 10 living at tick 4,500 on seed 2022; `born` is now production, so the default's trough on these seeds is live). (v) `wide` extinct on either seed: not validated, an extinction finding. (vi) The horizon question: "`memory_sensitivity` above 0" is `different_from_either_count` ≥ 1 over all living creatures at 20,000 on the `default` arm; "loops present" is at least one parent with a loop in the memory-loop census at 20,000 on that seed; the three temporal components are reported beside, not read by the rule. (vii) A 20-parent cohort is a lead-selection screen: one parent can carry the incidence, numerators and direction are reported, and no population-wide increase is claimed. (viii) The funnel's `causal` and the vote-delta's action-queue change are one fact measured twice; the agreement check must hold for the vote-side clause to be read. **Alternative explanations named now:** the vote-side dilution itself (27 of 78) lowering selection pressure on the whole mechanism; the overwrite harm changing which lineages survive; a causal non-vote incidence that is one lineage's; drift and shared ancestry within a 20-parent sample; checkpoint sampling swings (B1's 8 → 3); the panels' exposure (the battery presents zero for the target families; the recorded and authored contexts carry the causal stage); the sink ablation's intervention effect on dispatch (clearing a `RouterGate` or `CustomOutput` changes which successor runs, so a causal incidence is the total effect, not a sensor's contribution); no edge provenance in the census (a causal non-vote sink under `wide` may carry an ordinary `AddGraphEdge` edge, as under `default`; the mechanism's fingerprint is `family_connected` tracking `family_declared` and the destination kinds the born edges can reach). **Not validated otherwise.** A kill marks the run `incomplete` and it is rerun once at 6 h | **predeclared** | |

## Advice

Every Fable advisor consultation rule 8 lists, with the disposition of each
point raised.

| # | Rule 8 point | Advice (summary) | Dispositions |
| --- | --- | --- | --- |
| 1 | Before anything was built: the approach to the run after the contract, the branch code and the engine were read | Shape right; ten points. (1) The base moved: T20.F16 is production, so take only the census probe, the vote-delta reading and the E4 probe as file state, rework the flag semantics (`born_connected_destinations` through the serialized config, `PETRI_E4_WIDE`), and make W0's `votes` identity the frozen probe against the prototype probe, not the born run's raw output; check the draw-path diff first. (2) Read `recruitment_paths` on main for born-edge assumptions before writing the proto; predeclare the 78-index map, `votes` draw for draw production, `non_clear` one `gen_range(0..78)` in the sink's place, `skip_serializing_if` so the default digest is main's. (3) The sink ablation is clearing the sink's edges, not the funnel's `ablated`; record executed nodes through a `MeshExecutionMode` that keeps `node_idx`; report per kind and per (kind, slot) with `ActionVote` as a comparison row. (4) The memory read side has no causal instrument: predeclare loop-causal as the write sink causal under the sink ablation with a read of slot k present. (5) `memory_sensitivity` and the temporal readings are reachable from the census (`pub(super)` in `bench::indicators`); run them over all living at both checkpoints; record the final-action limit. (6) Fix the rule's resolutions: per sink kind; `default` likely has causal `WriteSlot` and `CustomOutput` already, so predeclare the by-construction failure as a finding about the rule; define the vote-side count as Σ `family_causal`, trivially true when the default's is 0; carry B1's resolutions; the default's trough on seed 2022 is live. (7) Grow W0.2's hash list with the new blocks; keep the pairs' check non-vacuous; carry the B0.2 (i) smoke against a `v3-cli` built at `e847ec8b`. (8) State the deviated contracts: the T20.F16 spec Non-Goal, T20.F02's unsupported destinations, the mutation spec's `Add` paragraph; the born run's three sentences are amended on main. (9) Battery: say it once, record power at every launch, builds and W0 first. (10) Turn economy: copy the born run's scripts with `sed` renames, one waiter per milestone, batched commits, the note in one write; the Codex-takes-every-role memory is stale; confirm the head, `npm ci`, viability first | All adopted in this predeclaration: (1) Run setup (code taken, draw-path drift) and gate W0; (2) the prototype section (index map, the recruitment-path fixtures) ; (3) and (4) the instrument extension; (5) block 3; (6) resolutions (ii) to (vi); (7) gates W0.2 and W0.3; (8) "Contracts it deviates from"; (9) Run setup (host) and the Order; (10) the scripts under `.bench-artifacts/lab/wiring/`, the Order |

## Codex reviews

### Review (a): the predeclaration, before any gate or row ran

Recorded when the job completes.

## Order

Predeclaration committed → Codex review (a) launched in the background → the
`instr:` and `lab:` commits (built without the prototype; the frozen census
and probe binaries copied and hashed) → the `proto:` commit under TDD, `cargo
test -p v3-core --test viability`, the unit tests → review (a) disposed,
amendment A1 committed → the builds and `make check` (gating the driver) → W0
(frozen, `votes`, `non_clear`, the 10⁷ `non_clear` run; minutes; must pass) →
the census smoke on the 200 × 200 recipe with every reading on (seconds) →
W0.2 (i) smoke (about 6 min; must pass) → the frozen instrument's reference
runs (`default-mutoff`, `default` on both seeds; about 25 min) → the
prototype binary's `default-mutoff` and `wide-mutoff` on both seeds with W0.3
→ its `default` on both seeds → W0.2 (ii) full comparison (must pass before
`wide` is read) → `wide` on both seeds → the W1 reading with its consultation
→ the plan forward (consultations before and after the draft) → Codex review
(b) → close in the run 1 order, docs only.

## Plan forward

Drafted after W1 is read.

## Cost

| Item | Wall |
| --- | --- |
| Recorded as they land | |
