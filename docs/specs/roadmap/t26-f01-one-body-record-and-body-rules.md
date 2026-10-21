# T26.F01 — One Body Record and Body Rules

**Status**: In Progress
**Last updated**: 2026-10-20
**Feature**: T26.F01
**Track**: [T26 — Body Traits](../../roadmaps/t26-body-traits.md)

## Goal

Heredity builds a body as well as a brain, as an egg carries its pigment and
markings beside its nervous system: the six-channel colour phenotype and the
kin identity travel in one heritable body record, passed to each
`CreatureState` constructor as one argument and inherited by one call; the
upper energy cap and the effort of a move each come from one body rule that
every caller uses. Nothing a creature does changes: every trajectory pin,
serialization fingerprint and mutation draw is identical to main's, and any
pin that moves is a defect.

## Non-Goals

- No value changes: the founder phenotype, the kin-tag rule, `max_energy`,
  `move_cost`, the effort floor and multiplier, and the
  `applied_events > 0` gate on phenotype and kin-tag mutation (inherited
  coupling, kept as the track says).
- No per-creature capacity, no carrying cost, no upkeep: T26.F02.
- The other readers of `max_energy` keep reading the config value: sensor
  normalization denominators, reproduce thresholds and the v3-lab birth
  sentinel. T26.F02 rules on each.
- Eat's zero floor and observe-after-cap order, predation's
  observe-before-cap order and its signed reverse transfers, and birth's
  direct litter assignment stay as they are.
- No wire or file format change: `CreaturePhenotypeResponse`,
  `phenotype_rgb`, the lab `summary.json`, the benchmark deterministic
  blocks.
- No config field, no telemetry signal, no frontend change.

## Inputs and Invariants

Inputs: the T26 row and Notes; the
[design note](../../strategy/interaction-features-design-2026-10-20.md) §2
and §4; [review](../../progress/readings/t26-t28-design-review.md) findings
2 and 12; the preparatory audit and the options weighed, in
[`docs/progress/readings/t26-f01.md`](../../progress/readings/t26-f01.md).

The code this feature changes, as it stands on main at `ced01959`:

| Site | Today |
| --- | --- |
| `creature/state.rs:316-440` | `new` and `new_with_cached_fields` take ten and thirteen positional arguments, four of them heritable non-brain state (`phenotype_channels`, `phenotype_active_channel`, `phenotype_channel_polarity`, `identity`), and repeat a 45-line body; 53 call sites (51 `new`, 2 `new_with_cached_fields`) across `v3-core` and `v3-lab`, including `v3-lab/tests/exploration_probe.rs` |
| `simulation/seeding.rs:18-20,96-111` | the founder phenotype constants are private; 26 test sites copy the literal `[0, 0, 92, 92, 138, 138]`, and the lab's `Phenotype::founder` seeds a simulation to read it (`v3-lab/src/eval.rs:36-70`) |
| `simulation/actions/reproduction.rs:388-390,518-536` | the child's phenotype and identity are read field by field, then mutated inline: phenotype (`mutate_phenotype`) first, then `mutate_kin_tag`, each behind its own `summary.applied_events > 0` test |
| `simulation/actions/mod.rs:107-114` | eat: `credited.clamp(0.0, max_energy)`, books `maximum_energy_clamp_loss` and `zero_floor_credit`, observes once after |
| `simulation/actions/predation.rs:105-117,126-138` | two attacker caps (`if energy > max_energy`), each after an observe, booking `maximum_energy_clamp_loss`; the transfer credit and kill bonus are booked as signed negatives of `observe_energy` |
| `simulation/actions/mod.rs:156-161` | the move charge is `pay_effort(creature, &config.energy, costs.move_cost, ActionMove)`, charged on success and on a blocked move |
| `docs/reference/v3-reproduction-spec.md:286-288` | the step list writes kin-tag mutation before phenotype mutation; the code and its draw order are the reverse |

Invariants:

1. **The record.** `crates/v3-core/src/creature/phenotype.rs` is a leaf
   module holding
   `Phenotype { channels: [u8; 6], active_channel: usize, polarity: [bool; 6] }`
   and `Phenotype::FOUNDER`; `crates/v3-core/src/creature/body.rs` holds
   `HeritableBody { phenotype: Phenotype, identity: CreatureIdentityState }`.
   Both are `Copy` with public fields. `body.rs` imports `mutate_phenotype`
   from `mutation/phenotype.rs`, which imports only the leaf, so there is no
   import cycle and the mutation keeps taking and returning the record. `CreatureState` carries one `body` field
   in place of the four loose fields; every reader (`sensors/reducers.rs`,
   the server's `state.rs` and `http/creature.rs`, tests) reads through it.
   The lab's own `Phenotype` type and its always-founder `Setup.phenotype`
   field are deleted; lab creatures take `HeritableBody::founder(0, seed)`,
   the values they had. The server response is built from the record with
   the same values.
2. **The founder.** `Phenotype::FOUNDER` is the public constant the private
   `seeding.rs` constants become, and `HeritableBody::founder(founder_index,
   startup_seed)` pairs it with `CreatureIdentityState::founder`; seeding
   calls it, and the lab's seeded-simulation probe for the founder
   phenotype is replaced by the constant. A test fixture replaces its
   literal copy with the constant and
   keeps the identity value it uses today (`CreatureIdentityState::default()`
   or `founder`), so no fixture's values change.
3. **One inheritance call.** `HeritableBody::inherit(&parent, mutated: bool,
   &PhenotypeConfig, rng) -> HeritableBody` copies the parent's record and,
   when `mutated`, mutates the phenotype first and the kin tag second with
   the same draws in the same order as today; when not `mutated` it draws
   nothing. Reproduction passes `summary.applied_events > 0` and keeps
   owning when the trigger is evaluated; the body record owns the order; the
   identity and phenotype modules keep owning how each mutates.
4. **Constructors.** Both constructors take `body: HeritableBody` in place
   of the four arguments; `new` computes the three caches it computes today
   and delegates to `new_with_cached_fields`, so the construction body
   exists once.
5. **The cap rule.** `CreatureState::energy_cap(&self, lifecycle:
   &EnergyLifecycleConfig) -> f32` returns `lifecycle.max_energy`; its
   readers are the three credit sites and the server's creature detail
   `max_energy` field (`v3-server/src/http/creature.rs`), which reports the
   same value today and the creature's own capacity once T26.F02 lands.
   One shared step in
   `simulation/actions/mod.rs`, beside `pay_effort`, applies a cap to a
   creature's energy and books the excess into
   `flows.maximum_energy_clamp_loss`, in the comparison form the predation
   sites use today (`if energy > cap`). `energy_accounting.rs` stays
   observation-only (its header contract, T14.F03): it keeps supplying the
   booking arithmetic (`applied_debit`), the creature supplies the cap
   value, and the energy mutation happens in the action layer. Eat applies
   its zero floor and books
   `zero_floor_credit` before that step and observes once after it; each
   predation site observes before it; the kill bonus keeps its own step;
   births assign the litter energy directly with no cap. For every input,
   each site's result and bookings equal today's: a credit above the cap is
   capped (including `+∞`), eat floors a negative credit to zero (including
   `−∞`), and a `NaN` energy stays `NaN` at every site as `clamp` and the
   predation comparison leave it today — eat's floor is therefore a
   comparison, not a `NaN`-suppressing `max(0.0)`. The characterization
   tests cover these cases and their bookings.
6. **The move rule.** `CreatureState::move_effort(&self, energy:
   &EnergyConfig) -> f32` returns `energy.action_effort(energy.costs.move_cost,
   self.cached_complexity, self.age)`; `apply_move` charges exactly that
   amount through the shared charge path (`pay_effort` is split into the
   effort computation and the charge, which observes `ActionMove`) on a
   successful and on a blocked move, booking `action_charges.r#move` and
   `failed_action_effort.r#move` as today. No other production site reads
   `costs.move_cost` as a charge.
7. **Determinism and pins.** No RNG draw is added, removed or reordered; no
   tick-phase order changes. `tests/applied_trajectory.rs`,
   `tests/reproducibility.rs` and `tests/viability.rs` pass with their pins
   untouched; the gate's deterministic blocks are byte-identical to the F09
   epoch; the recruitment-paths slow-tier readings are unchanged.
8. **Reference docs.** `v3-reproduction-spec.md` steps 10–11 state the
   code's order (one body inheritance call: phenotype, then kin tag);
   `v3-phenotype-spec.md` §§2–5 are brought to today's code, which they
   predate (`b9cb8beb`): the record's six-channel `Phenotype` fields in
   place of the three-channel RGB state, six-way polarity, the founder
   channels `[0, 0, 92, 92, 138, 138]` beside their RGB, a channel switch
   among the five other channels, and the detail response's actual
   exposure of `active_channel` and `polarity`
   (`v3-server/src/http/creature.rs:491-495`);
   `v3-creature-identity-spec.md` §4 names the body record as the carrier of
   the one call. No runtime rule changes; the documents describe the code.

## Telemetry

Not applicable: a behavior-preserving refactor that adds no counter,
mechanism switch or config value; every existing signal derives from
unchanged applied behavior.

## Verification Plan

**Question.** F01 claims the body record and the two body rules can replace
the loose arguments and the inlined cap and charge sites with no behavior
change; a differing gate deterministic block, a moved trajectory pin, a
changed draw stream or a recruitment-paths fingerprint mismatch would show it
wrong.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate` | The change touches tick-loop, action and reproduction code with no population reading in question; the gate's deterministic blocks against the F09 epoch are the byte-identity reading, in seconds. |
| Mutation gate | `run` | The cap step, the move rule and the body inheritance call are changed `v3-core` production logic whose behavior the characterization tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | `crates/v3-lab/src/eval.rs` changes (its phenotype type and field paths), and the tier holds the lab's end-to-end runs; it also pins the recruitment-paths readings the constructor change must leave identical. |

## Implementation Tasks

- [x] Characterization tests first, on main's behavior, in one commit, run
      after `cargo test -p v3-core --test viability`: eat (floor, cap, both
      bookings, observe order, and the `+∞`, `−∞` and `NaN` credit cases),
      predation (cap after the transfer and after
      the kill bonus, observe before each cap, signed reverse flows), a birth
      whose litter exceeds `max_energy` kept uncapped, the move charge on a
      successful and a blocked move, and the inheritance draw order
      (phenotype then kin tag when mutated, pinned on main's child-body
      values). The unmutated no-draw case is not observable through
      `apply_reproduce` (genome mutation consumes the same stream first), so
      it is pinned directly on `HeritableBody::inherit` by a property test,
      with the unchanged trajectory and reproducibility pins as the
      end-to-end reading.
- [x] `creature/phenotype.rs`: `Phenotype`, `Phenotype::FOUNDER`;
      `creature/body.rs`: `HeritableBody`, `founder`, `inherit`;
      `mutate_phenotype` over
      `Phenotype`; `seeding.rs` uses `founder`; the lab's `Phenotype`
      deleted (invariants 1–3).
- [x] Constructors take `body`; `new` delegates to `new_with_cached_fields`;
      the 53 call sites and every reader migrated; test literals replaced by
      `Phenotype::FOUNDER` with their identity values kept (invariants 2, 4).
- [x] `energy_cap` and the shared cap step; eat and both predation sites use
      them (invariant 5).
- [x] `move_effort` and the split charge path; `apply_move` uses them
      (invariant 6).
- [x] Reference docs corrected: reproduction step order, phenotype-spec
      §§2–5 synchronized with the six-channel model and transport
      visibility, identity-spec §4 (invariant 8).
- [ ] Closing: gate summary, mutation record, slow tier result recorded in
      Verification.

## Verification

- [x] Characterization and inheritance-order tests:
      `cargo test -p v3-core` -> 2326 passed, 0 failed, 10 ignored; names listed in
      `docs/progress/readings/t26-f01.md`.
- [x] `cargo test -p v3-core --test viability` -> 30 passed, 0 failed;
      `tests/applied_trajectory.rs` and `tests/reproducibility.rs` pass with
      unchanged pins.
- [x] `cargo test -p v3-lab -p v3-server -p v3-telemetry -p v3-cli` -> 718
      passed, 0 failed, 19 ignored.
- [x] `cargo check --workspace --all-targets`, `cargo clippy --workspace
      --all-targets` and `cargo fmt --all --check` -> clean.
- [ ] `make rust-test-slow` -> pending (recruitment-paths readings and
      lab end-to-end runs unchanged).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants` -> pending: summary line,
      output path, every survivor resolved as killed, equivalent, or deferred.
- [x] Gate (`make bench PROFILE=gate FEATURE=t26-f01-one-body-record-and-body-rules`,
      HEAD 52fc5b88): v3-cli exit 0 (`measurement_evidence.cli_exit`), outer
      `make` exit 0 (observed from the shell; the summary's `outer_exit` is
      null); `severe=false`; every deterministic counter equal to the F09
      epoch (delta 0.000000%, level ok). Summary
      `docs/progress/features/t26-f01-one-body-record-and-body-rules.json`
      (26,107 bytes); local raw
      `.bench-artifacts/t26-f01-one-body-record-and-body-rules/gate.json`
      (129,588 bytes, sha256 `46f9acef...ab62b27`, verified_local); the raw
      report is not staged.
- [ ] `make check` -> pending.

## Performance and Goal Impact

**Predeclaration — written before the run.** A refactor: no mechanism
reaches creatures, no indicator has a direction, no compute change is
expected (the record is a `Copy` of the same bytes; the cap and move rules
are one call each at sites that already did the arithmetic). The gate's
wall-clock is compared against the F09 epoch under the ordinary thresholds
and its deterministic blocks must be byte-identical; a severe flag, a
threshold crossing or any deterministic difference is a defect to fix, not a
cost to accept. Predeclared changes by artifact: trajectory pins none,
serialization fingerprints none, mutation draws none. Predeclared re-pins:
none.

**Measured verdict (2026-10-21, HEAD 52fc5b88, gate run once).** v3-cli exit 0,
outer `make` exit 0, `severe=false`, no flag, no threshold crossing against the
F09 epoch. Byte-identity reading: all nine deterministic per-creature-tick
counters equal the F09 epoch to six places (mesh_hops 5.716312, vm_steps
0.031219, graph_relax_iters 5.664662, plasticity_updates 0.157256,
actions_applied 1.637232, births 0.026297, pass_cap_hits 0.004623, passes
2.688388, decided_passes 0.000584; 143,855 creature-ticks, 3,783 births, final
populations 634, 632, 645). Predeclared trajectory pins, fingerprints and
draws: unchanged; no re-pin. Exception to read, not a gate result:
`comparison.references[0].wall_clock.level` is `severe` (0.017019 vs epoch
0.002638 ms per creature-tick, +545%; total 2,448 ms vs 380 ms), a non-fatal
level (`wall_levels_fatal=false`, so `comparison.severe` stays false). Context
only, not a conclusion: right after the run other sessions' builds and tests
were running (load average about 26 on 8 cores; `bench-wait` gates only the
start), and the deterministic counters show no compute change. The profile was
not rerun (once per closure); the orchestrator decides whether the wall-clock
level needs a quiet-host rerun.

- Summary of the declared profile: [gate](../../progress/features/t26-f01-one-body-record-and-body-rules.json).
- Full readings: [`docs/progress/readings/t26-f01.md`](../../progress/readings/t26-f01.md).

## Success Criteria

- [ ] Phenotype and kin identity are one `HeritableBody`, one constructor
      argument on both paths, built for founders by one function and
      inherited by one call that keeps the phenotype-then-kin-tag draw order.
- [ ] The upper energy cap comes from `energy_cap` and one shared step at
      eat and both predation sites; the move effort comes from `move_effort`;
      every caller's floor, ordering and signed flows are unchanged, shown by
      the characterization tests.
- [ ] No pin moved: the existing trajectory, reproducibility and viability
      tests, the gate's deterministic blocks and the slow-tier readings are
      identical to main's.
- [ ] `make check` exits 0; the mutation record has every survivor resolved.

## Notes for AI Agents

- Decision: the preparatory refactors R1–R5, R7 and R8 of the audit in
  `docs/progress/readings/t26-f01.md` are this feature's scope, not
  prerequisites; none is deferred.
- Decision: the `applied_events > 0` gate on phenotype and kin-tag mutation
  is inherited coupling and stays (track Notes).
- Deferred: routing the other `max_energy` readers (sensor denominators,
  reproduce thresholds, lab sentinel) through a per-creature
  rule is T26.F02's, which also re-derives the lab test oracles that compute
  the move charge from config (`v3-lab/src/eval.rs` tests) when it scales it.
