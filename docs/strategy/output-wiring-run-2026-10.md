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
an action vote only. The plan's local evidence says the **memory surface** is
not under-wired but **wired and inert**: at tick 2,000 memory writes reach a
slot on 14 to 88 % of genomes, reads are common, and no creature's final
action depends on what memory holds (`memory_sensitivity` 0 of 18,053 over
three worlds); custom outputs, router gates and action parameters have never
been measured at all (review (a) finding 19). Two questions, one row: (1) on the selected cohort of a
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
     The **executed** stage is a dispatch record (the node was dispatched;
     whether its effects committed or a VM memory instruction ran is not
     recorded: review (a) finding 10). The ablation runs **per sink kind**
     (all six kinds, every sink of the kind on every reachable node cleared
     together: the **joint** reading) and **per wired (kind, slot)** (only
     sinks the parent carries wired). Reported per kind: parents wired,
     executed, causal under the joint kind ablation (`parents_causal_kind`)
     and parents with any individually causal slot (`parents_causal_any_slot`);
     per (kind, slot): parents wired, executed, causal. **The joint
     kind-ablation causal-parent count alone decides resolution (ii)**; the
     per-slot reading is descriptive, cannot override it, and a disagreement
     between the two (redundancy, cancellation, dispatch changes) is recorded,
     not treated as an instrument failure (finding 3). `ActionVote` is the
     descriptive comparison row and `ClearSlot` is reported but excluded
     from the non-vote clause (it cannot carry a sensor's value).
     `parents_evaluated` must be 20. **Caveat (finding 4):** clearing the
     last wired sink of a Graph node with no compute nodes can disable its
     charged visit, so an action change can come from saved energy or a
     changed dispatch rather than from the sink's value reaching anything;
     every causal reading is an intervention effect on these panels.
  2. **Memory-loop census** (`memory_loop`, `memory-loop-v1`, same module),
     per parent and slot k in 0..16: **write wired** (a `WriteSlot(k)` sink
     with edges on a reachable Graph node, or a VM `StoreSlotImm { slot_idx:
     k }` on a reachable VM node; register-indexed `StoreSlot` counted apart
     as dynamic writes), **read wired** (a `GraphSource::SharedMemory { slot:
     k, .. }` edge, current or previous tick, on a reachable Graph node, or a
     VM `LoadSlotImm`/`LoadSlotPrev` of slot k; register-indexed `LoadSlot`
     counted apart as dynamic reads). VM immediates are normalized modulo 16
     as the VM resolves them; a Graph memory source past slot 15 resolves to
     0.0 and is no read (finding 11; one boundary fixture each). A **loop**
     is write wired and read wired on the same genome; **executed**: a
     writing node and a reading node were both dispatched on the panels (a
     dispatch proxy, in any scene; VM-only loops are included in this
     structural stage); **causal** (predeclared, the cheaper of the two the
     advisor named, and labelled for what it is: a **write-ablation-plus-
     read-wiring proxy**, not evidence that the loop mediates the change):
     the `WriteSlot(k)` sink's per-slot ablation above changes an action
     queue and a read of slot k is wired. The read side has no causal
     instrument of its own; a VM write cannot be ablated by the sink rewrite,
     so a VM-only loop is never loop-causal; and the finding 4 caveat
     (activation and energy) applies. Reported per parent and pooled: slots
     written, slots read, loops, loops executed, loops causal, dynamic reads
     and writes, and `ClearSlot` sinks wired.
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
  destinations); the vote-delta reading draws no mutation. **E6 runs at
  every checkpoint** (finding 8; the inherited census emitted it at one
  tick), so every reading is on at both. **Instrument fixtures (finding
  9),** on the branch before any gate: a vote sink causal and a custom
  output executed-only on the funnel's panels, a Graph write-and-read loop
  with the out-of-range source rejected and a `ClearSlot` reported apart, VM
  immediate wrapping and dynamic addressing counted apart with a VM-only
  loop never loop-causal, and purity (two readings equal, the genome
  untouched); joint-against-individual disagreement is not fixtured and is
  recorded descriptively when it occurs. **Cost (finding 17):** the sink
  funnel runs 20 × 264 × (1 + 6 + S) scene executions per checkpoint, S
  the parent's wired sinks (at most 94), and the memory readings about ten
  mesh evaluations per living creature plus sensor assembly; the actual S,
  living counts and wall times are recorded beside the inherited estimates,
  and the census smoke on the founder-scale recipe is a lower bound, not a
  projection.
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
- **Amendment A1 (pre-data, after Codex review (a)).** The 19 dispositions
  below; the instrument changes (E6 at every checkpoint, the Graph
  out-of-range read rejected, the fixtures) and the prototype's boundary
  test are second `instr:` and `proto:` commits, and the frozen instrument
  is built from the second `instr:` state.
- **Branch and commits.** Branch `worktree-output-wiring` from `e847ec8b`:
  `docs:` `1454f21a` (the predeclaration); `instr:` `520b4b4b`; `lab:`
  `3ead37f6`; `proto:` `2065366e`; `instr:` `45923a4c` (amendment A1);
  `proto:` `7f1c8da8` (A1 boundary test); `docs:` `de613af3` (A1); then this
  run's `docs:` commits. The **frozen revision** is `d2100d8b` in a detached
  nested worktree (`.bench-artifacts/lab/wiring/frozen-tree`, removed at
  closing): `3ead37f6` with `45923a4c` cherry-picked, so the instr+lab state
  without any `proto:` commit, reproducible from the kept branch. `cargo test
  -p v3-core --test viability` 28 passed before the prototype was committed;
  the prototype's 10 unit, coverage, serialization, boundary and proptest
  tests pass; the sink-use module's 5 unit tests and 4 fixtures pass. The
  binary hashes and the branch `make check` exit line are recorded under
  Gate results.
- **Landing scope.** Only `docs:` commits land on `main`; `proto:`, `instr:`
  and `lab:` stay on the branch, which is kept. T22 instrument candidates
  from this run (on the branch): the sink-use funnel and the memory-loop
  census beside the input funnel, the vote-delta reading and the births
  block.

## The prototype (W1's mechanism, `proto:`)

`mutation.born_connected_destinations: votes | non_clear` (default `votes`;
`policy-deviation` when `non_clear`; serialized only when `non_clear`, so the
`votes` config is byte-identical to `main`'s). `votes` **is production**
(T20.F16): on a Graph node `InputRef.Add` appends the reference and makes
three sampling calls in order, the sub-value `gen_range(0..width)` (`u16`),
the sink `gen_range(0..27)` (`usize`) over `VoteSink` by index, the weight
`gen_range(-1.0f32..=1.0)`, and pushes one edge onto that `ActionVote` sink;
the `votes` path is draw for draw that code (the same calls with the same
types). `non_clear`: the same event, the same three sampling calls in the
same order, with the second call `gen_range(0..78)` mapped by index
over the 78 sinks that use their input, in catalog order with the 16
`ClearSlot` sinks skipped: 0..24 `CustomOutput(0..24)`, 24..32
`RouterGate(0..8)`, 32..48 `WriteSlot(0..16)`, 48..75 `ActionVote` in
`VoteSink` index order, 75..78 `ActionParam` in `ActionParamField::ALL`
order. `ClearSlot` is excluded because it ignores its input (whatever is
wired to it, the slot is zeroed when the node runs; a sensor cannot be
carried), not for safety. The edge is pushed only when the index fits `u16`
and the node carries the drawn sink (a production node always does; a test
genome with a truncated catalog may not, and the event then stays a plain
declaration with the same draw stream: the truncated-catalog fixture pins
no edge, no reroll and the same next draw, finding 14). The event stays one
applied `InputRefAdd`; the operator definitions and weights, the
requested-supply law and the birth summary's accounting are unchanged;
within a fired birth the later events can differ between the two
destination classes (a range of 78 instead of 27 can consume a different
number of RNG words under rejection sampling, so the weight and every later
draw of that birth can differ even before a different sink joins the edge
pools: finding 13), so such births' applied and skip records are not each
other's; the `votes` path draws nothing extra and reproduces `main` birth
for birth. VM nodes are unchanged
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
destination class: the event accounting and the cost and supply **laws**
are the same, but the realized units need not be (finding 2): an edge into
an empty sink activates that sink, and `genome_size` counts the edge and
the newly wired sink, so a destination-dependent extra unit (more often
under `non_clear`, whose 51 non-vote sinks are mostly empty, than under
`votes`, whose 27 sinks the founder already wires) changes carrying cost,
gestation and later supply: a coupled confound the founder's W0 supply
equality does not control. What the mutation-off pairs cannot separate: the
destination class from the rest of the born mechanism, which is production
on both arms.

**Contracts it deviates from, stated.** The T20.F16 spec's Non-Goal "No
destination other than a vote sink of the same node, no compute-node edge",
its Non-Goal "No configuration switch ... or summary schema" and its
spec-owner decision "destination one uniform `ActionVote` sink, ... no
configuration switch" (the prototype adds a config field, which a feature
would either make production or remove: finding 16), the
[graph backend spec](../reference/v3-graph-backend-spec.md)'s born-edge
clause naming the vote sink, and the
[mutation spec](../reference/v3-mutation-spec.md)'s InputRef-domain `Add`
paragraph, which names the vote sink draw `gen_range(0..27)`; the T20.F02
[neutral-connection contract](../specs/roadmap/t20-f02-graph-neutral-connection-contract.md)'s
unsupported-destinations row (`ClearSlot`, `WriteSlot`, `CustomOutput`,
`ActionParam`, `RouterGate` outside the eligible set because a wired empty
sink can clear or overwrite incumbent data): F02 qualifies **zero-weight
neutral** recruitment and that qualification stays intact; a live-weight
born edge is outside its guarantee on every surface, and `non_clear` does
not qualify new neutral destinations, it lands a live edge where F02 names
the overwrite hazard. The realized effect (finding 15): the born edge adds
a random term to an occupied sink's weighted sum, or **activates** an empty
overwrite sink, which then writes the sensor's term where nothing was
written before; so harm per fired birth may rise (the births block reads it
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
| W0 | Mechanism identity and effect on the founder, through the E4 probe (`PETRI_E4_BIRTHS=1000000`; parents `founder` (a), `founder+declared` (b), `founder+declared+zero-edge` (c); no elites): the **frozen probe** (built from `main` without the prototype) once, the **prototype probe** with `votes` once and with `non_clear` (`PETRI_E4_WIDE=1`) once, plus a supplementary `non_clear` run at 10⁷ births for the destination-spread criterion | **`votes` path:** the prototype probe's `votes` output equals the frozen probe's in the **deterministic projection** (every field but `wall_secs` and the probe's config metadata) on all three parents, `genome_size`, `executed_resolved` and `frozen_age` compared first: `default` is byte-identical to T20.F16's engine. **`non_clear` path:** (1) every single-event target declaration arrives connected: on parent (a), among the single-event `InputRefAdd` births that declared the target on the vote node, `connect_any_given_declared` is exactly 1 and every `single_add_births.connect_by_location` key is `sink:<kind>` with no `ClearSlot` and no `compute:` key, every `connect_by_sub` key in 0..6; over all births `connect_any`'s CP interval contains `declared_vote_node_target`'s point. (2) Destinations spread over the 78 sinks by index, two different checks (finding 12): **coverage**, on the 10⁷ run the single-event destinations hit **all 78** non-clear sink keys (under uniform sampling at about 1,400 declarations a miss has probability about 10⁻⁶, so a miss is a mechanism failure); and **compatibility**, the share of `sink:ActionVote(…)` keys among them has a 95 % CP interval containing 27/78 = 0.3462 (at n ≈ 1,400 the interval is about ±2.5 % wide, a uniform mechanism fails it about 5 % of the time, and the power against a share of 0.33 is about 24 %: it is a compatibility check, not a proof of uniformity within or among kinds). **A lone CP failure** with coverage passing is investigated against the per-kind shares and the unit coverage test and recorded as a statistical failure with its p-value; the run is not rerun until it passes. Reported beside, descriptively, with n: the useful-signed share among single-`Add` declarations against 2/189 × 27/78 = 0.00366 and the per-kind shares against 24/78, 8/78, 16/78, 3/78. (3) Requested supply unchanged: `attempted_events_per_birth` on every parent equal between `votes` and `non_clear` (expected identical: the requested law is untouched). (4) `InputRefAdd` applied per birth within 5 % of the `votes` path's on every parent (not expected identical: later events of a fired birth can differ). (5) Integrated-loss equivalence: `lost_declaration` on (b) and (c) within 5 % of the `votes` path's, routes reported beside, with the born run's value-versus-occurrence limit. A failure of the `votes` identity, of (1) or of (2)'s coverage means the mechanism does not do what it says; a failure of (3), (4) or (5) beyond tolerance is investigated and explained; in every case W0 must pass after the disposition and a rerun before any world arm runs |
| W0.2 | Instrument identity: the census probe on the prototype binary at `votes` against (i) the launch-revision production binary `v3-cli-main` and (ii) the frozen instrument built without the prototype | (i) **Smoke:** a 2,000-tick run of the census probe (`default` recipe, seed 1022, checkpoints 1,000 and 2,000, every reading on) against `v3-cli-main --telemetry off run --ticks 2000 --sample-every 500 --seed 1022 --config default.json`: the sample tick sets equal (500, 1,000, 1,500, 2,000, both sides), every compared field present on both sides, `population`, `mean_genome_size`, `mean_generation` and `reproduction_actions_spawned_total` equal at every tick, both runs completed to 2,000, and the census header's `config_digest` equal to the run's `run_started` digest; `b02check.py` fails on any missing tick, field or completion record. The smoke blocks the pairs and the main arms. (ii) **Full comparison:** the frozen instrument runs `default` and `default-mutoff` on both seeds (20,000 ticks; the reference runs); the prototype binary's `default` and `default-mutoff` runs must equal them on every sample row (the fields above plus `mutation_events_applied_total` and `mean_energy`, each **required present**), the extinction tick, the footer (required present; `ticks` and `extinct`), and every scheduled checkpoint block (`input_use`, `e6`, `vote_deltas`, `births`, **`sink_use`, `memory_loop`, `memory_sensitivity`, `temporal_memory_sensitivity`**, `living`, `selected_parents`, `selected_generations`, `age_deciles`, `energy_deciles`; each required present, not null, at every reached checkpoint), each serialized once with sorted keys and compact separators and hashed; the expected sample ticks and checkpoints are derived from the header's horizon, sampling interval and checkpoint list and the run's own completion or extinction record, so matching omissions cannot pass (finding 5). The header is compared on every nested value, **including `config_digest` and the whole `prototypes` entry, which must be equal** (finding 6); only the recipe path and the run label may differ; the timing fields are excluded and listed. A full-run inequality blocks the `wide` arms from being read until explained and fixed |
| W0.3 | The mutation-off pair is identical by construction: with `per_unit_rate` 0 no event is requested, so no `Add` fires and the destination field is never read | `wide-mutoff` and `default-mutoff` on each seed: the same sample fields, extinction tick, footer and timing-free checkpoint blocks as W0.2 (ii) equal. The header's `config_digest` **differs** (the field serializes when `non_clear`) and `prototypes.born_connected_destinations` is `votes` against `non_clear`; both are verified, every other nested value of `prototypes` and every other header key but the recipe path and the run label must be equal (finding 6). The births block's `mutation_config` entry is compared apart (every key but the destination field equal). The block comparison is vacuous only when both runs are **verified extinct before the first scheduled checkpoint** (the extinction record on both, no checkpoint row on either); any other absence fails (finding 5). An inequality is a defect in the prototype's `votes` path and blocks the `wide` arms |

**Gate results.** **Binaries.** Reference `v3-cli-main` built at `e847ec8b`
(sha256 in `v3-cli-main.sha256`); frozen `census-frozen` `fb23883a…` and
`probe-frozen` `df8cd836…` built at the frozen revision `d2100d8b`;
prototype `census-proto` `baa57aaf…` and `probe-proto` `f2ae3e8e…` built at
`06623dc4` (the branch head, docs commits after the last code commit); each
binary lists its test. **`make check` on the branch exited 0** at `06623dc4`
(log `make-check-branch.log`; the `v3-cli` recipe-digest pin test included,
since the `votes` field is not serialized); `make check-docs` exited 0 on
the branch; the driver started only on that exit line. **W0 passed** (20 of
20 checks; summary [`w0.json`](output-wiring-run-2026-10/w0.json), reader
[`w0read.py`](output-wiring-run-2026-10/w0read.py); raw `w0/e4-frozen.json`
sha256 `32976206…`, `e4-votes.json` `97a5e66f…`, `e4-wide.json`
`24a23e71…`, `e4-wide-10m.json` `0fa205ca…`; 10⁶ births per parent in 2.1
to 2.6 s on every binary, 10⁷ in 20 to 22 s). **`votes` path:** the
prototype probe's output equals the frozen probe's in the deterministic
projection on all three parents, `genome_size`, `executed_resolved` and
`frozen_age` included: `default` is byte-identical to T20.F16's engine on
the founder (declared 194 of 10⁶, every one connected, useful signed 6 of
10⁶, attempted events 0.484315 per birth, as the born run's on path read).
**`non_clear` path.** (1) On the founder, 15,120 of 10⁶ births were
single-event `InputRefAdd` births and 132 of them declared `AreaFoodSummary(0)`
on the vote node; every one of the 132 carried the born edge
(`connect_any_given_declared` 1.0, CP 0.972 to 1), every destination a
`sink:` key of a non-clear kind, every sub-value in 0..6; over all births
`connect_any` equals `declared_vote_node_target` exactly (194 = 194). (2)
At 10⁷, 1,394 single-`Add` declarations hit **all 78** non-clear sinks
(coverage), and the `ActionVote` share 500 of 1,394 = 0.3587 (CP 0.3335 to
0.3845) contains 27/78 = 0.3462 (compatibility); per kind, descriptively,
`CustomOutput` 416 (0.298 against 24/78 = 0.308), `RouterGate` 128 (0.092
against 0.103), `WriteSlot` 308 (0.221 against 0.205), `ActionParam` 42
(0.030 against 0.038), every CP interval containing its predicted share;
the useful-signed share among single-`Add` declarations 7 of 1,394 = 0.0050
(CP 0.0020 to 0.0103) contains 2/189 × 27/78 = 0.0037. (3) The requested
supply is identical between `votes` and `non_clear` on every parent
(0.484315, 0.489357, 0.495378). (4) `InputRefAdd` applied per birth
0.021549 against 0.021533, 0.021520 against 0.021513, 0.021994 against
0.021970 (within 0.2 %). (5) Integrated loss 0.018711 against 0.018691 on
(b) and 0.007907 against 0.007909 on (c) (within 0.2 %); the operator-class
tallies differ as a fired birth's later draws differ (on (b)
`InputRefAdd+InputRefPrune` 208 against 197, `InputRefAdd+InputRefRawFieldMutation`
87 against 71). **Census smoke passed** (`smoke/small.json`, the 200 × 200
Canyon recipe with 400 founders, seed 33, 300 ticks, checkpoint 300, every
reading on, the prototype binary at `votes`): exit 0 in 5 s, 148 living at
the checkpoint, census 0.21 s, of which the sink-use funnel 0.09 s (S = 17
or 18 wired sinks on every parent: the founder's catalog), the memory
readings 0.004 s, vote deltas 0.006 s, births 0.05 s; on the founder-scale
cohort `CustomOutput` is wired on 20 parents and causal on 19 under both
the per-slot and the joint reading, `ActionParam` wired on 20 and causal on
18, `ActionVote` wired on 20 and causal on 19, `RouterGate`, `WriteSlot` and
`ClearSlot` unwired; no memory loop (3 parents with a VM write, no read);
`memory_sensitivity` 0 of 148. So the founder's own custom outputs and
action parameters are causal on the panels: resolution (ii)'s
by-construction failure is expected on those two kinds, and the clause rests
on `RouterGate` and `WriteSlot`. The cost formula with S = 17 gives 20 × 264
× 24 ≈ 127,000 executions in 0.09 s; at a 20,000-tick genome's S the block
is projected at seconds per checkpoint (read at the first checkpoint).
**W0.2 (i) smoke passed** (7 of 7 checks,
[`w02-smoke.json`](output-wiring-run-2026-10/w02-smoke.json), checker
[`b02check.py`](output-wiring-run-2026-10/b02check.py)): the prototype
binary's census run (`g1-default-s1022`, 2,000 ticks, 8 threads, 75 s) and
`v3-cli-main`'s run (80 s) carry the same four sample ticks, every compared
field present and equal at every tick, both completed, and the census
header's `config_digest` equals the run's `run_started` digest. **Frozen
references** (`census-frozen`, 8 threads): `default-mutoff` extinct at 5,557
(seed 1022) and 7,509 (seed 2022) in 111 s and 126 s; `default` complete at
20,000 in 742 s and 657 s. **W0.3 passed on both seeds**
([`w03-s1022.json`](output-wiring-run-2026-10/w03-s1022.json),
[`w03-s2022.json`](output-wiring-run-2026-10/w03-s2022.json), checker
[`identity.py`](output-wiring-run-2026-10/identity.py)): `wide-mutoff`
equals `default-mutoff` on every sample field at every expected tick (11 and
15 samples), the extinction tick (5,557 and 7,509), the footer, and the
checkpoint sets (none reached: both pairs die in the founder trough before
10,000, verified extinct before the first checkpoint on both runs, so the
block comparison is vacuous by the strict rule), the header differing only
in `config_digest`, `prototypes.born_connected_destinations` (`votes`
against `non_clear`, every other `prototypes` value equal), `recipe` and
`world`. **W0.2 (ii) passed on both seeds for both control arms**
([`w02-mutoff-s1022.json`](output-wiring-run-2026-10/w02-mutoff-s1022.json),
[`w02-mutoff-s2022.json`](output-wiring-run-2026-10/w02-mutoff-s2022.json),
[`w02-default-s1022.json`](output-wiring-run-2026-10/w02-default-s1022.json),
[`w02-default-s2022.json`](output-wiring-run-2026-10/w02-default-s2022.json)):
the prototype binary's `default-mutoff` runs (100 s and 121 s) and `default`
runs (698 s and 662 s) equal the frozen instrument's on every sample field
at every expected tick (40 samples for `default`), the extinction tick, the
footer, every scheduled timing-free checkpoint block at 10,000 and 20,000
(`input_use`, `e6`, `vote_deltas`, `births`, `sink_use`, `memory_loop`,
`memory_sensitivity`, `temporal_memory_sensitivity`, the cohort and decile
fields, every block present) by hash, and the header entirely
(`config_digest` and `prototypes` included): the prototype binary at `votes`
is the instrument built without the prototype, reading for reading, and
the `wide` arms may be read. Every launch from the frozen references on was
on AC (81 % charging at the first, 100 % by the pairs). **Block costs at
20,000-tick genomes** (`default`, both seeds, per checkpoint): the sink-use
funnel 0.30 to 0.60 s, the memory readings 0.36 to 0.63 s over 2,685 to
3,065 living, vote deltas 0.05 to 0.07 s, births 0.27 to 0.63 s, the whole
census 1.3 to 2.8 s. **Reproduction of the born run:** this run's `default`
(T20.F16 production on `main` `e847ec8b`) reproduces the born run's `born`
arm reading for reading on both seeds (window-mean population 2,876 and
2,681, genome size 5.52 × and 6.08 ×, the typed funnel rows 10/9/9/3 and
8/6/5/5 at 20,000 on seed 1022, 6/6/6/2 and 7/7/7/4 on seed 2022, the
action-changing target channels 8 and 6), as the feature's draw-for-draw
re-expression of the prototype predicts; the control pairs die at the born
run's ticks (5,557 and 7,509).

## Ledger

One row. No diagnostic (authored-genome) arm is used, and no authored genome
enters the natural-world arms; W0's prepared parents are measurements, not
mechanisms. A killed run is `incomplete`, never a biological negative; an
extinction is a finding.

| ID | Question | Method, instrument, sizes | Prediction / decision rule | Status | Result |
| --- | --- | --- | --- | --- | --- |
| W1 | Which output surfaces are wired, executed and causal on a natural world's selected cohort, does a memory loop become causal, does memory use appear by 10,000 or 20,000 ticks, and do sensors born into any input-using surface (`non_clear`) become causal there without costing persistence or more than two thirds of the vote-side gain? | The prototype above (`proto:`, `mutation.born_connected_destinations`, default `votes`, `policy-deviation` at `non_clear`). **Tested on the founder at unit level before any gate:** `votes`, `Add` on a Graph node is draw for draw production (one reference, one `ActionVote` edge, same draws); `non_clear`, `Add` on a Graph node appends one reference and exactly one edge on a non-`ClearSlot` sink, sourced from the appended index at a sub-value under the reference's width, weight in [−1, 1], compute nodes and the other node untouched; `non_clear` on a VM node appends one reference and leaves the backend unchanged; the field reaches `Add` through the config, `votes` is the default, absent from the serialized config at `votes` and round-trips at `non_clear`; over 4,096 seeds the `non_clear` edge reaches all 78 sinks with the vote share near 27/78; a proptest over generated genomes holds the one-reference, at-most-one-edge, never-`ClearSlot`, source and untouched-node invariants. **Design as B1:** Canyon goal recipe (`experiments/worlds/canyon-country.json`, jq overlays), seeds 1022 and 2022, 20,000 ticks, samples every 500, every arm on both seeds, through the census probe with checkpoints 10,000 and 20,000 (selected cohort of 20 parents, founder cohort, drift empty; the input funnel, the vote-delta reading, the births block, the sink-use funnel, the memory-loop census, the memory sensitivities and E6 at both), 3 h awake kill per run, one job at a time. Arms: `default` (`votes`, production), `wide` (`non_clear`), `default-mutoff` and `wide-mutoff` (`per_unit_rate` 0: identical by construction, gate W0.3). **Readings:** window means over ticks 15,000 to 20,000 (population, genome size in founder units (97), births per creature-tick, extinction interval); the typed target-family funnel rows (`AreaFoodSummary:0`, `AreaFoodSummary:1`, `NeighborBarrierRing`: declared / connected / executed / causal, distinct parents, `parents_evaluated` 20) at both checkpoints; the vote-delta block at both (executed target channels, channels whose ablation changes an action queue, channels whose ablation moves any vote); the **agreement check** (per typed family, the funnel's `family_causal` against the distinct parents with an action-changing vote-delta channel of that family; per channel row likewise), which blocks the interpretation of the causal and vote-delta readings until it holds; the **sink-use funnel** per kind and per (kind, slot) (wired / executed / causal parents) at both; the **memory-loop census** (slots written, read, loops, loops executed, loops causal, dynamic reads and writes) at both; **`memory_sensitivity`** (zeroed, scrambled, either, over all living) and the three temporal components at both; dead / silent / changed per birth from the births block, descriptive | **Prediction, with a direction or its absence for every reading; every prediction is read at 20,000 unless it names 10,000, and every reading not listed here (vote-moving channels, `ClearSlot` rows, `wide`'s wired and executed sink stages, executed loops, dynamic reads and writes, slots written and read, the inherited retention readings, the temporal components under `wide`) has no directional prediction at either checkpoint (finding 18).** `wide` persists on both seeds; its window-mean population **below** `default`'s but above 0.5 × (a born edge adds a random term to an occupied overwrite sink's sum or activates an empty one, so a fired `Add` on a Graph node can now write where nothing was written; E5 found whole-family wiring harmful 4 of 5; structural connection does not guarantee a behavior change); genome size **not below** `default`'s; births per creature-tick: **no direction**; extinction: none predicted; dead, silent, changed per birth: **no direction** (underpowered and battery-limited, as B1 found). Input funnel typed rows at 20,000, descriptively: `family_declared` **not reliably different** (both arms protect a connected declaration from pruning), `family_connected` tracks `family_declared` on both arms, `family_executed` and `family_causal` on the vote surface **down** under `wide` (27 of 78 born edges reach a vote), about one third of `default`'s, with the vote-delta's action-changing target channels **down** in step. **Sink-use funnel** at 20,000: under `default`, `WriteSlot` and `CustomOutput` wired on most parents, executed on most, causal on **few or none**; `RouterGate` and `ActionParam` wired on fewer, causal on none; whether any non-vote kind becomes causal by 20,000 where it is not at 10,000 is the horizon reading; under `wide`, **at least one causal incidence on `WriteSlot` or `CustomOutput` where `default` has none** (the row's clause). **Memory-loop census:** loops present on both arms at both checkpoints, loops causal **0 at 10,000** on both arms; at 20,000 `wide` **not below** `default`. **`memory_sensitivity` on `default`:** the plan leaves it open; this run predicts **0 of all living on both seeds at both checkpoints** (the live survey's quarter came at 281,000 ticks on an older substrate), with the temporal components near the T20.F16 goal's levels (operator state under 1 %, persisted outputs about 1 %, previous slots 0); `wide`: not predicted. E6: **no prediction**. The validation reads 20,000; the 10,000 checkpoint is descriptive. **Validation rule, verbatim from the plan's draft rule, which the goal command fixes as written:** "`wide` is **validated as a lead** if on both seeds at 20,000: `wide` persists; its window-mean population is at least 0.5 × `default`'s; the sink-use funnel shows at least one causal incidence on a non-vote sink where `default` has none; and the vote-side target causal count is not below one third of `default`'s. **The horizon question is answered apart**, on `default` alone: `memory_sensitivity` above 0 at 20,000 on either seed means memory use is a matter of horizon and T11.F10 should read later checkpoints; 0 on both seeds with loops present means the block is selection or closure, and option C or a temporal assay comes next. A 20-parent cohort is a lead-selection screen; one lineage can carry a count." **Resolutions, predeclared:** (i) "on both seeds" governs every clause after it; each clause is read on each seed. (ii) "A causal incidence on a non-vote sink where `default` has none" is read **per sink kind** among `CustomOutput`, `RouterGate`, `WriteSlot` and `ActionParam` (`ClearSlot` cannot carry a sensor and is reported only) on the selected cohort at 20,000, by the **joint kind-ablation** count `parents_causal_kind` alone (the per-slot reading is descriptive and cannot override it; a disagreement is recorded): some such kind with `parents_causal_kind` ≥ 1 under `wide` and 0 under `default` on that seed. If `default` has at least one causal parent on **every** one of the four kinds on a seed, the clause fails on that seed by construction; that is recorded as a finding about the rule, with the per-kind and per-slot comparison (`wide` against `default`, numerators and distinct parents) beside it, and the row is not validated by this rule. (iii) "The vote-side target causal count" is Σ `family_causal` (distinct parents) over the three typed target rows on the selected cohort at 20,000; the clause holds when `wide`'s is at least one third of `default`'s, and trivially when `default`'s is 0. **Named for what it measures (finding 1):** `family_causal` is an action change under input-channel ablation through **any** surface (N3 found causal reads with zero vote delta), so this is an all-surface target-input causal proxy, not a measure of the vote-side gain; the rule keeps the number as the command fixes it, the note claims no vote-side preservation from it, and the vote-delta block's vote-moving and action-changing target channels are reported beside it as the vote-surface reading. (iv) If `default` is extinct before 20,000 or has no living sample in the window on a seed, the population ratio and the 20,000 comparison are undefined on that seed and the row is not validated by this rule, with `wide` persisting where `default` died recorded as a finding (the born run's `default` had 795 and 815 living at its trough on these seeds, and its `born` arm 10 living at tick 4,500 on seed 2022; `born` is now production, so the default's trough on these seeds is live). (v) `wide` extinct on either seed: not validated, an extinction finding. (vi) The horizon question: "`memory_sensitivity` above 0" is `different_from_either_count` ≥ 1 over all living creatures at 20,000 on the `default` arm on either seed; the zero branch ("0 on both seeds with loops present") requires a valid all-living sensitivity block at 20,000 on `default` on **both** seeds (the run complete to 20,000, `parents_evaluated` 20) and at least one parent with a loop in the memory-loop census at 20,000 on **both** seeds; any other combination (a missing checkpoint, `default` extinct, fewer than 20 parents evaluated, zero sensitivity on both seeds with loops on one seed only) is recorded as **inconclusive**, never as a biological zero or a selection-or-closure verdict (finding 7); the three temporal components are reported beside, not read by the rule. (vii) A 20-parent cohort is a lead-selection screen: one parent can carry the incidence, numerators and direction are reported, and no population-wide increase is claimed. (viii) The funnel's `causal` and the vote-delta's action-queue change are one fact measured twice; the agreement check must hold for the vote-side clause to be read. **Alternative explanations named now:** the vote-side dilution itself (27 of 78) lowering selection pressure on the whole mechanism; the overwrite harm changing which lineages survive; the destination-dependent extra unit (sink activation) changing cost, gestation and supply; a causal non-vote incidence that is one lineage's; drift and shared ancestry within a 20-parent sample; the cohorts' backend composition (VM births are untouched by the mechanism) and generation counts, which differ between arms and are reported from the census (`reachable_graph_nodes`, `selected_generations`) as the denominators of exposure (finding 19); checkpoint sampling swings (B1's 8 → 3); the panels' exposure (the battery presents zero for the target families; the recorded and authored contexts carry the causal stage); the sink ablation's intervention effect on dispatch (clearing a `RouterGate` or `CustomOutput` changes which successor runs, so a causal incidence is the total effect, not a sensor's contribution); no edge provenance in the census (a causal non-vote sink under `wide` may carry an ordinary `AddGraphEdge` edge, as under `default`; the mechanism's fingerprint is `family_connected` tracking `family_declared` and the destination kinds the born edges can reach). **Not validated otherwise.** A kill marks the run `incomplete` and it is rerun once at 6 h | **predeclared** | |

## Advice

Every Fable advisor consultation rule 8 lists, with the disposition of each
point raised.

| # | Rule 8 point | Advice (summary) | Dispositions |
| --- | --- | --- | --- |
| 2 | Before the gates and the world chain (the phase's approach), after amendment A1, the `instr:`, `lab:` and `proto:` commits, the frozen tree and the readers; the first call returned an overload error and the second succeeded | Chain correctly sequenced and the A1 fixes right; six points. (1) `make check-docs` on the branch now: the note links a summaries folder that does not exist; create it with the readers (closing work anyway). (2) When the frozen and proto builds finish, check both hashes files have two lines and both binaries list their test (the first frozen attempt silently produced only the probe). (3) Record this consultation and fill the Run setup's branch commits, viability result and the frozen revision's composition before W0 starts. (4) Remove the nested frozen-tree worktree (and prune) before `ExitWorktree`, once the binaries are hashed. (5) Turn economy: three waiters (W0, the identity gates, DONE/STOP), three note commits, draft the plan-forward skeleton while `wide` runs. (6) Watch, not change: a `votes` projection failure at W0 is a build or tree question, rebuild and rerun; a lone CP failure with coverage passing is the predeclared statistical case; at the smoke read `sink_use_secs` and S and project with S from 20,000-tick genomes; at the W1 reading lead with the per-kind `parents_causal_kind` table, since the by-construction failure of (ii) is decided there first. Retracted from Advice 1: the sed-rename suggestion (fresh scripts were right; the born script's cargo path would have rebuilt mid-chain) | (1) The folder created with `identity.py`, `b02check.py`, `w0read.py`, `w1read.py` and `make check-docs` run on the branch (exit recorded in Gate results); (2) checked at both wakeups; (3) this commit (Run setup: branch commits, viability, frozen revision); (4) and (5) in the close order and the waiters; (6) read at the wakeups |
| 1 | Before anything was built: the approach to the run after the contract, the branch code and the engine were read | Shape right; ten points. (1) The base moved: T20.F16 is production, so take only the census probe, the vote-delta reading and the E4 probe as file state, rework the flag semantics (`born_connected_destinations` through the serialized config, `PETRI_E4_WIDE`), and make W0's `votes` identity the frozen probe against the prototype probe, not the born run's raw output; check the draw-path diff first. (2) Read `recruitment_paths` on main for born-edge assumptions before writing the proto; predeclare the 78-index map, `votes` draw for draw production, `non_clear` one `gen_range(0..78)` in the sink's place, `skip_serializing_if` so the default digest is main's. (3) The sink ablation is clearing the sink's edges, not the funnel's `ablated`; record executed nodes through a `MeshExecutionMode` that keeps `node_idx`; report per kind and per (kind, slot) with `ActionVote` as a comparison row. (4) The memory read side has no causal instrument: predeclare loop-causal as the write sink causal under the sink ablation with a read of slot k present. (5) `memory_sensitivity` and the temporal readings are reachable from the census (`pub(super)` in `bench::indicators`); run them over all living at both checkpoints; record the final-action limit. (6) Fix the rule's resolutions: per sink kind; `default` likely has causal `WriteSlot` and `CustomOutput` already, so predeclare the by-construction failure as a finding about the rule; define the vote-side count as Σ `family_causal`, trivially true when the default's is 0; carry B1's resolutions; the default's trough on seed 2022 is live. (7) Grow W0.2's hash list with the new blocks; keep the pairs' check non-vacuous; carry the B0.2 (i) smoke against a `v3-cli` built at `e847ec8b`. (8) State the deviated contracts: the T20.F16 spec Non-Goal, T20.F02's unsupported destinations, the mutation spec's `Add` paragraph; the born run's three sentences are amended on main. (9) Battery: say it once, record power at every launch, builds and W0 first. (10) Turn economy: copy the born run's scripts with `sed` renames, one waiter per milestone, batched commits, the note in one write; the Codex-takes-every-role memory is stale; confirm the head, `npm ci`, viability first | All adopted in this predeclaration: (1) Run setup (code taken, draw-path drift) and gate W0; (2) the prototype section (index map, the recruitment-path fixtures) ; (3) and (4) the instrument extension; (5) block 3; (6) resolutions (ii) to (vi); (7) gates W0.2 and W0.3; (8) "Contracts it deviates from"; (9) Run setup (host) and the Order; (10) the scripts under `.bench-artifacts/lab/wiring/`, the Order |

## Codex reviews

### Review (a): the predeclaration, before any gate or row ran

Job `.bench-artifacts/lab/wiring/codex/review-a.out.md` (brief `review-a.md`),
verdict `not-ready`, 9 blocking and 10 advisory. Every finding is adopted as
**amendment A1** (the sections above carry the fixes, each marked with its
finding number); no confirmation round is run, since no finding changes the
row's question, its arms or the user's validation rule, and every fix is a
predeclared wording, a gate rule, a checker rule or an instrument change
tested on the branch before any gate runs.

| # | Finding (short) | Severity | Disposition |
| --- | --- | --- | --- |
| 1 | `family_causal` is an any-surface action change; it cannot establish vote-side preservation | blocking | Adopted: resolution (iii) names it an all-surface target-input causal proxy, keeps the number, claims no vote-side preservation, reports the vote-delta block beside |
| 2 | "Nothing else changes": sink activation adds a unit, so cost, gestation and supply differ by destination | blocking | Adopted: not-forcing check (5) and the alternatives name the coupled confound |
| 3 | Joint kind ablation and the union of causal slots can disagree | blocking | Adopted: the joint count `parents_causal_kind` alone decides (ii); per-slot descriptive; disagreements recorded |
| 4 | Clearing the last wired sink can disable a charged visit; loop causal can be an energy effect | blocking | Adopted: the caveat stated; loop causal labelled a write-ablation-plus-read-wiring proxy; computation unchanged |
| 5 | `identity.py` accepts matching omissions and an unverified vacuous case | blocking | Adopted: the checker requires headers, footers, fields and scheduled blocks, derives expected ticks from completion or extinction, vacuity only after verified pre-checkpoint extinction on both |
| 6 | Blanket header exclusions weaken the `votes` identity | blocking | Adopted: under W0.2 (ii) `config_digest` and the whole `prototypes` entry must be equal; under W0.3 only the destination leaf and the digest may differ; W0's projection drops `wall_secs` only |
| 7 | The horizon rule lacks outcomes for missing checkpoints, extinction, fewer parents, loops on one seed | blocking | Adopted: resolution (vi) requires valid blocks on both seeds and loops on both for the zero branch; everything else inconclusive |
| 8 | E6 at 20,000 only contradicts "every reading on at both" | blocking | Adopted: E6 at every checkpoint in the census (`instr:`), hashed under W0.2 (ii) |
| 9 | The unit tests validate the prototype, not the instruments | blocking | Adopted: authored fixtures for sink clearing semantics, Graph and VM loop classification with boundaries, purity; the joint-versus-individual case recorded descriptively |
| 10 | A node-index record is dispatch, not committed effects; VM loops' execution stage unclear | advisory | Adopted: the stage is a dispatch record; VM-only loops are in the structural stages and never loop-causal |
| 11 | VM immediates wrap modulo 16; Graph sources past 15 resolve to zero | advisory | Adopted: both stated and fixtured; the Graph read past slot 15 rejected in code |
| 12 | Coverage and CP containment are different checks; false-failure 4.6 %, power 24 % | advisory | Adopted: W0 (2) separates them, records n and limits, predeclares the lone-failure disposition |
| 13 | Three sampling calls, not three RNG words; rejection differs by range; keep production's types | advisory | Adopted: the prototype paragraph; types named; the `votes` draw-for-draw test pins the production path |
| 14 | The truncated-catalog fallback is not pinned for `non_clear` | advisory | Adopted: a boundary test pins no edge, no reroll and the same next draw |
| 15 | "Replaces incumbent data" overstates: an occupied sink gains a term; an empty sink is activated | advisory | Adopted: reworded in the contracts section and the prediction |
| 16 | The list omits F16's no-switch decision and the graph-backend clause; F02's neutral qualification is not broadened | advisory | Adopted: both named; F02 stated intact with `non_clear` outside its guarantee |
| 17 | The new workload's cost is unestimated | advisory | Adopted: the formulas stated; actual S, living counts and wall times recorded; the smoke called a lower bound |
| 18 | Several readings lack a direction or its absence; checkpoint scope unclear | advisory | Adopted: the prediction's preamble lists the unlisted readings as undirected and reads at 20,000 unless named |
| 19 | The premise overreaches to unmeasured surfaces; backend composition and generation missing from the alternatives | advisory | Adopted: the Question restricted to memory; the alternatives name composition and generation with their census denominators |

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
