# T27.F03 — Giving Energy

**Status**: In Progress
**Last updated**: 2026-10-21
**Feature**: T27.F03
**Track**: [T27 — Social Signals and Exchange](../../roadmaps/t27-social-signals-and-exchange.md)

## Goal

Trophallaxis and parental feeding: a creature passes part of its store to
the creature on a neighbouring cell, and stealing takes part of the victim's
the same way, so care and exploitation are both possible and the simulation
assigns neither a meaning. How much moves is the creature's: a heritable
share per kind (`steal_share`, `give_share` in `HeritableBody`, founder 0.2,
mutated at birth, cryptic until a sink is wired) × a decoded effort field per
kind (`StealEffort`, `GiveEffort`; unwired means ×1) × the source's current
energy (the victim's for a steal, the giver's own for a give). Both acts pay
one graded effort with the `NoOp` floor; a transfer that moves nothing fails
and credits no kill. `GiveEnergy` is a fourth directed vote kind;
`StealEnergyAmount` is retired; the row's catalog refactor is folded in and
the zero-transfer fix lands as its own commit.

## Non-Goals

- Any meaning, kin test or recipient selection inside the action: who
  receives is the brain's choice through the direction sinks; parental care
  is the expected first use (track note), not a target.
- A fixed amount anywhere (the reverted `exchange.bite`); any give cost
  beyond the graded effort; a sustain cost (signals carry none).
- Held signals (T27.F02), carrying (T28.F01), the capacity and its rules
  (T26.F02); the kill rule and its bonus formula, which this feature reaches
  only when a steal takes the whole store.
- A gift event record or overlay, a recipient classification; renaming the
  `predation` stats, block or families, so the series stay continuous.
- Backward compatibility: a config naming `predation.steal_cost_rate` or a
  genome serialized under the old sink shape is rejected, and every moved
  pin is predeclared and re-pinned.

## Inputs and Invariants

| Input | Where | What it fixes |
| --- | --- | --- |
| Roadmap row and Notes | `docs/roadmaps/t27-social-signals-and-exchange.md` F03 row; Notes "F03 amount (user decisions, 2026-10-21)", "F03 lands the zero-transfer fix", "F03 folds in the directed-kind catalog refactor", "Steal under the old decoder evolved late", "F02 depends on F03", "Verification" | The three amount factors, the effort rule, the folded refactor, the separate fix commit, the profiles. |
| Design review | [`t27-design-review.md`](../../progress/readings/t27-design-review.md) findings 1–4; [`t26-t28-design-review.md`](../../progress/readings/t26-t28-design-review.md) findings 4 and 8 | The payoff predeclaration, the inconclusive 2,000-tick zero, F02 after F03. |
| Prior art | [prior-art note](../../strategy/interaction-features-prior-art-2026-10-20.md) §Giving: lossless transfer plus the effort floor, give from what the donor holds, Polyworld's `GetGiveStatus` (energy × give activation × fraction) | Why the gift is lossless and proportional to the giver's store. |
| Options, payoff and audit | [`docs/progress/readings/t27-f03.md`](../../progress/readings/t27-f03.md) | The alternatives weighed, the payoff arithmetic and the refactor table behind the `Decision:` notes. |
| T26.F01, T26.F02 | `creature/body.rs` (`HeritableBody { phenotype, identity, capacity }`, `founder(index, seed, capacity)`, `inherit(&parent, mutated, &MutationConfig, &EnergyConfig, rng)` with the capacity's per-birth chance and log-step draws after the kin tag), `actions/mod.rs` (`pay_effort`, `charge_effort`, `cap_energy`), `CreatureState::energy_cap`, `config/simulation.rs:682-696` (`CapacityMutationConfig`), `v3-cli/src/bench/capacity.rs` | The record the shares join, the inheritance rule they copy, the credit path a gift and a steal use, the sampled distribution block the shares mirror. |
| Catalog today | `creature/genome/vote.rs` (`VOTE_SINK_COUNT` 30, `VOTE_KIND_COUNT` 4, `ACTION_PARAM_FIELD_COUNT` 4, three directed variants), `creature/genome/cgp/mod.rs:101-147` (`OutputSinkKind`, `FIRST_ACTION_VOTE_SINK` 64, `FIXED_SINK_COUNT` 98), `runtime/vote_select.rs`, `runtime/action_decode.rs`, `mutation/types/mod.rs:91-104`, `mutation/graph/group.rs:132,170,235`, `mutation/vm/operators.rs:230,279`, `mutation/input_ref/mod.rs:349`, `mutation/compound.rs:23` | The one catalog and every draw over it. |
| Steal today | `simulation/actions/predation.rs` (`apply_steal_energy`: effort `steal_cost_rate × requested` before the target resolves, `min(requested, victim.energy)`, a zero amount still `Transferred`, a victim at or below zero killed), `simulation/tick.rs:1145-1190` (`execute_steal_energy` reads the amount back from `last_tick_predation_events`), `contracts/actions.rs` (`WorldAction::StealEnergy { direction, amount }`, `action_type() -> u8`, `param`), `config/simulation.rs:1005-1021` (`PredationConfig`), `neighborhood/census.rs:160-174` (the founder never commits a steal) | The path the share rule replaces and the founder guard. |
| Ledger | `simulation/energy_accounting.rs` (`DeathCause::ALL` 16, `ActionCharges`, `FailedActionEffort`, `EnergyFlows`, `net_action_overdraft`), `tick/window_seam.rs:131-143` (`FlowMarks`), `creature/action_log.rs` (`ActionType` 5, `ActionResult` 9), `simulation/stats.rs`, `simulation/outcomes.rs` | Where a transfer is booked and logged. |
| Exports | `v3-cli/src/bench/tracking.rs` (`PredationTracking`, charge and flow blocks `applied-energy-flows-v3`, `capacity`), `bench/drives.rs` (`drives-v1`), `v3-telemetry/src/metrics.rs:556-562,625-666,900-930,1160-1170`, `metrics/tests.rs`, `trace.rs:430-445` (`predation.*` phase attributes), `scripts/telemetry-dashboards-check`, `v3-server/src/state.rs:96-126,218-230,385-400`, `v3-server/src/http/creature.rs:44-60`, `v3-server/src/types.rs` (`v3alpha12`), `v3-lab/src/summary.rs` (`GENOME_FORMAT` 7), `neighborhood/mutation_effects/mod.rs:44-45`, `neighborhood/input_use/mod.rs:48` | Every surface that names a kind, a charge, a flow or a body field. |
| Frontend | `components/inspector/voteSinks.ts`, `types/genome.ts`, `types/trace.ts`, `types/action-log.ts`, `types/config.ts:198-280`, `graphNodeFormatters.ts`, `inputRefUtils.ts`, `vmInstructionFormat.ts`, `config-panel/runtime/PredationSection.tsx`, `VitalsBanner.tsx`, `stats/shared/ActionBar.tsx`; the `nearbyFields.json` pattern (`contracts/inputs.rs:583-600`) | The hand mirrors the fixture replaces and the pin pattern. |
| Pins | `tests/viability.rs`, `tests/applied_trajectory.rs`, `tests/baseline_worlds.rs`, `tests/reproducibility.rs`, `tests/recruitment_paths.rs` and the `--lib` rows, `mutation/graph/recruitment_tests.rs`, `creature/genome/mod.rs` serde shapes, `census.rs`, the v3-cli trajectory fixtures, the gate and goal deterministic blocks, `docs/progress/benchmark-series.json` | The predeclared re-pins (invariant 9). |
| Reference specs | `v3-graph-backend-spec.md`, `v3-mesh-execution-spec.md`, `v3-vm-isa-spec.md`, `v3-mutation-spec.md`, `v3-sensor-spec.md`, `v3-tick-orchestration-spec.md:263-268`, `v3-runtime-config-spec.md:320-335` and the `predation` section, `v3-reproduction-spec.md:284-298`, `v3-startup-seeding-spec.md:154-156`, `v3-server-api-protocol-spec.md` | Every sentence that names the catalog sizes, the steal amount, the rate or the body's inheritance. |

Invariants state the closing state; invariant 9 and the commit table give
each commit's intermediate counts.

1. **One directed shape.** `VoteSink` is `Eat`, `Directed(DirectedVoteKind, u8)`,
   `Drive(DirectedVoteKind)`, `Terminate`, `Decide`;
   `DirectedVoteKind::ALL` is `[Move, Reproduce, StealEnergy, GiveEnergy]`
   (`N` = 4). `Eat` 0, `Directed(K, d)` `1 + 8·K.index() + d`, `Drive(K)`
   `1 + 8·N + K.index()`, `Terminate` `VOTE_SINK_COUNT − 2`, `Decide`
   `VOTE_SINK_COUNT − 1`, `VOTE_SINK_COUNT = 3 + 9·N` = 39;
   `VOTE_KIND_COUNT` 5 with `VoteKind::ALL` = `Eat` then the directed kinds'
   vote kinds in `ALL` order (a test pins it); `FIXED_SINK_COUNT` 108.
   `DirectedVoteKind::sink`/`of_sink`, `VoteSink::kind`, `GroupClass::of_sink`,
   `OutputSinkKind::direction`, `vote_select` and `can_host_facts` have one
   arm for the directed case and never name a kind; `decode_commit` extracts
   the kind and direction once and keeps one arm per kind for the action it
   builds. Serde: `{"Directed":["Move",3]}`; drives keep `{"Drive":"Move"}`.
2. **One fixture.** `frontend/src/types/vote-sinks.json` holds `voteKinds`,
   `directedKinds`, `voteSinks` (the serde value of every sink in index
   order), `actionParamFields` (`{ field, kind }` in catalog order) and
   `actionTypes` (discriminant order, with codes); a Rust test in `vote.rs`
   pins it to the declarations as `nearbyFields.json` is pinned. The frontend
   derives its sink index functions, counts, kind and field lists and
   action-code strings from it; no hand arithmetic or list of the catalog
   remains in TypeScript, and the `inputRefUtils.test.ts` parity pattern
   covers the fixture.
3. **Charge arrays and return values.** `ActionCharges` and
   `FailedActionEffort` are `[f64; ACTION_TYPE_COUNT]` newtypes indexed by
   `ActionType` (declared in `contracts/actions.rs`, re-exported by
   `creature::action_log`) with `ActionType::flow_key()` (`noop`, `eat`,
   `move`, `reproduce`, `steal_energy`, `give_energy`);
   `net_action_overdraft`, the v3-cli `energy_flows` export (`BTreeMap`s keyed
   by flow key, definition `applied-energy-flows-v4`), the telemetry
   `action_charge.<key>` / `failed_action_effort.<key>` points and
   `FlowMarks::of`'s charge sum iterate `ActionType::ALL`; the `NoOp` failed
   slot stays 0 and is exported like the others; the netting property test
   samples every row (conservation, `failed ≤ charge`, the `NoOp` failed slot
   zero). `FlowMarks`' reward sum excludes `gift_credit`: the credit is the
   recipient's. `WorldAction::action_type()` returns `ActionType` (`as u8` is
   the queue code). `apply_steal_energy` and `apply_give_energy` return their
   result with the amount moved and the counterpart; the tick's executors log
   and record damage from it and no longer read `last_tick_predation_events`.
4. **The shares.** `HeritableBody` gains `steal_share: f32` and
   `give_share: f32`, each in `[exchange.share_min, 1.0]`;
   `HeritableBody::founder(index, seed, capacity, share)` sets both to
   `share` (`exchange.founder_share`, 0.2); seeding and the lab pass it.
   `inherit` copies the parent's shares clamped to the range in force, then,
   on every birth after the capacity draw, inherits `steal_share` then
   `give_share` by the capacity's rule: one `f32` draw against
   `mutation.share.chance` and, when it fires, one uniform log-space step
   `δ ∈ [−log_step, log_step]`, the child `(parent × e^δ)` clamped to the
   range. The three scalar traits share one `inherit_scalar` step and one
   `LogStepMutationConfig { chance, log_step }` type. The creature detail
   (`/creatures/{id}`) reports `steal_share` and `give_share` beside
   `max_energy`. Config, with `PredationConfig` keeping only
   `kill_complexity_bonus_multiplier`:

   | Key | Default | Normalized | Role |
   | --- | --- | --- | --- |
   | `exchange.effort_rate` | 0.05 (today's `predation.steal_cost_rate`, which it replaces) | `[0, 1]`, fallback 0.05 | the one effort rate of both acts |
   | `exchange.share_min` | 0.01 | `(0, 1]`, fallback 0.01 | the lower bound of both shares |
   | `exchange.founder_share` | 0.2 | `[share_min, 1]`, fallback the clamped 0.2 | every founder's `steal_share` and `give_share` |
   | `mutation.share.chance` | 0.1 | `[0, 1]`, fallback 0.1 | the per-birth draw of both shares |
   | `mutation.share.log_step` | 0.25 | `[0, ln(1 / share_min)]` after `share_min` is normalized (the range's own log width, 4.6 at the default; 0 when `share_min` = 1, where both shares stay pinned at 1), non-finite → 0.25 clamped to that bound | the step's half-width |
   | `mutation.capacity` | unchanged | unchanged | now typed `LogStepMutationConfig`; serde key unchanged |

   Normalization order: `exchange.share_min`, then `exchange.founder_share`,
   then `mutation.share`. A live patch of the bounds reaches newborns through
   the clamp at birth; a living creature keeps its stored shares, as it keeps
   its capacity (T26.F02), so the detail reports the stored value and the
   `shares` block's `at_min_fraction` / `at_max_fraction` count equality with
   the bounds in force. Fixtures cover a child clamped at `share_min`, a
   child clamped at 1, and `share_min` = 1.
5. **The effort modifier.** `ActionParamField::StealEffort` takes
   `StealEnergyAmount`'s position (3) and `GiveEffort` follows (4):
   `ACTION_PARAM_FIELD_COUNT` 5, VM `field_idx` in `0..5`. The decoder maps a
   field value `x` to the modifier `2^clamp(x, −2, 2)` in `[0.25, 4]`; a
   non-finite `x` and the unwired 0 give exactly 1.0. `WorldAction::StealEnergy
   { direction, effort }` and `GiveEnergy { direction, effort }` carry the
   modifier; `param(1)` reads it. The intensity of an attempt is
   `min(1, share × modifier)` with the actor's own share of that kind.
6. **The transfer rules.** `GiveEnergy` follows `StealEnergy` in
   `DirectedVoteKind::ALL`; `VoteKind::GiveEnergy`; `ActionType::GiveEnergy = 5`
   (`ACTION_TYPE_COUNT` 6); `apply_steal_energy(attacker, sim, dir, effort)`
   stays in `predation.rs` and `apply_give_energy(giver, sim, dir, effort)`
   lives in `simulation/actions/giving.rs`; the intensity, amount and
   charge-base arithmetic the two acts share lives once, beside `pay_effort`.
   Both actions follow one order:

   | Step | `StealEnergy` (attacker A, victim V) | `GiveEnergy` (giver G, recipient R) |
   | --- | --- | --- |
   | Intensity | `i = min(1, A.steal_share × effort)` | `i = min(1, G.give_share × effort)` |
   | Effort, first | `action_effort(exchange.effort_rate × i × max(A.energy, 0))` against `ActionStealEnergy`, booked `action_charges[StealEnergy]`; the base is known before the target resolves | `action_effort(exchange.effort_rate × i × max(G.energy, 0))` against `ActionGiveEnergy`, booked `action_charges[GiveEnergy]` |
   | Counterpart | the creature on the neighbour cell in `dir`, never A; none → `RejectedNoVictim` | the creature on the neighbour cell, never G; none → `RejectedNoRecipient` |
   | Amount | `i × V.energy` when `V.energy > 0.0`, else 0 (a comparison: `NaN` or an exhausted victim moves nothing); 0 → `RejectedNothingToTake` | `i × G.energy` after the effort when `G.energy > 0.0`, else 0; 0 → `RejectedNothingToGive` |
   | Applied test | `Transferred` only when the debited side's stored `f32` energy changed; an amount `f32` rounding absorbs is the same zero-transfer failure. The credit side books its own applied change, 0 when rounding absorbs it: one-sided rounding is still `Transferred`. The fixtures cover a sub-resolution amount for both actions (`share_min` against a tiny store). | same |
   | Failure booking | `failed_action_effort[StealEnergy]` | `failed_action_effort[GiveEnergy]` |
   | Debit | V's energy reduced by the amount; the applied change, observed against `Predation`, is `predation_victim_debit` | G's energy reduced by the gift; the applied change, observed against `Gift`, is `gift_debit` |
   | Credit | A's energy raised by the amount; the applied change is `predation_attacker_credit`; then `cap_energy` (excess → `maximum_energy_clamp_loss`) | R's energy raised by the gift; the applied change, observed as recovery, is `gift_credit`; then `cap_energy`. Each side books its own `f32` arithmetic, so the pair agrees up to rounding and is never asserted exactly; a recipient at its cap still receives a `Transferred` gift |
   | After | kill rule, bonus, `TransferredAndKilled`, `PredationEventRecord`, V's `DamageDelta`: unchanged, reached only at `i = 1` (`V.energy − V.energy ≤ 0`) or by rounding; `Transferred` otherwise | `Transferred`; a giver left at or below zero is removed right after this dispatch by the shared post-action removal, its remaining queue dropped and its cell freed this tick, with pending cause `Gift` |
   | Action log | `Success`, `TransferredAndKilled`, `NoVictim` (both rejections); amount = transfer | `Success`, new `ActionResult::NoRecipient = 9`, `EnergyConstraints` (nothing to give); amount = gift |
   | Outcome accumulator | success flag; V's damage from the returned amount | G's success flag; no damage |

   `PredationActionResult` gains `RejectedNothingToTake`
   (`"RejectedNothingToTake"`); `GiveActionResult` is `Transferred`,
   `RejectedNoRecipient`, `RejectedNothingToGive` with `as_key` the variant
   names. `DeathCause::ALL` gains `ActionGiveEnergy` (`action_give_energy`)
   and `Gift` (`gift`).
7. **Counting and versions.**

   | Surface | Change |
   | --- | --- |
   | `SimStats` | `give_actions_attempted_total`, `give_actions_transferred_total`, `give_actions_rejected_total`, `give_actions_by_result: HashMap<GiveActionResult, u64>`, `last_tick_give`, reset like the predation fields |
   | goal summary | a `giving` block beside `predation` (`GivingTracking`: the three totals, `actions_by_result` keyed by `GiveActionResult::as_key`), kept in the committed summary (`KEPT_CASE_BLOCKS`); `energy_flows.gift_debit` / `gift_credit`; `mortality.by_cause` carries the new keys through `DeathCause::ALL`; `drives` lists four kinds (`drives-v2`); a `shares` block beside `capacity`, sampled in the same pass, with the `CapacityDistribution` fields for `steal_share` and `give_share` over `[share_min, 1]` |
   | server | every payload carrying the predation fields (stats, health, the WS status event) gains `give` beside `steal`, the three give totals and `give_actions_by_result` beside `predation_actions_by_result`; the creature detail gains `steal_share`, `give_share`; `PROTOCOL_VERSION` `v3alpha13` in commit 1 with a §2 history line naming the `Directed` shape, extended in later commits with the retired amount, the shares, the `GiveEnergy` action and the stats fields |
   | telemetry | families in the Telemetry table; `petri.drive_kind` `give_energy`; dashboard `always_names` |
   | versions | `GENOME_FORMAT` (today 7) 8 in commit 1 (sink serde), 9 in commit 3 (field 3's meaning), 10 in commit 4 (catalog indices); `neighborhood-coverage-v4` (39 sink rows); `input-use-v4` (`CommitCounts` width 5, `ActionVotes` / `PreviousPassVotes` width 39); `mutation-effects-v5` stands (input-field contexts untouched) |
8. **Reach.** `can_host_facts(GiveEnergy)` is true (it has a drive), so ring,
   pair and fact-member groups target it like the other kinds; `AddVote` and
   `AddGraphEdge` reach its nine sinks; the samplers draw over the new counts
   (`VOTE_SINK_COUNT` 39, `FIXED_SINK_COUNT` 108, kind draws `0..4`,
   `RetargetGroupKind` `0..3`, `WriteActionParam` `0..5`). The founder wires
   no steal or give sink and never commits either; the census test that a
   unit edge into an unused kind never kills the founder covers `GiveEnergy`.
   The shares are cryptic until a sink is wired: no founder-economy reading
   moves except through the added draws.
9. **Determinism and pins, by commit.** No tick-phase order changes. The
   gate and goal epochs are re-pinned in the closing commit; a re-pin is
   accepted only with the cause its comment records.

   | Commit | Draws and behavior | Re-pinned | Unchanged, and why |
   | --- | --- | --- | --- |
   | 1 catalog | no draw added, removed or reordered; no behavior change | text fingerprints only: the `Directed` shape in `recruitment_paths` rows, genome fixtures and server payload tests; `applied_trajectory.rs`'s sampled digests, which hash serialized genomes; `baseline_worlds.rs`'s `legacy_default_short_run_identity` pins, which hash genome `Debug` text | the applied guard, every founder-only digest and every behavior reading |
   | 2 zero-transfer fix | no draw changes; a steal that moves nothing fails (`RejectedNothingToTake`, no kill, no `Predation` death, no event record) | every mutation-on pin whose trajectory holds such a steal; `viability.rs` founder-only digests unmoved | founder-only pins: the founder never commits a steal |
   | 3 shares and effort | two chance draws (and their steps) per birth after the capacity draw; `WriteActionParam` field draws `0..4` keep their range but field 3 means `StealEffort`; a steal takes `i × V.energy` | every pin that depends on a trajectory with births (`applied_trajectory.rs`, `reproducibility.rs`, `viability.rs` and `baseline_worlds.rs` digests, seeded fixtures, the gate's deterministic blocks), the config digests, as T26.F02's invariant 8, and any text fingerprint naming `StealEnergyAmount` | mutation-engine draws, the recruitment-paths slow-tier readings (one-tick scenes, no births) |
   | 4 give | kind draws `0..4`, `RetargetGroupKind` `0..3`, sink draws 39 / 108, field draws `0..5`, sub-index draws over the widened `ActionVotes`, `PreviousPassVotes`, `CommitCounts` | every mutation-on pin; the recruitment-paths slow-tier readings | founder-only behavior: a kind with zero votes never leads `select` (strict comparison, `Eat` first) and the founder reads only the action-queue type; the founder-only digests moved in commit 3 only |

10. **Reference docs** state the code; no sentence names `StealEnergyAmount`,
    `predation.steal_cost_rate` or a requested amount as current.

    | Document | States |
    | --- | --- |
    | `v3-graph-backend-spec.md`, `v3-mesh-execution-spec.md` | the `Directed` shape and index rule, 39 vote sinks, 108 fixed sinks, five kinds, five parameter fields, the pass-end loop over four directed kinds |
    | `v3-vm-isa-spec.md` | `AddVote` rows `GiveEnergy` 25–32, drives 33–36, `Terminate` 37, `Decide` 38; `field_idx` `0..5` with `StealEffort` 3 and `GiveEffort` 4; `ReadActionQueueType` code 5; `ReadActionQueueParam` slot 1 as the effort modifier |
    | `v3-mutation-spec.md` | 108 sinks, 39 vote sinks, kind draws over four kinds, field draws over five |
    | `v3-sensor-spec.md` | `CommitCounts` width 5; `ActionVotes` / `PreviousPassVotes` width 39 |
    | `v3-tick-orchestration-spec.md`, `v3-runtime-config-spec.md` | the steal and give steps with the intensity, the charge base and the floor; the `exchange` section; `mutation.share`; `action_charges.<action>` keys; "Energy after action" states the post-action removal the code does |
    | `v3-reproduction-spec.md`, `v3-startup-seeding-spec.md` | steps 10–11 with the share draws after the capacity draw; the founder's shares from `exchange.founder_share` |
    | `v3-server-api-protocol-spec.md` | the action list with `GiveEnergy`, the `Directed` sink shape, the detail's share fields, the history line |

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.run.give_actions_attempted`, `give_actions_transferred`, `give_actions_rejected` | counter (Sum), added to `always_names` | the give executor | `petri.run_id` |
| `petri.run.give_actions_by_result` | counter by `petri.result` (`Transferred`, `RejectedNoRecipient`, `RejectedNothingToGive`) | the give executor | `petri.run_id` |
| `petri.run.predation_actions_by_result` | existing family, gains `RejectedNothingToTake` | the steal executor | `petri.result` |
| `petri.run.energy_flows` points `gift_debit`, `gift_credit`, `action_charge.give_energy`, `failed_action_effort.give_energy`, `failed_action_effort.noop` | existing family (`petri.flow`) | the give ledger and the charge arrays | `petri.run_id` |
| `petri.run.predation_*`, flows `predation_victim_debit` / `predation_attacker_credit` | existing | the share-rule steal | unchanged names; a transfer with zero credit occurs only under one-sided rounding |
| `petri.run.deaths_by_cause` keys `action_give_energy`, `gift` | existing family | `DeathCause::ALL` | `petri.cause` |
| `petri.tick.actions` with `petri.action = "give"` | sampled gauge | `last_tick_give` | existing family |
| goal summary `giving`, `shares`, `energy_flows.gift_*`, `action_charges.give_energy`, `mortality.by_cause.gift`, `drives` four kinds; `predation` block | per-world blocks | the same counters and the sampled body pass | reported per world in the readings file, zero included |
| `petri.config.exchange.effort_rate`, `exchange.share_min`, `exchange.founder_share` on the `actions` phase span and `creature_window` root (replacing `predation.steal_cost_rate`); `run.started` body `exchange.*`, `mutation.share.*` | recorded configuration | `SimulationConfig` | beside `petri.config.predation.kill_complexity_bonus_multiplier` |
| creature detail `steal_share`, `give_share`; creature window / trace payloads | existing records and spans | the body record; `WorldAction::GiveEnergy`, `Directed` sinks in `best` | protocol `v3alpha13` |

## Verification Plan

**Question.** F03 claims that a committed give moves `min(1, give_share ×
effort) × giver.energy` to the creature on the target cell and a committed
steal takes `min(1, steal_share × effort) × victim.energy` from it, both
through the ledger, never as a zero transfer and never with a kill for
nothing, with the shares inherited by the capacity's rule and the founder
population still alive; a failing fixture or ledger test, a goal run counting
a `Transferred` with no matching debit flow or a kill with no debit, or a
viability failure shows it wrong. The counts of gives and steals at 2,000 ticks are a
reading, zero included (the track note calls a 2,000-tick zero inconclusive
under the late steal onset), and cannot show the feature wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal`, viability first (`cargo test -p v3-core --test viability` before the gate) | the track note names F03 "viability first, gate + goal": a creature-authored pressure the standard baseline contract reads in the three goal worlds, and a vote-vector, field and body widening whose cost the gate reads; goal-long is rationed to T27.F02, the track's one carrier, which depends on F03 so its long run includes giving and the shares. |
| Mutation gate | `run`: fresh `make rust-mutants-ci`, with local `MUTANTS_ITERATE=0 make rust-mutants` as the fallback when Actions is unavailable | new production logic in `v3-core` (the share inheritance, the modifier, the amount and charge rules, the give action, the catalog shape, the charge arrays) and in `v3-cli` / `v3-telemetry` exports that tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | the sink catalog and the mutation draws change, which moves the recruitment-paths recorded readings the tier pins. |

## Implementation Tasks

Four commits in dependency order, tests first in each; every re-pin carries
the cause named in invariant 9.

| Commit | Lands | Verified by |
| --- | --- | --- |
| 1 catalog (behavior-preserving) | invariants 1, 2, 3 at today's widths (`N` = 3: 30 vote sinks, 98 fixed sinks, four vote kinds, four parameter fields, five action types; the four-kind counts are commit 4's): `Directed`, the fixture, the charge arrays, `action_type() -> ActionType`, executors reading the returned result; `PROTOCOL_VERSION` `v3alpha13` and its history line; `GENOME_FORMAT` 8 (sink serde); the reference docs' `Directed` shape (10); the window-seam equivalence oracle retaining death-tick actions (audit R10: `tick/tests/window_seam.rs` compares against a record the test keeps per creature and tick, not the removable action log; the `WORKAROUND(T26.F02)` marker and the reseed history go; a target dying inside its window is covered, which a giver exhausted by its gift needs in commit 4); invariant 9 row 1 re-pinned, cause "sink serde and `Debug` shape" | the catalog index tests, the fixture pin, the frontend parity tests, a creature-window assertion over the iterated charge sum, `tests/applied_trajectory.rs`'s applied guard, `cargo test -p v3-core` |
| 2 zero-transfer fix | the steal column's amount comparison, applied test and `RejectedNothingToTake` under the old amount decoder (6); the key in every export (7); reference-doc sentences (10); invariant 9 row 2 re-pinned, cause "a zero steal fails and kills nothing" | predation tests: `Transferred` only with a positive transfer; an exhausted victim yields `RejectedNothingToTake`, no kill, no `Predation` death, no event record; `cargo test -p v3-core` |
| 3 shares and effort | invariant 4 (body fields, `inherit_scalar`, `LogStepMutationConfig`, `ExchangeConfig` replacing `predation.steal_cost_rate`, detail fields, `shares` block); invariant 5 for `StealEffort`; the steal column with its ledger, `NaN` and sub-resolution tests (6); `GENOME_FORMAT` 9 (field 3's meaning) and the history line (7); reference-doc sentences (10); invariant 9 row 3 re-pinned, cause "share draws per birth; `StealEffort`; the share rule" | body tests (founder shares, the clamp on every birth, draw order and the no-draw case, a proptest that the child's share stays in range for any parent and config); decoder tests (×1 at 0 and `NaN`, the bounds); predation tests (charge base above and at the floor, amount against the victim's store, a kill at intensity 1 and under the rounding exception — a victim whose store is a few `f32` ulps, where `i × V.energy` rounds to `V.energy` — and no kill at intensity 0.2 against an ordinary store); config normalization tests, the clamp fixtures of invariant 4; `tests/viability.rs`; `cargo test -p v3-core` |
| 4 give | the four-kind catalog (39 / 108, five fields, six action types) and the give column, `giving.rs`, `GiveEffort`, death causes, action-log result (1, 5, 6); counting and versions with `GENOME_FORMAT` 10 (7); reach (8); reference-doc sizes and steps (10); invariant 9 row 4 re-pinned, cause "kind draws `0..4`, sink draws 39 / 108, field draws `0..5`, sub-index widths"; founder-only digests unmoved from commit 3 | the give fixture (a gift credited and capped, each side's applied booking, `NoRecipient`, `EnergyConstraints`, a giver exhausted by its gift and removed after its dispatch); the creature-window assertion over a give's charge; telemetry family tests; dashboards check names; `tests/viability.rs`; `cargo test -p v3-core` |

- [x] Commit 1 (catalog).
- [x] Commit 2 (zero-transfer fix).
- [ ] Commit 3 (shares and effort).
- [ ] Commit 4 (give).
- [ ] Verification Plan: viability first, then gate and goal through the
      benchmark specialist; mutation gate and slow tier through the mutation
      specialist; epochs re-pinned in the closing commit.

## Verification

- [ ] Focused tests: `cargo test -p v3-core` -> result; `cargo test -p v3-core
      --test viability` -> result (run first, per the track note);
      `make rust-test-slow` -> result (recruitment-paths readings re-pinned
      to the 39-sink catalog).
- [ ] Frontend: `npm run lint && npm run test && npm run build` in
      `frontend/` -> result; the fixture parity tests listed.
- [ ] Mutation gate: fresh `make rust-mutants-ci` (or the local fallback),
      its summary line, output path, and every survivor resolved as killed,
      equivalent, or deferred, listed here.
- [ ] Gate and goal: summaries at
      `docs/progress/features/t27-f03-giving-energy.json` and
      `t27-f03-giving-energy-goal.json`, local raw hash/byte count and
      verification time checked, series entry points to the summary, no new
      full report staged. Per-world `predation`, `giving` and `shares`
      readings, zero included, in
      [`docs/progress/readings/t27-f03.md`](../../progress/readings/t27-f03.md).
- [ ] Telemetry: the families in the Telemetry table present in
      `crates/v3-telemetry/src/metrics/tests.rs` and
      `scripts/telemetry-dashboards-check`.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: parental
provisioning and regurgitation feeding (a bird feeding nestlings, ant
trophallaxis, a vampire bat sharing a blood meal), and a bite taken from
prey; the mechanism reaches creatures through two directed vote kinds, two
decoded fields and two body traits, with no world change and no configured
amount: the constants are the effort rate (0.05), the floor (0.4), the share
range and the founder's 0.2. Compute: nine more vote cells and one more
parameter field per pass, two body fields and two chance draws per birth, a
`shares` sample per goal snapshot; gate wall-clock is expected within the
ordinary thresholds against the T26.F02 epoch, and the deterministic blocks
change because every birth draws and the catalog widens. The gate and goal
epochs are re-pinned in the closing commit. The payoff arithmetic is in the
readings file; its conclusions, with the expected readings:

| Reading | Expected | A different reading says |
| --- | --- | --- |
| 2,000-tick persistence, three worlds | an observation, not an equality claim: the founder wires no steal or give sink and its shares are cryptic, so the founding economy is unchanged, while the evolving population runs under a new decoder, the zero-transfer rule, a wider catalog and whatever exchange it discovers; a difference from the T26.F02 goal epoch is attributed (the draws disabled and compared; `predation` and `giving` occurrences; the catalog widening) and recorded | equality is expected only of a paired control: the F03 code with its share draws disabled against the T26.F02 code (main at `4b9a5247`), same seeds and settings, founder-only (mutation off), on the common behavior projection: per world, the final population and the tick of extinction or persistence to 2,000; the gate's nine per-creature-tick counters (`mesh_hops`, `vm_steps`, `graph_relax_iters`, `plasticity_updates`, `actions_applied`, `births`, `pass_cap_hits`, `passes`, `decided_passes`) and its creature-tick and birth totals; `energy_flows.food_intake_by_type`, `action_charges.{noop,eat,move,reproduce}`, `lifecycle_decay`, `genome_carrying`, `capacity_upkeep`; the `capacity` distribution — since the founder never commits a steal or a give and a kind with zero votes never wins. Excluded as changed by design: the `drives` block (`drives-v2`, four kinds), the `predation` result keys, the new `giving` and `shares` blocks, `steal_energy`/`give_energy` charge rows, `mortality.by_cause` keys, format and version strings. A difference inside the projection is a defect; a goal-world difference is a reading, and a defect only where a named invariant fails |
| `predation_kills_total`, `mortality.by_cause.predation` | near zero: a kill takes the whole store, at intensity 1 (`steal_share × modifier ≥ 1`, which the founder's 0.2 at modifier 1 never reaches) or when `f32` rounding makes `i × V.energy` equal a tiny store; `TransferredAndKilled` without a positive `predation_victim_debit` is a defect | kills at the old rate: check the intensity cap and the comparison against the kill fixtures before accepting |
| `predation_actions_by_result` | every `Transferred` has a positive `predation_victim_debit`; `RejectedNothingToTake` counts exhausted-victim steals that used to count as kills; `RejectedNoVictim` dominates while steals are blind | a `Transferred` with no debit: the applied test is wrong |
| Steal payoff at neutral effort | the charge base at the defaults (0.24–0.33) is below the floor, so an attempt charges `0.4 × m`, `m` the complexity-and-age multiplier (≥ 1); the illustrative gross return per blind attempt is 0.19–0.59 before the attacker's cap, which gives a break-even hit rate of 6–8% only under the stated assumptions (m = 1, no cap loss, the old population means); profitability is unestablished and is read, not targeted, as `predation_attacker_credit` (credited after the cap) against `action_charges.steal_energy` | — |
| `giving` block, `gift_debit` / `gift_credit` | reported per world, zero included; a give moves 0.2 × the giver's store at neutral effort, so a giver pays about 5–7 plus the floor; the first evolved use is expected to be parent-to-child feeding and is not targeted | a `Transferred` give with `gift_debit` 0: a defect; a zero `gift_credit` is permitted only where the recipient's rounding absorbs it or the cap books it as clamp loss |
| `shares` block | min, p10, median, p90, max, mean near 0.2 with no predeclared direction: the shares are cryptic until wired, so they walk under the mutation alone | a mass at `share_min` or 1 within 2,000 ticks: check the step and the clamp before accepting; the clamped walk permits boundary mass over long runs, and a founder that wires neither sink feels no selection on it |
| Gate per-creature-tick counters | `actions_applied`, `passes`, `decided_passes`, `births` within the ordinary thresholds; `mesh_hops`, `vm_steps` may move by the widened draws | a severe level: attribute before accepting; a compute change beyond the catalog widening and the draws is a defect |

**Measured verdict.** Pending.

- Summaries of the declared profiles: [gate](../../progress/features/t27-f03-giving-energy.json),
  [goal](../../progress/features/t27-f03-giving-energy-goal.json).
- Full readings: [`docs/progress/readings/t27-f03.md`](../../progress/readings/t27-f03.md).

## Success Criteria

- [ ] A committed give moves `min(1, give_share × effort) × giver.energy` to the neighbour through `gift_debit` / `gift_credit` with the recipient capped; a committed steal takes `min(1, steal_share × effort) × victim.energy` through the predation flows; both charge `action_effort(exchange.effort_rate × intensity × actor.energy)` with the floor; a zero transfer is a rejection with no kill credit.
- [ ] `steal_share` and `give_share` are inherited with the capacity's rule from the founder's `exchange.founder_share`, stay in `[share_min, 1]`, and are visible in the creature detail and the goal summary's `shares` block.
- [ ] `VoteSink::Directed` with four kinds, 39 vote sinks, five parameter fields, `vote-sinks.json` pinned and consumed by the frontend; `StealEnergyAmount` and `predation.steal_cost_rate` are gone.
- [ ] Viability passes first; gate and goal summaries stored and epochs re-pinned; the goal readings report steals, gives, kills and shares per world, zero included; mutation gate and slow tier run with every survivor resolved.

## Notes for AI Agents

- Decision: in-feature refactors R1–R7 and R10 of the preparatory audit in
  `docs/progress/readings/t27-f03.md` are this feature's scope (the `Directed`
  shape, the fixture, the charge arrays, the returned transfer result,
  `ExchangeConfig`, the shared scalar inheritance, `action_type() ->
  ActionType`, the window-seam oracle T26.F02 deferred to the next feature
  touching the seam); R8 and R9 are accepted conventions with no marker.
- Decision: the charge base for both acts is `exchange.effort_rate ×
  intensity × the actor's own energy at the charge`, so a steal's effort is
  known before its target resolves; the effort modifier is `2^clamp(x, −2,
  2)`; the intensity is `min(1, share × modifier)`.
- Decision: the share draws are on every birth after the capacity draw,
  `steal_share` then `give_share`, under one `mutation.share` config.
- Deferred: the long-run reading of giving, stealing and the share
  distributions runs on T27.F02's goal-long, the track's one carrier; this
  feature's 2,000-tick reading reports them, zero included.
