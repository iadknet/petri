# T27.F03 — Giving Energy

**Status**: In Progress
**Last updated**: 2026-10-21
**Feature**: T27.F03
**Track**: [T27 — Social Signals and Exchange](../../roadmaps/t27-social-signals-and-exchange.md)

## Goal

Trophallaxis and parental feeding: a creature can pass a mouthful of energy
to the neighbour in a chosen direction, and stealing takes a mouthful the
same way, so care and exploitation are both possible and the simulation
assigns neither a meaning. `GiveEnergy` is a fourth directed vote kind (a
drive and eight direction sinks) whose commit moves one fixed bite,
`exchange.bite`, from the giver to the creature on the target cell:
lossless, the recipient credited through T26.F01's cap rule with the excess
booked as clamp loss, the gift booked as paired ledger flows. `StealEnergy`
moves the same bite from the victim, its `StealEnergyAmount` parameter is
retired, and a zero transfer is a failure for both. The track row's
directed-kind catalog refactor (`VoteSink::Directed`, a `voteSinks.json`
fixture, charge arrays indexed by action type) is folded in, and the
zero-transfer fix lands as its own commit.

## Non-Goals

- Any meaning, kin test or recipient selection inside the action: who
  receives is the brain's choice through the direction sinks; parental care
  is the expected first use (track note), not a target.
- A give amount parameter (design note §3 C: an unwired parameter reads 0,
  the live steal finding); any give cost beyond the effort floor.
- Held signals (T27.F02), carrying (T28.F01), the cap or reserves (T26.F02),
  the kill rule and its bonus (unchanged).
- A gift event record or overlay, per-creature gift counters, a recipient
  classification (readings file, options); renaming the `predation` stats,
  block or families, which keep their names so the series stay continuous.

## Inputs and Invariants

| Input | Where | What it fixes |
| --- | --- | --- |
| Roadmap row and Notes | `docs/roadmaps/t27-social-signals-and-exchange.md` F03 row; Notes "F03 folds in the directed-kind catalog refactor", "F02 depends on F03", "F02 and F03 are creature-authored pressures", "Verification: F03 viability first, gate + goal" | Scope, the folded refactor, the separate zero-transfer commit, the bite payoff predeclaration, the profiles. |
| Design note | `docs/strategy/interaction-features-design-2026-10-20.md` §2 (actions, energy), §3 C, §4 (what folds into C, the T28.F01 ordering) | The shape, the precedent and the collision order. |
| Design review | `docs/progress/readings/t26-t28-design-review.md` findings 4 (bite payoff) and 8 (F02 after F03) | Why the bite is chosen against targeting economics. |
| Payoff, options and audit | [`docs/progress/readings/t27-f03.md`](../../progress/readings/t27-f03.md) | The bite-size table, the alternatives weighed and the refactor table behind the `Decision:` notes. |
| T26.F01 | spec invariants 5 and 6: `cap_energy` in `simulation/actions/mod.rs` books `maximum_energy_clamp_loss`; `pay_effort` / `charge_effort`; `CreatureState::energy_cap` | The credit path a gift and a steal use. |
| Catalog today | `creature/genome/vote.rs` (`VOTE_SINK_COUNT` 30, `VOTE_KIND_COUNT` 4, `ACTION_PARAM_FIELD_COUNT` 4, three directed variants), `creature/genome/cgp/mod.rs:101-147` (`OutputSinkKind`, `FIRST_ACTION_VOTE_SINK` 64, `FIXED_SINK_COUNT` 98), `runtime/vote_select.rs`, `runtime/action_decode.rs`, `mutation/types/mod.rs:91-104` (`GroupClass::of_sink`), `mutation/graph/group.rs:132,170,235` (kind draws), `mutation/vm/operators.rs:230,279` (field and sink draws), `mutation/input_ref/mod.rs:349`, `mutation/compound.rs:23` (vote widths) | The one catalog and every draw over it. |
| Steal today | `simulation/actions/predation.rs`, `simulation/tick.rs:1098-1130` (`execute_steal_energy`), `contracts/actions.rs` (`WorldAction::StealEnergy { direction, amount }`, `action_type`, `param`), `config/simulation.rs:910-926` (`PredationConfig`) | The path the bite rule replaces. |
| Ledger | `simulation/energy_accounting.rs` (`DeathCause::ALL` 15, `ActionCharges`, `FailedActionEffort`, `EnergyFlows`, `net_action_overdraft`), `creature/action_log.rs` (`ActionType` 5, `ActionResult` 9 variants), `simulation/stats.rs:438-454`, `simulation/outcomes.rs` | Where a transfer is booked and logged. |
| Exports | `v3-cli/src/bench/tracking.rs` (`PredationTracking`, `ActionChargeTracking`, `FailedActionEffortTracking`, `EnergyFlowTracking` `applied-energy-flows-v3`, `by_cause`), `v3-cli/src/bench/drives.rs`, `v3-telemetry/src/metrics.rs:556-562,646-662,902-927,1159-1165`, `metrics/tests.rs`, `trace.rs:342-418` (`actions` phase attributes), `scripts/telemetry-dashboards-check`, `v3-server/src/state.rs:100-126,222-226`, `v3-server/src/types.rs` (`v3alpha12`), `v3-lab/src/summary.rs` (`GENOME_FORMAT` 7), `neighborhood/mutation_effects/mod.rs:45` (`neighborhood-coverage-v3`), `neighborhood/input_use/mod.rs:48` (`input-use-v3`) | Every surface that names a kind, a charge or a flow. |
| Frontend | `components/inspector/voteSinks.ts`, `types/trace.ts:146-158`, `types/genome.ts:84-125`, `types/action-log.ts`, `edgeGroupWeights.ts:36-42`, `graphNodeFormatters.ts:151-230`, `inputRefUtils.ts:15-25,151-153`, `vmInstructionFormat.ts:167`, `VitalsBanner.tsx:12`, `stats/statLabels.ts:70`; the `nearbyFields.json` pattern (`contracts/inputs.rs:583-600`) | The hand mirrors the fixture replaces and the pin pattern. |
| Pins | `tests/viability.rs` founder-only digests (must not move), `tests/applied_trajectory.rs`, `tests/baseline_worlds.rs`, `tests/reproducibility.rs`, `tests/recruitment_paths.rs` and the `--lib` rows, `mutation/graph/recruitment_tests.rs`, `creature/genome/mod.rs:900-940` (serde shapes), `census.rs:160-174`, the v3-cli trajectory fixtures, the gate and goal deterministic blocks, `docs/progress/benchmark-series.json` epochs | The predeclared re-pins (Performance section). |
| Reference specs | `v3-graph-backend-spec.md:60-66,180-205,240-260`; `v3-mesh-execution-spec.md:52-112`; `v3-vm-isa-spec.md:90,99,366-384`; `v3-mutation-spec.md:419-426,657-662`; `v3-sensor-spec.md:224` (`CommitCounts`) and the `ActionVotes` rows; `v3-tick-orchestration-spec.md:217,254-259`; `v3-runtime-config-spec.md:276-290` and the `predation` section; `v3-server-api-protocol-spec.md` §2, `:685`, `:796` | Every sentence that names the catalog sizes, the steal amount or the rate. |

Invariants:

1. **One directed shape.** `VoteSink` is `Eat`, `Directed(DirectedVoteKind, u8)`,
   `Drive(DirectedVoteKind)`, `Terminate`, `Decide`. With `N =
   DirectedVoteKind::ALL.len()`: `Eat` 0, `Directed(K, d)` `1 + 8·K.index() + d`,
   `Drive(K)` `1 + 8·N + K.index()`, `Terminate` `VOTE_SINK_COUNT − 2`,
   `Decide` `VOTE_SINK_COUNT − 1`, `VOTE_SINK_COUNT = 3 + 9·N`. `VoteKind::ALL`
   is `Eat` followed by the directed kinds' vote kinds in `ALL` order (a test
   pins it). `DirectedVoteKind::sink`/`of_sink`, `VoteSink::kind`,
   `GroupClass::of_sink`, `OutputSinkKind::direction` and `vote_select` have
   one arm for the directed case and never name a kind; `decode_commit`
   extracts the kind and direction once and keeps one arm per kind for the
   action it builds. Serde: `{"Directed":["Move",3]}`; drives keep
   `{"Drive":"Move"}`. A new kind touches `DirectedVoteKind`, `VoteKind`,
   `ActionType`, `WorldAction`, a decoder arm and its action, never the
   index arithmetic.
2. **One fixture.** `frontend/src/components/inspector/voteSinks.json` holds
   `voteKinds`, `directedKinds`, `voteSinks` (the serde value of every sink in
   index order), `actionParamFields` (`{ field, kind }` in catalog order) and
   `actionTypes` (discriminant order, with codes); a Rust test in `vote.rs`
   pins it to the declarations, as `nearbyFields.json` is pinned. The
   frontend derives its sink index functions, counts, kind and field lists
   and action-code strings from it; no hand arithmetic or list of the
   catalog remains in TypeScript, and the `inputRefUtils.test.ts` parity
   pattern covers the new fixture.
3. **Charge arrays.** `ActionCharges` and `FailedActionEffort` are
   `[f64; ACTION_TYPE_COUNT]` newtypes indexed by `ActionType` (declared in
   `contracts/actions.rs`, re-exported by `creature::action_log`), with
   `ActionType::flow_key()` (`noop`, `eat`, `move`, `reproduce`,
   `steal_energy`, `give_energy`); `net_action_overdraft`, the v3-cli
   `energy_flows` export (`BTreeMap`s keyed by flow key, definition
   `applied-energy-flows-v4`) and the telemetry `action_charge.<key>` /
   `failed_action_effort.<key>` points and the creature-window
   `FlowMarks::of` charge sum (`tick/window_seam.rs`) iterate
   `ActionType::ALL`; the NoOp failed slot stays 0 and is exported like the
   others; the netting property test samples every `ActionType` row
   (conservation, `failed ≤ charge`, the NoOp failed slot zero).
   `FlowMarks`' reward sum excludes `gift_credit`: the credit is the
   recipient's, not the acting creature's.
   `WorldAction::action_type()` returns `ActionType` (`as u8` is the queue
   code). `apply_steal_energy` and `apply_give_energy` each return their
   result with the amount moved and the counterpart; the tick's executors
   log the amount from it and no longer read `last_tick_predation_events`.
4. **The bite.** `SimulationConfig.exchange: ExchangeConfig { bite: f32 }`,
   default 8.0, normalized finite and positive (fallback 8.0), recorded on
   the actions phase span as `petri.config.exchange.bite` and in the
   `run.started` body with no edit, and offered by the frontend config panel
   like every live-patchable value. `PredationConfig` is unchanged:
   `steal_cost_rate` (0.05) now scales the bite, so steal's effort base is
   `steal_cost_rate × bite`, which at the defaults is the floor.
   `ActionParamField::StealEnergyAmount` is retired: `ACTION_PARAM_FIELD_COUNT`
   3, VM `field_idx` in `0..3`, `WorldAction::StealEnergy { direction }`,
   `param(1)` of a steal reads 0.0.
5. **The transfer rules.** `GiveEnergy` follows `StealEnergy` in
   `DirectedVoteKind::ALL`; `VoteKind::GiveEnergy`; `ActionType::GiveEnergy = 5`
   (`ACTION_TYPE_COUNT` 6); `WorldAction::GiveEnergy { direction }`; the
   decoder reads no field; `apply_give_energy(giver, sim, dir)` lives in
   `simulation/actions/giving.rs`. Both actions follow one order:

   | Step | `StealEnergy` (attacker A, victim V) | `GiveEnergy` (giver G, recipient R) |
   | --- | --- | --- |
   | Effort, first | `action_effort(steal_cost_rate × bite)` (the floor at the defaults) against `ActionStealEnergy`, booked `action_charges[StealEnergy]` | `action_effort(noop_cost)` (the floor) against `ActionGiveEnergy`, booked `action_charges[GiveEnergy]` |
   | Counterpart | the creature on the neighbour cell in `dir`, never A; none → `RejectedNoVictim` | the creature on the neighbour cell, never G; none → `RejectedNoRecipient` |
   | Amount | `min(bite, V.energy)` when `V.energy > 0.0`, else 0 (a comparison: `NaN` or exhausted transfers nothing); 0 → `RejectedNoVictim` | `min(bite, G.energy)` after the effort when `G.energy > 0.0`, else 0; 0 → `RejectedNothingToGive` |
   | Applied test | `Transferred` only when the debited side's stored `f32` energy changed; an amount `f32` rounding absorbs on that side (a configured bite below its resolution) is the same zero-transfer failure. The credit side books its own applied change, 0 when rounding absorbs it: one-sided rounding is still `Transferred`. The fixtures cover a sub-resolution bite for both actions; the default bite (8.0) against energies at or below the cap never rounds away. | same |
   | Failure booking | `failed_action_effort[StealEnergy]` | `failed_action_effort[GiveEnergy]` |
   | Debit | V's energy reduced by the amount; the applied change, observed against `Predation`, is `predation_victim_debit` | G's energy reduced by the gift; the applied change, observed against `Gift`, is `gift_debit` |
   | Credit | A's energy raised by the amount; the applied change is `predation_attacker_credit`; then `cap_energy` | R's energy raised by the gift; the applied change, observed as recovery, is `gift_credit`; then `cap_energy` (excess → `maximum_energy_clamp_loss`). Each side books its own `f32` arithmetic, so the pair agrees up to rounding and is never asserted exactly; a recipient at its cap still receives a `Transferred` gift |
   | After | kill rule, bonus, `TransferredAndKilled`, `PredationEventRecord`, V's `DamageDelta`: unchanged; `Transferred` otherwise | `Transferred`; a giver left at or below zero is removed right after this dispatch by the shared post-action removal (`tick/helpers.rs`), its remaining queue dropped and its cell freed this tick, with pending cause `Gift` |
   | Action log | `Success`, `TransferredAndKilled`, `NoVictim`; amount = transfer | `Success`, new `ActionResult::NoRecipient = 9`, `EnergyConstraints` (nothing to give); amount = gift |
   | Outcome accumulator | success flag; V's damage | G's success flag; no damage |

   `DeathCause::ALL` gains `ActionGiveEnergy` (`action_give_energy`) and
   `Gift` (`gift`).
6. **Counting and versions.**

   | Surface | Change |
   | --- | --- |
   | `SimStats` | `give_actions_attempted_total`, `give_actions_transferred_total`, `give_actions_rejected_total`, `give_actions_by_result: HashMap<GiveActionResult, u64>`, `last_tick_give`, reset like the predation fields |
   | goal summary | a `giving` block beside `predation` (`GivingTracking`: the three totals, `actions_by_result` keyed by `GiveActionResult::as_key`), kept in the committed summary (`KEPT_CASE_BLOCKS`); `energy_flows.gift_debit` / `gift_credit`; `mortality.by_cause` carries the new keys through `DeathCause::ALL`; `drives` lists four kinds (`drives-v2`) |
   | server | every payload carrying the predation fields (stats, health, the WS status event) gains `give` beside `steal`, `give_actions_attempted_total`, `give_actions_transferred_total`, `give_actions_rejected_total`, and `give_actions_by_result` beside `predation_actions_by_result`; `PROTOCOL_VERSION` `v3alpha13` in commit 1 with a §2 history line naming the `Directed` shape, extended in commits 2 and 3 with the retired amount, the `GiveEnergy` action and the stats fields |
   | telemetry | families in the Telemetry table; `petri.drive_kind` `give_energy`; dashboard `always_names` |
   | versions | `GENOME_FORMAT` bumps in each commit that changes a persisted representation or its meaning: 8 (commit 1, sink serde), 9 (commit 2, `field_idx` meaning), 10 (commit 3, catalog indices); `neighborhood-coverage-v4` (39 sink rows); `input-use-v4` (`CommitCounts` width 5, `ActionVotes` / `PreviousPassVotes` width 39 change channel meanings); `mutation-effects-v5` stands (input-field contexts untouched) |
7. **Reach.** `can_host_facts(GiveEnergy)` is true (it has a drive), so ring,
   pair and fact-member groups target it like the other kinds; `AddVote` and
   `AddGraphEdge` reach its nine sinks; the samplers draw over the new counts
   (`VOTE_SINK_COUNT` 39, `FIXED_SINK_COUNT` 106, kind draws `0..4`,
   `RetargetGroupKind` `0..3`, `WriteActionParam` `0..3`). The founder wires
   no give sink and never commits a give; the census test that a unit edge
   into an unused kind never kills the founder covers `GiveEnergy`.
8. **Determinism and pins, by commit.** The gate and goal epochs are
   re-pinned in the closing commit.

   | Commit | Draws and behavior | Re-pinned | Unchanged, and why |
   | --- | --- | --- | --- |
   | 1 catalog | no draw added, removed or reordered; no behavior change | text fingerprints only: the `Directed` shape in `recruitment_paths` rows, genome fixtures and server payload tests; `applied_trajectory.rs`'s sampled digests, which hash serialized genomes (serde alone, as its T11.F27 and T25.F03 re-pins); `baseline_worlds.rs`'s `legacy_default_short_run_identity` pins, which hash genome `Debug` text (text alone, as its T11.F27 re-pin) | the applied guard, every founder-only digest and every behavior reading |
   | 2 bite | `WriteActionParam` field draws `0..3`, sink surface 97; a steal transfers the bite | every mutation-on pin | founder-only pins (`viability.rs` digests, founder baselines): the founder never commits a steal (`census.rs:160-174`) and reads no vote-width input |
   | 3 give | kind draws `0..4`, `RetargetGroupKind` `0..3`, sink draws 39 / 106, sub-index draws over the widened `ActionVotes`, `PreviousPassVotes`, `CommitCounts` | every mutation-on pin | founder-only pins: a kind with zero votes never leads `select` (strict comparison, `Eat` first) and the founder reads only the action-queue type |
9. **Reference docs** state the code; no sentence names `StealEnergyAmount`
   as current or the rate as scaling a requested amount.

   | Document | States |
   | --- | --- |
   | `v3-graph-backend-spec.md`, `v3-mesh-execution-spec.md` | the `Directed` shape and index rule, 39 vote sinks, 106 fixed sinks, five kinds, three parameter fields, the pass-end loop over four directed kinds |
   | `v3-vm-isa-spec.md` | `AddVote` rows `GiveEnergy` 25–32, drives 33–36, `Terminate` 37, `Decide` 38; `field_idx` `0..3`; `ReadActionQueueType` code 5 |
   | `v3-mutation-spec.md` | 106 sinks, 39 vote sinks, kind draws `gen_range(0..4)` over four kinds |
   | `v3-sensor-spec.md` | `CommitCounts` width 5; `ActionVotes` / `PreviousPassVotes` width 39 |
   | `v3-tick-orchestration-spec.md`, `v3-runtime-config-spec.md` | the steal and give steps with their effort bases and the bite; `exchange.bite`; `action_charges.<action>` keys; "Energy after action" states the post-action removal the code does (`tick.rs` after each dispatch), not the next Phase 0 |
   | `v3-server-api-protocol-spec.md` | the action list with `GiveEnergy`, the `Directed` sink shape, the history line |

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.give_actions_attempted`, `give_actions_transferred`, `give_actions_rejected` | counter (Sum), added to `always_names` | the give executor | `petri.run_id` |
| `petri.run.give_actions_by_result` | counter by `petri.result` (`Transferred`, `RejectedNoRecipient`, `RejectedNothingToGive`) | the give executor | `petri.run_id` |
| `petri.run.energy_flows` points `gift_debit`, `gift_credit`, `action_charge.give_energy`, `failed_action_effort.give_energy`, `failed_action_effort.noop` | existing family (`petri.flow`) | the give ledger and the charge arrays | `petri.run_id` |
| `petri.run.predation_*`, `predation_actions_by_result`, flows `predation_victim_debit` / `predation_attacker_credit` | existing | the bite-rule steal | unchanged names; in the default goal runs the credit is non-zero where a transfer occurs (invariant 5 permits a zero credit only under one-sided rounding) |
| `petri.run.deaths_by_cause` keys `action_give_energy`, `gift` | existing family | `DeathCause::ALL` | `petri.cause` |
| `petri.tick.actions` with `petri.action = "give"` | sampled gauge | `last_tick_give` | existing family |
| goal summary `giving` block, `energy_flows.gift_*`, `action_charges.give_energy`, `mortality.by_cause.gift`, `drives` four kinds; `predation` block and `predation_attacker_credit` | per-world blocks | the same counters | reported per world in the readings file, zero included |
| `petri.config.exchange.bite` on the `actions` phase span and `creature_window` root; `run.started` body `exchange.bite` | recorded configuration | `SimulationConfig` | beside the existing `petri.config.predation.*` attributes |
| creature window / trace payloads | existing spans | `WorldAction::GiveEnergy`, `Directed` sinks in `best` | protocol `v3alpha13` |

## Verification Plan

**Question.** F03 claims that a committed give moves one bite from the giver
to the creature on the target cell and a committed steal takes one bite
from it, both through the ledger and never as a zero transfer, with the
founder population still alive; a failing fixture or ledger test, a goal
run counting a `Transferred` with no matching flow, or a viability failure
shows it wrong. The count of gives and bites at 2,000 ticks is a reading,
zero included (the track note calls it inconclusive for use), and cannot
show the feature wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal`, viability first (`cargo test -p v3-core --test viability` before the gate) | the track note names F03 "viability first, gate + goal": a creature-authored pressure the standard baseline contract reads in the three goal worlds, and a vote-vector and catalog widening whose cost the gate reads; goal-long is rationed to T27.F02, the track's one carrier, which depends on F03 so its long run includes giving. |
| Mutation gate | `run`: fresh `make rust-mutants-ci`, with local `MUTANTS_ITERATE=0 make rust-mutants` as the fallback when Actions is unavailable | new production logic in `v3-core` (the give action, the bite rule, the catalog shape, the charge arrays) and in `v3-cli` / `v3-telemetry` exports that tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | the sink catalog and the mutation draws change, which moves the recruitment-paths recorded readings the tier pins. |

## Implementation Tasks

- [x] Commit 1 (catalog, behavior-preserving, tests first): invariants 1, 2 and 3; `PROTOCOL_VERSION` `v3alpha13` and its history line, `GENOME_FORMAT` 8 (6); the reference docs' `Directed` shape (9); the text fingerprints of invariant 8's first row re-pinned with the cause "sink serde and `Debug` shape"; the applied guard, every founder digest and every behavior reading unmoved (8). Verified by the catalog index tests, the fixture pin, the frontend parity tests, a creature-window assertion over the iterated charge sum, `tests/applied_trajectory.rs`'s applied guard and `cargo test -p v3-core`.
- [x] Commit 2 (the bite and the zero-transfer fix; depends on commit 1): invariant 4; the steal column of the rules table with its ledger, `NaN` and sub-resolution tests (5); `GENOME_FORMAT` 9 and the history line (6); the reference-doc sentences, the post-action removal one included (9); every mutation-on pin re-pinned with the causes "field draws `0..3`, sink surface 97, steal transfers the bite", founder-only pins unmoved (8). Verified by the predation tests (`Transferred` only with a positive transfer, `RejectedNoVictim` on an exhausted victim, the flow pair), `tests/viability.rs` and `cargo test -p v3-core`.
- [x] Commit 3 (giving; depends on commit 2): the give column of the rules table, `giving.rs`, death causes, action-log result (5); counting and versions (6); reach (7); the reference docs' sizes and steps (9); every mutation-on pin re-pinned with the causes "kind draws `0..4`, sink draws 39 / 106, sub-index widths", founder-only pins unmoved (8). Verified by the give fixture (a gift credited and capped, each side's applied booking, `NoRecipient`, `EnergyConstraints`, a giver exhausted by its gift and removed after its dispatch), the creature-window assertion over a give's charge, the telemetry family tests, `tests/viability.rs` and `cargo test -p v3-core`.
- [ ] Verification Plan: viability first, then gate and goal through the benchmark specialist; mutation gate and slow tier through the mutation specialist; epochs re-pinned in the closing commit.

## Verification

- [x] Focused tests: `cargo test -p v3-core --test viability` -> 30 passed, first; `cargo test -p v3-core --features telemetry-seams` -> 2290 lib, all pass; `cargo test --workspace --exclude v3-core` -> pass but `gate_profile_has_no_severe_regression_against_series_references` (`decided_passes` 0 -> 0.000238; the closing epoch re-pin owns it); `npm test` -> 681 passed; check, clippy, fmt clean.
- [ ] Three-world pressure: `exchange.bite` 8.0 in Orchards, Canyon and Confluence (no world file overrides it); per world the `giving` and `predation` blocks, the gift and predation flows, kills and the `gift` / `action_give_energy` / `action_steal_energy` deaths, zero included, in the readings file.
- [ ] Mutation gate: fresh `make rust-mutants-ci` (or the local fallback), its summary line, output path, and every survivor resolved as killed, equivalent, or deferred.
- [ ] Slow tier: `make rust-test-slow` -> `<result>`, recruitment-paths rows re-pinned.
- [ ] Gate and goal: summaries at `docs/progress/features/t27-f03-giving-energy.json` and `-goal.json`, raw hash/byte count checked, series entries point to them, no full report staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: trophallaxis
and parental feeding; the gift reaches a neighbour through the body (one
mouthful of the giver's own energy) and the world (the adjacent cell), and
the same mouthful is what a bite takes. References and thresholds: the gate
and goal are compared against the current epochs (re-pinned at T27.F01's
closure, `docs/progress/benchmark-series.json`) under the existing
thresholds; no compute cost is predeclared beyond the widened vote vector
(30 to 39 floats, one more kind in `select`) and graph catalog (98 to 106
sinks), which is expected inside the thresholds. The workflow's gate stands:
a `severe` compute regression blocks closure until the user accepts it as a
reading of the changed draw stream or remediation removes it; a `severe` on
a behavior indicator is a trajectory reading reported to the user; the
epochs are re-pinned only on acceptance and nothing is tuned to pass. A
wall-clock `severe` on the epoch host is re-run once before it is read; a
missing level is inconclusive. Behavior: steals now transfer energy, so
`predation_attacker_credit` becomes non-zero where a transfer occurs and
kills can appear; the payoff table (upper bounds at the F09 hit rates) puts
blind stealing near break-even at a bite of 8 and the targeting payoff is a
measurement, so no direction is predeclared for population, births or
energy indicators; give attempts at 2,000 ticks are expected near zero, a
valid reading. The expected first use, if any, is parental care
(Taylor-Davies et al. 2024); it is not targeted and its absence is not a
failure.

| Artifact | Predeclared change |
| --- | --- |
| trajectory pins | mutation-on re-pinned (commits 2 and 3); founder-only unchanged |
| text fingerprints (serde, `Debug`) | re-pinned (commit 1 shape, commit 3 counts) |
| mutation draws | re-pinned (commits 2 and 3) |
| epochs | re-pinned in the closing commit, on acceptance of any `severe` |

**Measured verdict.** Pending.

- Summaries of the declared profiles: [gate](../../progress/features/t27-f03-giving-energy.json),
  [goal](../../progress/features/t27-f03-giving-energy-goal.json).
- Full readings: [`docs/progress/readings/t27-f03.md`](../../progress/readings/t27-f03.md).

## Success Criteria

- [ ] A brain can vote `GiveEnergy` toward a direction; a committed give moves `min(bite, giver energy)` to the creature there, booked as `gift_debit` / `gift_credit` with any excess over the cap as clamp loss, and a give with no recipient or nothing to give fails and pays the floor.
- [ ] A committed steal moves `min(bite, victim energy)`; `StealEnergyAmount` is gone; no `Transferred` is counted without a positive transfer.
- [ ] `VoteSink::Directed`, `voteSinks.json` and the `ActionType`-indexed charge arrays are the one catalog; the frontend keeps no hand copy.
- [ ] The three goal worlds report their `giving` and `predation` occurrences, zero included; viability, gate, goal, mutation gate and slow tier ran as declared.

## Notes for AI Agents

- Decision: the bite is 8.0, one config value `exchange.bite` read by give and steal alike, chosen against the recorded 4–9% blind victim-hit rates and the 0.4 floor (readings file: upper-bound payoff near break-even blind, 19 floors targeted, the Taylor-Davies 20:1 ratio; the realized payoff is a measurement).
- Decision: a give pays the effort floor (track note); a steal keeps `predation.steal_cost_rate` as its effort base, scaling the bite (the floor at the defaults), so the rate stays a policy knob.
- Decision: a transfer is `min(bite, available)`; a non-positive or `NaN` available energy transfers nothing (a comparison, as T26.F01's cap); a giver may exhaust itself giving, attributed to `Gift`, as a parent may through `ParentalTransfer`; each side's flow is its own applied `f32` change, never a mirrored nominal amount.
- Decision: a new directed kind is appended to `DirectedVoteKind::ALL` and invariant 1's index rule places it; T28.F01's kinds follow the same rule.
- Decision: a give with no recipient logs `ActionResult::NoRecipient`, with nothing to give `EnergyConstraints`; a steal whose victim has nothing to take is `RejectedNoVictim`.
- Decision: the catalog reaches the frontend through one Rust-pinned `voteSinks.json`; the frontend keeps no hand copy of it.
- Decision: the per-kind charge ledger is indexed by `ActionType` and every consumer iterates `ActionType::ALL`; the NoOp failed-effort slot exists and stays zero.
- Deferred: the long-run reading of whether gives are used, by whom and toward whom, goes to T27.F02's goal-long run, the track's carrier.
- Deferred: a gift event record or overlay and per-creature gift counters, until a reading shows gifts in use.
