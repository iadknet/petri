# T11.F20 — Per-Birth Supply Rule Retirement

**Status**: In Progress
**Last updated**: 2026-09-23
**Feature**: T11.F20
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

One way to mutate. The per-unit draw, `Binomial(units, per_unit_rate)`, is the
only mutation-count rule in the engine, the config, the panel, and the
reference specs. Production births draw it on the parent's own
`genome_size()`, exactly as today. The drift walk and the recruitment-paths
experiment draw it on a fixed unit count equal to the canonical V3Alpha1
founder's `genome_size()` (97), so their exposure is an instrument constant
(about 0.485 requested events per birth at the default rate), not a second
configurable rule. Removal of a rule with no natural analog; the per-unit
draw's analog (per-base copy error) is stated at T11.F19.

## Non-Goals

- No production behavior change: no new rate, cap, cost, or target draw; the
  default `per_unit_rate` stays 0.005.
- No epoch re-pin and no attempt to remove the goal severe on `decided_passes`
  (vs T19.F04) or the gate `pass_cap_hits` flag, both of which were accepted
  earlier.
- No edits to historical records: the T11.F16/T11.F17 depth-2,000 rows, the
  T13.F06 reading, closed specs, strategy notes, `docs/progress/` summaries,
  sweeps and readings, and `docs/specs/runtime-config-apply-fixes.md` stay as
  they are. The drift floors are not reinstated.
- No T11.F13, T03.F11 or T11.F10 work. No new environmental pressure, so the
  three-world integration rule does not apply.

## Inputs and Invariants

Sources of truth: the track's T11.F20 row and its Notes entry ("T11.F20 (added
2026-09-14 …)"), the T19 execution-contract Notes entry, the "Floors and the
no-regression rule" and "Drift floor history" Notes, and the
[T11.F19 spec](t11-f19-per-unit-mutation-supply.md) (Goal, Inputs and
Invariants, Performance). The code:

- `crates/v3-core/src/config/simulation.rs`: `MutationConfig`
  (`#[serde(deny_unknown_fields)]`, with required keys `mutation_probability`,
  `per_birth_mutation_events_min`, and `per_birth_mutation_events_max`),
  `with_legacy_supply`, `normalize`, and the founder pin test
  (`per_unit_rate × founder.genome_size()`).
- `crates/v3-core/src/mutation/engine/mod.rs`: `requested_event_count` and
  `apply_mutations_with_food_type_count`.
- `crates/v3-core/src/neighborhood/{drift.rs,births.rs}`: `observe`,
  `observe_checkpoint`, `supply_rule`, `VERSION`, and `per_birth_result`.
- `crates/v3-core/src/neighborhood/recruitment_paths/{records.rs,experiment.rs,tests.rs}`:
  `Supply::{Legacy,Production}`, `proposal_mutation_config`,
  `mutation_context`, and the pinned-baseline tests.
- The founder assertion `genome.genome_size() == 97` in
  `crates/v3-core/src/creature/state.rs`.
- The seven tracked root `world-recipe-*.json` files, each carrying all six
  supply keys.
- The goal recipe digest pins in `crates/v3-cli/tests/bench_artifacts.rs`.
- `frontend/src/components/config-panel/runtime/{MutationSection.tsx,bounds.ts,bounds.test.ts}`,
  `frontend/src/types/config.ts`, `frontend/src/test/fixtures.ts`,
  `ConfigPanel.test.tsx`, and `ControlBar.test.tsx`.

**Production-identity baseline.** The feature branches from `2c09fd76`. Every
commit since T19.F06's benchmark revision `a81276de` is test-only or docs
(`git diff a81276de 2c09fd76 -- crates` touches only `#[cfg(test)]` code), so
T19.F06's reports are the matching pre-T11.F20 reports:
[gate](../../progress/features/t19-f06-retirement-and-observability.json),
[goal](../../progress/features/t19-f06-retirement-and-observability-goal.json).
Their raw reports are verified local at
`.bench-artifacts/t19-f06-retirement-and-observability/{gate,goal}.json` in the
main checkout. Pinned epochs (unchanged): gate
`t19-f04-vote-based-action-selection.json`, goal-worlds
`t19-f04-vote-based-action-selection-goal.json`. T11.F19's reports are
historical context only.

Research summary (local evidence; no external dependency is at stake):

| Decision | Options | Adopted and why |
| --- | --- | --- |
| Stored config or recipe carrying a retired key | reject; ignore through per-field skip attributes | **Reject.** Every config struct is `deny_unknown_fields`. T19.F06 retired `action_queue_cap` the same way (`02d33611`, `retired_queue_cap_key_is_rejected`), and AGENTS.md makes backward compatibility a non-goal. Ignoring the keys would let stale recipes load silently. |
| Instrument draw | Binomial(founder units, rate); the walked genome's own size; an exact 0.55 through a founder-scaled rate; exactly one event per birth | **Binomial(97, rate)**, as the row proposes. It uses the production code path and rate, so a change to either shows up in the walk. The fixed units keep the walk finite (under drift, the own-size draw doubles the mesh every 20 to 30 generations; T11.F19 note, Section 5.2). The mean of 0.485 stays within 12% of the old 0.55. A rate scaled to reproduce 0.55 would carry a number from the retired rule. One event per birth would remove zero-event births (61.5% under the pinned draw). |

Invariants:

1. **One count rule.** `requested_event_count` has a single body: one
   `gen_bool(per_unit_rate)` per unit, one event per success. Its unit count
   comes from the caller. Every production birth and every production reading
   passes the parent genome's own `genome_size()`, read once. This covers
   reproduction, the founder and evolved neighborhood halves, and T14.F12's
   read. Outside engine-level tests (invariant 5), the only other value any
   caller passes is the instrument constant below. No config field, `cfg(test)` switch, or simulation-path override
   selects a count, and no field of `MutationConfig` names one.
2. **Instrument constant.** The instrument unit count is the existing
   `creature::founder::FOUNDER_GENOME_SIZE_UNITS` (97), which
   `the_canonical_founder_genome_is_ninety_seven_units` in `creature/state.rs`
   already pins to the canonical V3Alpha1 founder's measured `genome_size()`.
   No second anchor and no bare `97` is added.
   The rate is the `per_unit_rate` of the config the instrument is handed:
   `drift::observe` gets the case config, and `recruitment_paths` gets
   `MutationConfig::default()`. Every drift walk birth, every drift checkpoint
   birth (`observe_checkpoint` into `per_birth_result`), and every
   recruitment-paths legacy-panel proposal draws on it. The T13.F07 S0 panel
   (`Supply::Production`) keeps the child's own size.
3. **Production identity.** Compare matched per-unit configurations and seeds
   with `2c09fd76`. Every production birth's RNG consumption is
   byte-identical. The exhaustive list of `deterministic` paths allowed to
   differ, gate and goal, is: each goal case's `drift_depth`; the goal
   `recruitment_paths` block (including its `config` and `config_digest`);
   and each goal case's `config_digest` in `profile` and `goal_indicators`
   (five keys fewer). Every other `deterministic` field is byte-identical,
   including work counters, persistence rows, founder neighborhood rows,
   T14.F12 read rows, and `mutation_supply`. Outside `deterministic`, the
   gate's `measurement_evidence.effective_config_digest` changes for the same
   reason. The short-run identity hash in
   `legacy_default_short_run_identity` (`crates/v3-core/tests/baseline_worlds.rs`)
   stays `7548337837478651677`. The founder pin test stays unchanged.
4. **Rejection.** A config, recipe, or runtime patch that carries any of
   `mutation_probability`, `per_birth_mutation_events_min`,
   `per_birth_mutation_events_max`,
   `per_birth_mutation_event_continuation_probability`, or
   `per_unit_supply_enabled` is rejected by the existing strict
   deserialization, with no alias. The seven root recipes drop these keys and
   keep `per_unit_rate`. Committed benchmark summaries, which still embed the
   keys, remain loadable as comparison references, because
   `comparison_inputs_from_bytes` reads a summary's `comparison_inputs`, not
   its `deterministic` payload. The typed full-report branch is unaffected:
   no series reference is a full report, and none of the 97 historical T15.F02
   blobs carries a retired key (readings). Post-T15.F01 goal raws embed
   `recruitment_paths.config`, so typed decoding (`bench-summarize`, a raw
   passed as a reference) of goal raws written before this feature stops
   working. This is deliberate and not shimmed. Main already cannot decode
   any goal raw older than T19.F06, because `02d33611` removed
   `mutation.action_queue_cap` from the strict `MutationConfig` (the T19.F05
   raw carries `"action_queue_cap": 4`). A shim would therefore rescue only
   T19.F06's goal raw, which is already summarized. Every such raw was
   converted at its own closure, and its committed summary is the reference.
5. **Tests keep their properties.** Tests that forced a fixed count re-express
   it through the per-unit rate. Engine-level tests may also pass an explicit
   unit count; for example, rate 1.0 on N units requests exactly N events,
   which replaces the old min = max = N at trigger 1.0. Each keeps its asserted
   property. Values pinned under the legacy rule by test-only fixtures are
   re-pinned and recorded. Production pins (invariant 3) never move. No
   viability assertion is weakened; a viability failure is escalated.
6. Accounting is unchanged: `attempted = applied + skipped`, skipped events
   are never retried, and rate 0.0 requests nothing. The T11.F19 property
   tests (`0 <= requested <= units`, rate 0 gives 0, rate 1 gives units, and
   rate normalization) stay, stated on the caller-supplied unit count.

Fixed design:

| Surface | Change |
| --- | --- |
| `MutationConfig` | Delete the five fields, their defaults, the normalization lines, `with_legacy_supply`, and `default_per_unit_supply_enabled` / `default_mutation_event_continuation_probability`. Add a rejection test for each retired key. |
| Engine | Delete the legacy branch. The count draw's unit count is supplied by the caller, through an extra argument or a sibling entry point (implementer's choice). Production call sites pass `genome_size()`. |
| Drift walk | `observe` stops overriding the config and draws walk births and checkpoint births on the instrument constant. `DriftDepth::VERSION` becomes `drift-depth-v4`. `supply_rule` names the pinned draw, its unit count, and its rate (for example `per-unit draw on the canonical V3Alpha1 founder's genome_size() 97 at per_unit_rate 0.005: Binomial(97, 0.005) events per walk and checkpoint birth`). Doc comments in `crates/v3-cli/src/bench/schema.rs` note v4. |
| Recruitment paths | `Supply::Legacy` becomes a variant, with its serialized name and `rule()` string, that names the founder-pinned per-unit draw. `mutation_context` says the same. `VERSION` stays `recruitment-paths-v1`, following the re-pin precedent of T11.F22, T17.F02, and T19.F04. The pinned tests (`production_prepared_lineages_match_the_recorded_baseline_and_metadata`, `production_cost_selection_lineages_pin_the_first_reading`, and the others that read the legacy panel) are re-pinned with a doc line. Their selected-inapplicable discard counts stay `[0, 0]`. |
| Frontend | Remove the "Per-Unit Supply" toggle, the four legacy rows, `LEGACY_ONLY`, the min/max cross-field bounds, and their types, fixtures, and tests. "Rate / Unit" stays. |
| Reference docs | `v3-mutation-spec.md` (Section 4.1 and the selection randomization rules; the founder figure becomes 97 units / 0.485); `v3-runtime-config-spec.md` (mutation table; "Mutation randomization semantics" steps 1 and 2); `v3-reproduction-spec.md` (supply paragraph); `v3-server-api-protocol-spec.md` (both mutation examples). Each states one rule, and says the two instruments draw it on the founder's size. |
| Recipes and pins | Strip the five keys from the seven root recipes. Re-pin the three goal recipe digests. Update the v3-server tests and v3-cli bench tests that set the keys. |

## Implementation Tasks

- [ ] `cargo test -p v3-core --test viability` is the first test command of
      implementation, run as a pre-change baseline. Then config, engine, and
      rejection (invariants 1, 4, 6), test first. Once `MutationConfig` changes
      shape and compiles, viability is again the first test command run.
- [ ] Instruments on the founder-pinned draw: drift `v4` and `supply_rule`;
      recruitment-paths `Supply`, `mutation_context`, and re-pins
      (invariant 2).
- [ ] Re-express every fixed-count test (invariant 5) across v3-core, v3-cli,
      and v3-server.
- [ ] Root recipes and goal recipe digest pins.
- [ ] Frontend panel, types, fixtures, and tests.
- [ ] The four reference docs.
- [ ] Gate and goal benchmarks, with readings in
      `docs/progress/readings/t11-f20.md`.

## Verification

- [ ] `cargo test -p v3-core --test viability` passes as the first test
      command of implementation and again as the first after the shape
      change. Output goes in readings.
- [ ] Focused tests: retired-key rejection (config, recipe, runtime patch);
      the instrument-constant pin; the drift walk drawing on the constant
      whatever the walked size; recruitment-paths re-pins; the short-run
      identity hash unchanged; the founder pin unchanged. Names and results go
      in readings.
- [ ] Retirement is complete. `rg` over `crates/`, `frontend/src/`,
      `docs/reference/`, and the root recipes for the five retired names,
      `with_legacy_supply`, and "Per-Unit Supply" finds only the rejection
      tests. Command and output go in readings.
- [ ] `make check` passes in the worktree.
- [ ] Production identity (invariant 3). Diff the new gate and goal raw
      reports' `deterministic` sections against T19.F06's raw reports. The
      diff lists only the permitted paths, and both reports' comparisons load
      the T19.F04 and T19.F06 summaries. The path list goes in readings.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t11-f20-per-birth-supply-rule-retirement.json`
      and `…-goal.json`, with local raw hash, byte count, and verification
      time checked. Series entries point to them, `epoch_baseline` is
      unchanged, and no full report is staged.

## Performance and Goal Impact

**Predeclaration — written before the run.** This retires a rule with no
natural analog. It adds no sensor and no pressure. Production compute is
unchanged: the same draws, and one fewer branch per birth. The instruments now
make 97 `gen_bool` draws per birth where the old rule made one or two. That is
about 10.7 M draws per goal case for the walk (50 lineages × 2,000 generations,
plus 5 × 2,000 checkpoint births) and about 8 M for the 82,944 recruitment
proposals, on the order of 10 ms. With about 12% fewer requested events, the
mutation work drops by about the same amount.

References: the previous closure is T19.F06 for both the gate and the goal;
the epochs are T19.F04. Thresholds: counters flag at 10% and are severe at
50%; wall flags at 25% and is severe at 100%. The goal profile runs once.
Neither epoch is re-pinned. Pre-feature numbers are tabulated in
[readings](../../progress/readings/t11-f20.md).

| Reading | Predeclaration |
| --- | --- |
| Gate and goal work counters, persistence, founder neighborhood, T14.F12 read, `mutation_supply`, all against T19.F06 | Identical as exact integers per seed and world (invariant 3). Any difference does not fit the predeclaration and is escalated before anything is presented. |
| Same counters against the epoch (T19.F04) | Exactly T19.F06's comparison. Gate: `pass_cap_hits` flag +35.106383%, not severe. Goal: `severe=true` on `decided_passes` +255.434783% and a `pass_cap_hits` flag +22.682268%, so the goal CLI exits 3 and `make` exits 2. That severe is the one the user accepted on 2026-09-22 at T19.F05. It is reported with its acceptance record and not remediated. |
| Config identity | Goal case digests change in all three worlds (`inputs_changed` true, comparable), as do `recruitment_paths.config` and `config_digest` and the gate's `effective_config_digest`. |
| Drift walk: version and `supply_rule` | `drift-depth-v4`, and the string names the founder-pinned draw |
| Drift walk: requested events per checkpoint birth (`by_requested_events`) | This is the powered check that checkpoint births draw on the instrument constant, not on the walked genome's size. At every checkpoint and world, the zero-request share is within [0.58, 0.65] (Binomial(97, 0.005) gives 0.615; 2,000 births; about ±3σ). The mean requested count per birth is within [0.44, 0.53] (expected 0.485). A miss does not fit the predeclaration. |
| Drift walk: changed per all births, per checkpoint | Heuristic expectation: about 0.875 of the pre-feature count (the event-bearing share falls from 0.44 to 0.385, and the walk accumulates about 12% fewer events per generation). The walked trajectories themselves change with the draw, so the scaling is not exact. Escalation tripwires are the Poisson 1% bound of 0.875 × pre-feature, in counts per 2,000 (Orchards and Confluence / Canyon): depth 0: 245 / 239; depth 22: 147 / 137; depth 250: 30 / 28; depth 1,000: 12 / 6; depth 2,000: 1 / 0. A reading below a tripwire is escalated before closure. Any checkpoint below half its pre-feature count, including a zero, is reported with an attribution before its row is adopted: the event-bearing share, and changed per event-bearing birth (`any_events.changed / any_events.applied`) against the pre-feature value. The operators and target draw are unchanged, so with the requested-events check met, such a drop is read as the draw change, and the row is adopted as the new per-world reference under the no-regression rule. |
| Drift walk: dead per all births; executed nodes at 1,000 and 2,000 | No direction, because trajectories change. Reported against the pre-feature values: dead as tabulated in readings; executed nodes 7.42 / 6.98 / 7.42 at depth 1,000 and 9.60 / 10.06 / 9.60 at depth 2,000. |
| `recruitment_paths` block | `arms`, `pairs`, `opportunities`, `constructed`, and `total_proposals` (82,944, unchanged) are reported before and after, with no direction. The version is unchanged. The discard counts in the pinned tests stay zero. |
| Wall | The standing absolute budgets apply on any host: founder neighborhood under 10 s; summed evolved neighborhood under 180 s; the whole goal run is investigated past 15 minutes; `drift_depth_wall_clock_ms` (one aggregate over the three worlds) under 30 s, the T11.F19 budget. Relative readings hold only on the T19.F06 host (`Isaacs-MacBook-Pro-2.local`); on another host they are report-only. They are: aggregate `drift_depth_wall_clock_ms` not above 24,370.5 ms + 10%, and `recruitment_paths_wall_clock_ms` within 10% of 12,609.0 ms. Gate and goal wall per creature-tick have no direction. |

**Measured verdict.** Pending.

- Summaries: [gate](../../progress/features/t11-f20-per-birth-supply-rule-retirement.json),
  [goal](../../progress/features/t11-f20-per-birth-supply-rule-retirement-goal.json).
- Full readings: [`docs/progress/readings/t11-f20.md`](../../progress/readings/t11-f20.md).

## Success Criteria

- [ ] `MutationConfig` carries only the per-unit rate for supply. A config
      carrying a retired key is rejected. No panel control, reference doc, or
      recipe names the per-birth rule.
- [ ] Production is byte-identical to `2c09fd76` apart from the permitted
      paths (invariant 3).
- [ ] The drift walk (`drift-depth-v4`) and recruitment paths draw on the
      founder-pinned per-unit count. Their new rows are recorded as the
      reference, and no tripwire is crossed.

## Notes for AI Agents

- Decision: Fable credits were exhausted, so at the user's direction no role used Fable 5.1 — the spec owner ran on Opus 5.5 (Agent model parameter), implementer briefs were told not to consult the advisor, and the orchestrator ran no /advisor fable and consulted no advisor.
- Decision: the drift walk and the recruitment-paths legacy panel draw `Binomial(canonical founder genome_size(), per_unit_rate)` (track row, user direction 2026-09-14/2026-09-21); a later change to the canonical founder's size or the default rate moves both instruments' rows and must bump `drift-depth` and re-pin the recruitment-paths tests.
