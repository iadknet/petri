# T03.F08 — Genome Size Maintenance Cost

**Status**: In Progress
**Last updated**: 2026-09-07
**Feature**: T03.F08
**Track**: [T03 — Functional Traits and Metabolism](../../roadmaps/t03-functional-traits-and-metabolism.md)

## Goal

Every unit of brain structure a creature carries costs energy each tick
whether or not it ever runs. The charge is a per-tick carrying cost on
`genome_size()`, settled in the same Phase 0 step as `energy_decay_per_tick`
and removed by the same death rule, so junk costs something to carry and
selection sheds it through the existing deletion operators with no new
operator, cap, or pruning rule. The rate is sized so the two-node founder's
budget is barely touched. Natural analog: tissue is expensive to maintain even
when idle (the brain draws about a fifth of resting metabolism, and neurons
cost energy to keep alive, not only to fire), so unused structure is lost by
selection, the way bacteria shed pseudogenes under deletional bias (Mira,
Ochman, and Moran 2001). It reaches creatures through the body: energy only,
no sensor, no score, no bonus. What running costs is T03.F10's, not this
feature's.

## Non-Goals

- No cost on execution: steps, hops, evaluations, and plasticity are charged
  by T03.F10 and the existing opcode table, untouched here.
- No reachability-aware rate. The executed core pays the same per-unit rate
  as junk (the analog says idle tissue costs what active tissue costs to
  keep alive; a lower rate for executed nodes would need the reachability
  analysis that makes `energy.complexity_cost` blind to junk, and is a
  T11.F13 arm if ever wanted).
- No change to `energy.complexity_cost` (multiplies action costs by
  `complexity()`, excludes junk by design, disabled) or to
  `mutation.genome_size_pressure_enabled` (the cap lever, disabled). Both
  stay in the code as T11.F13's comparison levers; neither is enabled with
  this cost in production.
- No size cap, no pruning, no new mutation operator, no change to operator
  weights or targeting, no bonus for small genomes.
- No new sensor, introspection input, server status field, inspector
  annotation, or goal-profile indicator. The only new instrument is three
  fields on the CLI's existing `tick_sample` event (below), which the
  long-run reading needs and which derive from applied creature state.
- No change to the neighborhood battery, the drift walk, or their versions;
  neither runs Phase 0, so their rows are predeclared identical to T03.F10's.
- No scaling of the charge by the age or complexity action multipliers; like
  decay, it is a world-level Phase 0 cost.

## Inputs and Invariants

- Source intent: the owning track's T03.F08 row and detailed note (brought
  forward 2026-09-07 as the program's junk bound in place of the genome-size
  cap; dependencies removed; unit, rate, core-versus-junk rate, and
  composition with `complexity_cost` are the decisions this spec fixes).
- Evidence, from the [depth research note](../../strategy/mesh-depth-research-2026-09-07.md)
  Sections 3.1, 3.2, and 3.5 (the user's 281,405-tick run at median
  generation 1,990): brain execution charged 0.000152 energy per
  creature-tick, 0.03% of the 0.5 decay, and carrying structure charged
  nothing; live `genome_size()` p25 / median / p75 / max 5,970 / 6,246 /
  7,167 / 11,787 against the founder's 96; `complexity()` median 532; total
  mesh nodes median 124 with 19.5 reachable and 3.4 executed; total node
  count grew by drift at 0.0595 nodes per generation (correlation 0.83 with
  generation), and a drift walk with no selection reaches 140 nodes at
  generation 2,000, so selection today neither adds nor removes junk.
- Persistence references: T11.F04 `w1600` sweep
  (`docs/progress/sweeps/t11-f04/w1600.json`: final 11,610 / 10,398 / 11,093
  on seeds 11 / 22 / 33, mean energy 21.85 at tick 100 and 69.61 at tick
  2,000 on seed 11); T03.F10 goal report (final 1,786 / 3,712 / 1,306,
  minimum 959 / 1,366 / 536, plateau 1,449.40 / 2,830.52 / 900.31). Small
  worlds are not usable for a long run: the T11.F04 `w0256` and `w0512`
  sweeps go extinct on two or three seeds.
- Existing seams: `run_phase_0` in `crates/v3-core/src/simulation/tick.rs`
  (step 3 subtracts `energy_decay_per_tick`, step 4 removes `energy <= 0.0`);
  `CreatureGenome::genome_size` in `crates/v3-core/src/creature/genome/mod.rs`
  (every node, input ref, target, instruction, constant, compute node and
  its inputs, wired sink, slot, and gate; equal weights; includes junk; no
  reachability); `CreatureState` in `crates/v3-core/src/creature/state.rs`
  (genome immutable after birth; `cached_complexity` and
  `cached_reachable_nodes` computed in `new` and copied by
  `new_with_cached_fields` on the no-mutation fast path in
  `simulation/actions/reproduction.rs`); `EnergyLifecycleConfig` and
  `SimulationConfig::normalize` in `config/simulation.rs`
  (`normalize_f32_finite_nonneg`); the runtime config patch path and its
  frontend panel (`EnergyLifecycleSection.tsx`, `types/config.ts`,
  `test/fixtures.ts`, `ControlBar.test.tsx`); `TickSampleEvent` in
  `crates/v3-cli/src/lib.rs`; `docs/reference/v3-runtime-config-spec.md`
  Section 4 and `docs/reference/v3-tick-orchestration-spec.md` Phase 0
  sub-steps.
- Invariants: energy is conserved and behavior-backed (the charge is a
  subtraction from the creature's energy and nothing else); the genome is
  immutable after birth, so a size cached at birth is always current; with
  the rate at 0.0 every creature's energy trajectory is byte-identical to
  the pre-feature one.

Fixed design, decided before implementation:

| Decision | Value |
| --- | --- |
| Unit | `genome_size()` units, not mesh nodes. Junk lives inside nodes as well as between them: the live executed VM programs are 159 instructions long with 72 live (depth note Section 3.3), and a 500-instruction node carries more than a 19-instruction detour. Per-node pricing would leave intra-node introns free and reward packing junk into fewer, larger nodes. `genome_size()` already exists, counts everything with equal weights, and is what the size-pressure lever reads, so the two levers are comparable in T11.F13. |
| Rate | `energy.lifecycle.genome_carry_cost_per_unit`, `f32`, default `1e-4` energy per unit per tick. Finite and non-negative; invalid values fall back to `1e-4`; `0.0` disables the charge. Sizing from the evidence (corrected 2026-09-07 after implementation: the canonical founder is 111 `genome_size()` units at this revision, not the 96 the depth note and the track note carry from an earlier revision; the rate was not changed): the founder's 111 units pay 0.0111 per tick, 2.2% of the 0.5 decay; a live median executed core (`complexity()` 532) would pay 0.053 (10.6% of decay); the deep run's median genome (6,246) would pay 0.625, 1.25 times decay, so a creature carrying that much junk pays 2.25 times the founder's maintenance; a typical junk mesh node (about 55 units) pays 0.0055 per tick, about 1% of decay, which at goal populations of 1,000 to 4,000 is a selection coefficient the population can see. At the goal profile's depth (about 3 units of growth per generation, so about 250 units by generation 50) the charge is about 5% of decay, so persistence is not expected to move much. |
| Core versus junk | Same rate for every unit. Recorded as a design choice, not an assumption: the analog (maintenance is charged on tissue kept alive, whether or not it fires) says yes; a discount for executed structure would require the reachability analysis that already exists in `complexity()` and would recreate its blind spot on unreachable junk; and T11.F17 already aims variation at the executed core, so the core's exposure does not depend on a price break here. |
| Composition with `complexity_cost` | Independent. The carrying charge is a Phase 0 lifecycle cost beside `energy_decay_per_tick`; `complexity_cost` and `age_cost` multiply action costs only and are not applied to it, exactly as they are not applied to decay. Neither lever is edited. |
| Charge site | The Phase 0 energy-decay sub-step (numbered 3 in `run_phase_0`'s comment and 4 in the orchestration spec) becomes `energy -= energy_decay_per_tick + genome_carry_cost_per_unit * cached_genome_size as f32`, computed as one `f32` sum and one subtraction per creature per tick, immediately before the existing `lifetime_energy_sum` sample and step 4's `energy <= 0.0` removal. A creature whose charge takes it to or below zero dies in the same tick by the existing rule. The ulp of an `f32` at 200 is about 1.5e-5, so the founder's 0.0096 lands. |
| Cache | `CreatureState` gains `cached_genome_size: u32`, computed from the genome in `new`, passed through `new_with_cached_fields` from the parent on the no-mutation fast path beside `cached_complexity`, so the per-tick cost is one multiply-add and Phase 0 never walks a genome. |
| Determinism | Production behavior changes for every creature (every energy trajectory shifts by the charge), so no gate or goal `deterministic` block is predeclared identical to any prior report except the neighborhood and drift-walk blocks, which do not run Phase 0 and are predeclared identical to T03.F10's. Cross-process reproducibility is unchanged in kind and remains covered by `crates/v3-core/tests/reproducibility.rs`. |
| CLI instrument | `TickSampleEvent` in `v3-cli` gains `mean_genome_size`, `mean_mesh_nodes`, and `mean_generation` (`f64`, means over the living population at the sample tick, computed only at sample ticks from `cached_genome_size`, `genome.nodes.len()`, and `generation`; `0.0` when the population is empty). This is the long-run reading's instrument and the only telemetry added. |
| Frontend | `LifecycleEnergyConfig` gains the field; `EnergyLifecycleSection.tsx` gains one row (label "Carry Cost / Unit", min 0, max 0.01, step 0.00001, default 0.0001, tooltip naming the per-tick charge on `genome_size()`); fixtures and the control-bar fixture carry it. |
| Reference docs | `v3-runtime-config-spec.md` Section 4 gains the row and a "Genome carrying cost" note (unit, site, not scaled by the action multipliers, not applied to junk-excluding `complexity()`); `v3-tick-orchestration-spec.md` Phase 0 energy-decay sub-step names the combined subtraction. |

Predeclared readings, taken from the stored closure reports and read against
the T03.F10 gate and goal reports (previous closure), the pinned goal epoch
(T11.F17), and the gate epoch `remove-complementary-nutrition`:

| Reading | Reference | Predeclaration |
| --- | --- | --- |
| Goal persistence (1600², 10,000 founders, seeds 11/22/33, 2,000 ticks) | T03.F10 goal: final 1,786 / 3,712 / 1,306, minimum 959 / 1,366 / 536; T11.F04 `w1600` final 11,610 / 10,398 / 11,093 | No seed goes extinct; final, minimum, and plateau reported beside both references, no direction (the charge is about 5% of decay at this depth). |
| Goal `reachable_structure_size_distribution` | T03.F10 goal: mean 88.249706, median 74, p25 66, p75 106, max 367 | Reported; no direction. At generation 50 the charge on a 100-unit genome is 2% of decay, too small to read here; the long run below is the instrument for structure. |
| Goal `vm_steps`, `mesh_hops`, `graph_relax_iters`, `plasticity_updates`, `actions_applied`, `births` | T03.F10 goal and the pinned epoch | Reported under the usual thresholds (work flag +10%, severe +50%); no severe budgeted. Ecology moves, so ok/flag readings are read as ecology, not compute. |
| Gate counters and wall time | T03.F10 gate and the gate epoch | No severe; wall flags/severe at +25%/+100% on a matching host. No epoch re-pin budgeted. |
| Founder and evolved neighborhood rows; drift walk rows (changed/all 0.001500 / 0.008000 floors at 1,000 / 2,000; dead pooled 8 / 4,000; hop-cap hits 0; executed and total nodes) | T03.F10 goal | Founder neighborhood and every drift-walk row byte-identical to T03.F10 (neither runs Phase 0). Evolved rows are reported; they are confounded by the changed population as before. Floors not below (strict). |
| Long run under selection, paired | Depth note: 0.0595 nodes per generation by drift; founder 2 nodes, 96 units | Two `v3-cli run` arms at the default config (1600², 10,000 founders), seed 11, 12,000 ticks, `--sample-every 500`, each wrapped in `scripts/bench-wait`: the cost arm at the default rate and a control arm from a config file identical to the defaults except `genome_carry_cost_per_unit` 0.0. Read at the final sample: `mean_mesh_nodes` and `mean_genome_size` in the cost arm strictly below the control arm's; neither arm extinct; both arms' `mean_generation`, and the slope `(mean_mesh_nodes - 2) / mean_generation`, reported beside 0.0595. Budget: 40 minutes wall time for the pair (about 20 ticks per second at this world size); a pair that exceeds it is reported and stops there. |
| Observation budgets | Workflow caps | Founder neighborhood below 10 s; summed evolved below 180 s; drift walk below 30 s; whole goal run below 15 minutes. |

## Implementation Tasks

- [x] Add `genome_carry_cost_per_unit` to `EnergyLifecycleConfig` with a serde
      default, normalization, the config-spec row and note, and the frontend
      type, fixtures, panel row, and control-bar fixture; failing tests
      first. (`b514ca3e`)
- [x] Add `cached_genome_size` to `CreatureState` (computed in `new`, copied
      through `new_with_cached_fields` from the parent on the fast path) and
      charge it in `run_phase_0` as the one combined subtraction above; tests
      first, `cargo test -p v3-core --test viability` first after the charge
      lands. (`f6266bfc`, `1fa5ce19`)
- [x] Property tests for the pure charge: the per-tick charge is
      `decay + rate * size` within one ulp, monotone non-decreasing in size
      and in rate, equal to decay when rate is 0.0 or size is 0, and never
      negative. Example tests (plumbing, not an invariant): `cached_genome_size`
      equals a fresh `genome_size()` for every founder profile and for a
      mutated child. (`1fa5ce19`; five properties on `phase_0_energy_charge`
      in `crates/v3-core/src/simulation/tick/tests/phase0.rs`; no
      `proptest-regressions` entry appeared)
- [x] Add the three `tick_sample` fields to `v3-cli` with a test that reads
      them from a short run, then run the paired long run and record it.
      (`96edc818`; long run recorded under "Measured" below)
- [x] Update `v3-tick-orchestration-spec.md`, `docs/progress.md`, and
      `docs/progress/benchmark-series.json`; store the gate and goal reports.
      (`1fa5ce19`, `deff4ac6`, and the closure commit)

## Verification

- [x] `cargo test -p v3-core --test viability` first after the charge lands;
      `cargo check --workspace --all-targets` after coherent Rust edits;
      focused suites `cargo test -p v3-core --lib simulation`,
      `cargo test -p v3-core --lib config`, `cargo test -p v3-core --lib
      creature`, `cargo test -p v3-cli`, and `npm --prefix frontend test --
      --run` pass.
- [x] Unit tests: at rate 1.0 and decay 0.0 a creature loses exactly its
      genome size in energy per tick; at the default rate the founder loses
      `0.5 + 111e-4` per tick exactly (the spec's `96` is stale — see the
      correction below); at rate 0.0 Phase 0 is byte-identical to the
      pre-feature charge; a creature whose charge crosses zero is removed in
      the same tick; a newborn on the no-mutation fast path carries a cached
      size equal to a fresh `genome_size()`; the config round-trips,
      normalizes NaN and negative to the default, and defaults to `1e-4`.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after the simplify pass;
      record the summary line, output path, and every survivor's resolution.
- [x] `make bench PROFILE=gate FEATURE=t03-f08-genome-size-maintenance-cost`
      stores `docs/progress/features/t03-f08-genome-size-maintenance-cost.json`;
      one `make bench PROFILE=goal FEATURE=t03-f08-genome-size-maintenance-cost`
      stores the `-goal.json` report. Record every predeclared reading above,
      the observation budgets, and the compute comparisons.
- [x] Paired long run, once, as predeclared: the two `v3-cli run` command
      lines, the control config's diff from the defaults, the final-sample
      readings of both arms, and the wall time of each arm, recorded below.
- [x] Second goal run: Not applicable by the 2026-09-05 workflow decision;
      `crates/v3-core/tests/reproducibility.rs` covers cross-process
      reproducibility inside `make check`.
- [x] `make roadmap-check` on document edits; final `make check` exits 0 on
      the closure content, with the tested commit reported in the parent task.

### Founder size correction

The spec's sizing paragraph and its `0.5 + 96e-4` unit-test target are stale:
`founder_genome(FounderProfile::V3Alpha1).genome_size()` is **111**, not 96, at
this revision (complexity 63, two mesh nodes), pinned by
`the_canonical_founder_genome_is_one_hundred_eleven_units` in
`crates/v3-core/src/creature/state.rs` and read independently by the CLI test
`tick_sample_reports_the_founder_structure_means_before_any_birth`. Nothing in
the Fixed design table changes: the rate stays `1e-4`, decided before
implementation and not adjusted after measurement. The consequence is that the
founder pays `0.0111` per tick, **2.22%** of the `0.5` decay rather than the
1.9% the sizing paragraph predicted, so the first clause of the second success
criterion ("the founder pays under 2% of decay") is not met as written. This
is recorded, not waived; the orchestrator owns the decision.

### Results, 2026-09-07

Commands run in the worktree, in this order, and their results:

- `cargo test -p v3-core --lib config` — ok, 134 passed, 0 failed (config
  field, red first: `no field genome_carry_cost_per_unit` before the field
  landed).
- `npm --prefix frontend run lint` — 201 files checked, 3 pre-existing
  `noArrayIndexKey` warnings in `FoodTypesSection.tsx` and
  `FertilitySection.tsx`, files this feature does not touch; no error.
- `npm --prefix frontend test -- --run` — 58 files, 308 tests passed.
- `cargo test -p v3-core --lib creature` — ok, 163 passed, 0 failed (red first
  at `left: 111, right: 96`, which is the founder-size correction above).
- `cargo test -p v3-core --test viability` — ok, 24 passed, 0 failed, run
  first after the charge landed.
- `cargo test -p v3-core` (whole package, not only `--lib`) — first run
  FAILED with 3 failures, which is the intended red plus two fixture
  interactions; green after the fixes below (1,244 passed, 0 failed, 1
  ignored, plus every integration target).
- `cargo test -p v3-core --lib simulation::actions::reproduction` — ok, 6
  passed, 0 failed.
- `cargo test -p v3-core --lib simulation::tick::tests::phase0` — ok, 23
  passed, 0 failed (18 example tests and 5 properties).
- `cargo test -p v3-cli` — ok, 39 + 9 + 17 + 9 passed, 0 failed.
- `cargo check --workspace --all-targets` — clean after every coherent Rust
  edit, through the compile hook.
- `cargo clippy --workspace --all-targets` — clean, no warnings.
- `cargo fmt --all` — applied.
- `make roadmap-check` — `roadmap-check: validation passed`, exit 0.
- `make check` — exit 0 before the measurements; rerun on the closure content
  with the tested commit reported in the parent task. The same three
  pre-existing frontend lint warnings appear and do not fail the step.
- `make bench PROFILE=gate FEATURE=t03-f08-genome-size-maintenance-cost` —
  exit 0, report stored, `severe=false` against both references.
- `make bench PROFILE=goal FEATURE=t03-f08-genome-size-maintenance-cost` —
  exit 0, report stored, `severe=false`. Run once, per the 2026-09-05
  decision.

Two pre-existing tests changed, both because the new charge is a world-level
Phase 0 cost their fixtures deliberately zero out:

- `failed_action_penalty_ramp_uses_effective_tick_value`
  (`simulation/tick/tests/actions.rs`) already sets `energy_decay_per_tick`,
  `move_cost`, and both action multipliers to zero so the tick's energy
  difference is the failed-action penalty alone. It now sets
  `genome_carry_cost_per_unit = 0.0` beside them. No assertion was weakened.
- `crates/v3-core/tests/common/mod.rs`'s shared `test_config` does the same
  for the temporal fixtures, which read a creature's energy difference across
  a tick as the production reward signal. Only `e3_skipped_module_visits`
  actually failed (weight `0.97499084` against an expected `0.9738922`, a
  0.0022 discrepancy that is exactly the fixture genome's 22 units at the
  default rate); the fix zeroes the new charge in the same helper that
  already zeroes decay, with a comment. No assertion was weakened, and the
  discrepancy confirms that the reward signal is taken from the post-Phase-0
  snapshot and so excludes this charge.

One pre-existing test was removed as strictly subsumed rather than weakened:
`phase_0_decays_energy` asserted `initial - decay` within `f32::EPSILON` on the
founder fixture. `phase_0_charges_the_founder_its_carrying_cost_beside_decay`
uses the same fixture and initial energy and asserts the exact combined charge
with `assert_eq!`, and `phase_0_at_rate_zero_reproduces_the_pre_feature_energy_bit_for_bit`
asserts the decay-only landing bit for bit through `to_bits()`.

Unit tests, in `crates/v3-core/src/simulation/tick/tests/phase0.rs`: the pure
charge is `0.5 + 111e-4` at the production defaults, the genome size itself at
rate 1.0 with no decay, and the decay alone at rate 0.0 or size 0; the founder
loses exactly `decay + 111 * 1e-4` through `run_phase_0` and strictly more than
decay alone; the charge follows the cached size, not the genome; at rate 0.0
the landing energy is bit-identical to `initial - decay`; a creature the
carrying charge takes below zero is removed from the slotmap and from world
occupancy in the same tick while one it leaves above zero survives with the
exact remainder; and `lifetime_energy_sum` samples the post-charge energy. In
`crates/v3-core/src/creature/state.rs`: `a_new_creature_caches_its_own_genome_size`
covers the minimal genome and all five founder profiles;
`new_with_cached_fields_preserves_provided_reachable` pins the copied size. In
`crates/v3-core/src/simulation/actions/reproduction.rs`: 40 ticks of a seeded
200-founder world at `mutation_probability` 1.0 and 0.0 assert that every
living creature's `cached_genome_size` equals a fresh `genome_size()`, covering
the mutated and fast paths. In `crates/v3-cli`: the empty population reads
`(0.0, 0.0, 0.0)`, a four-founder world reads the founders' own mean with 2.0
mesh nodes and generation 0.0, an NDJSON `tick_sample` at tick 1 reads
`111.0 / 2.0 / 0.0`, and a 60-tick run reads a mean generation above zero.

Property tests for the pure charge (`phase_0_energy_charge`), five properties
over decay `0..10`, rate `0..1`, and size `0..20,000`: the charge equals
`decay + rate * size` exactly; it is monotone non-decreasing in size and in
rate; it is bit-identical to the decay when the rate is 0.0 or the size is 0;
and it is never negative. No assertion depends on which cases were drawn, and
no `proptest-regressions` file appeared for this module.

Simplification pass (`simplify` skill; single-pass inline review of the diff
against merge base `8d43b0e1`, not the four-agent fan-out, which was
unavailable). Three fixes applied in `fdf2ecef`: `phase_0_energy_charge` had
been inserted between `run_phase_0`'s doc comment and its signature, silently
stealing the doc block — the helper moved above it and `run_phase_0`'s
documentation is restored; `structure_means` in `v3-cli` replaced a
three-tuple `fold` with a shared `mean` closure over three `sum()` passes,
which reads plainly and costs nothing at 24 sample ticks; and the two
duplicated `CreatureState::new` cached-size tests in `state.rs` merged into one
loop over the minimal genome plus all five founder profiles. The config,
normalization, frontend, and serde work already used the repository's
declarative helpers (`normalize_f32_finite_nonneg`, `#[serde(default = ...)]`,
`FieldDef` rows), so nothing else changed.

Mutation testing, fresh (`MUTANTS_ITERATE=0 make rust-mutants`, run after the
simplify pass and after the measurements, diff against merge base
`8d43b0e1`):

- Summary line: `49 mutants tested in 4m: 46 caught, 3 unviable`.
- Output path: `/Users/istefanek/.local/share/petri-tools/mutants/t03-f08/mutants.out`,
  with `run-mode.txt` recording `fresh`.
- The target printed `rust-mutants: no survivors`. `missed.txt` and
  `timeout.txt` are both empty: **no mutant was missed by every test and none
  timed out**, so there is no survivor to resolve as killed, equivalent, or
  deferred, and the target was run once.
- No `#[mutants::skip]` attribute and no `exclude_re` entry was added anywhere
  in this feature.
- The three unviable mutants are `Default::default()` substitutions for
  `build_tick_sample -> TickSampleEvent`,
  `CreatureState::new_with_cached_fields -> Self`, and
  `apply_reproduce -> ReproductionActionResult`; none of those types implements
  `Default`, so the mutants do not compile and are not survivors.
- Every mutant generated on this feature's own new code was caught: all 27
  constant-tuple replacements of `structure_means`, its `population == 0` guard
  (`== ` to `!=`), and both arithmetic mutations of its `/`; all three constant
  replacements of `phase_0_energy_charge` and all four of its arithmetic
  mutations (`+` to `-`, `+` to `*`, `*` to `+`, `*` to `/`).

## Performance and Goal Impact

Natural analog: maintenance metabolism of tissue kept alive whether or not it
fires, and the deletional bias that removes unexpressed sequence once carrying
it costs anything. It reaches creatures through the body: the charge is
energy, settled in Phase 0 beside decay, and selection sees it only as a
slightly higher maintenance rate and earlier starvation for larger genomes.
No sensor, reward, or authored script.

Predeclared cost: one multiply-add per creature-tick in Phase 0 and one
`u32` computed once per birth on the mutated path (the fast path copies it).
Wall time per creature-tick is expected flat. No severe compute allowance and
no epoch re-pin are budgeted for either profile. The `deterministic` blocks
of both profiles differ from every prior report for every seed, because every
creature's energy now moves by the charge; the founder neighborhood and
drift-walk blocks are predeclared identical to T03.F10's.

The paired long run is the feature's structural reading: the goal profile at
generation 50 cannot see a 2% charge, and the drift walk has no energy. Its
predeclared direction is that carried structure is smaller under the cost
than without it at the same tick, with no extinction. A reading that is not
below is investigated before closure, since it would mean the rate is under
the selection threshold at this population size; the rate is not raised
without a recorded reason, and never to make cognition expensive.

### Measured, 2026-09-07

Reports, both stored on this branch and produced with production code unchanged
since `fdf2ecef`: gate
`docs/progress/features/t03-f08-genome-size-maintenance-cost.json`
(`make bench PROFILE=gate` exit 0, `severe=false`) and goal
`docs/progress/features/t03-f08-genome-size-maintenance-cost-goal.json`
(`make bench PROFILE=goal` exit 0, `severe=false`). Both carry `git_revision`
`fdf2ecef166f43d187ea241bd616efb07ebc983a`. T03.F10 is the previous closure;
T11.F17 remains the pinned goal epoch and `remove-complementary-nutrition` the
gate epoch, and neither is re-pinned.

Predeclared readings:

| Reading | Reference | Predeclaration | Measured | Result |
| --- | --- | --- | --- | --- |
| Goal persistence, seeds 11/22/33 | T03.F10 final 1,786/3,712/1,306, minimum 959/1,366/536; T11.F04 `w1600` final 11,610/10,398/11,093 | No seed extinct; final, minimum, plateau reported, no direction | No extinction on any seed. Final 2,153 / 1,091 / 986; minimum 1,551 / 791 / 986; plateau 1,871.002000 / 955.606000 / 1,250.428000 (T03.F10 plateau 1,449.400000 / 2,830.522000 / 900.308000) | Met |
| Goal `reachable_structure_size_distribution` | T03.F10 mean 88.249706, median 74, p25 66, p75 106, max 367 | Reported; no direction | mean 88.999527, median 71, p25 65, p75 111, max 336 | Reported |
| Goal `vm_steps`, `mesh_hops`, `graph_relax_iters`, `plasticity_updates`, `actions_applied`, `births` | T03.F10 goal and the pinned epoch | Work flag +10%, severe +50%; no severe budgeted | Against T03.F10 / the pinned epoch: `vm_steps` 22.556467 (-0.043897% / -84.569874%); `mesh_hops` 2.074751 (+0.104362% / +0.170528%); `graph_relax_iters` 1.000711 (+0.171171% / -0.191198%); `plasticity_updates` 0.032645 (-2.622002% ok / +11.618286% flag); `actions_applied` 1.237238 (+0.196386% / -11.684335%); `births` 0.016080 (+3.096749% / -2.290818%) | Met; `severe=false` |
| Gate counters and wall time | T03.F10 gate and the gate epoch | No severe; wall flags/severe at +25%/+100% on a matching host; no epoch re-pin | All six ok against both references (`vm_steps` 22.443992, -0.224459% vs T03.F10 and -1.350797% vs the epoch; `plasticity_updates` 0.007563, +2.814029% / -32.297914%). The harness recorded `wall_clock: null` for every reference in both profiles because the host is not matching: this machine now reports hostname `MacBookPro.lan` where every reference was taken on `Isaacs-MacBook-Pro-2.local` (same `Apple M1 Pro`, `aarch64`, 8 logical cores). Raw wall/creature-tick 0.0013887669 gate (+5.49% against T03.F10's 0.0013164589, -10.97% against the epoch's 0.0015598310) and 0.0057962037 goal (+0.63% against T03.F10's 0.0057599713), both inside the +25% flag had the comparison been made | Met; no epoch re-pin |
| Founder and evolved neighborhood rows; drift walk rows | T03.F10 goal | Founder neighborhood and every drift-walk row byte-identical; evolved reported and confounded; floors not below (strict) | The whole `mutational_neighborhood.founder` block, its `battery` block, and the entire `drift_depth` block compare **equal** to T03.F10's, field for field. Drift changed/all births 0.001500 at depth 1,000 and 0.008000 at 2,000 (both floors met exactly); dead pooled 8 / 4,000 (4 and 4); hop-cap hits 0 at all five depths; mean executed nodes 2.000000 / 2.260000 / 2.880000 / 4.280000 / 4.860000; total nodes at 2,000 143.760000. Evolved pooled 1,059 / 3,300 changed (0.320909) and 66 / 3,300 dead (0.020000), against T03.F10's 967 / 3,300 (0.293030) and 57 / 3,300 (0.017273) | Met; evolved reported as the predeclared confound |
| Long run under selection, paired | Depth note: 0.0595 nodes per generation by drift | Cost arm's `mean_mesh_nodes` and `mean_genome_size` strictly below the control's at the final sample; neither extinct; generations and node slopes reported; pair under 40 minutes | Cost arm 386.535913 units in 10.126750 nodes; control 1,093.835901 in 18.402314. Both strictly below (-64.66% and -44.97%). Neither arm extinct at any sample. Full table below | Met |
| Observation budgets | Workflow caps | Founder below 10 s; summed evolved below 180 s; drift below 30 s; goal run below 15 min | Founder neighborhood 0.060 s (gate 0.051 s); summed evolved 0.426 s; drift walk 3.333 s; whole goal run 294.85 s | Met |

#### Paired long run, seed 11, 12,000 ticks

Run once. The two arms, each wrapped in `scripts/bench-wait` and invoking the
already-built release binary directly so the wall times exclude compilation:

```
scripts/bench-wait target/release/v3-cli run \
  --ticks 12000 --sample-every 500 --seed 11 --config <cost-config.json>
scripts/bench-wait target/release/v3-cli run \
  --ticks 12000 --sample-every 500 --seed 11 --config <control-config.json>
```

Both config files came from one source: `SimulationConfig::default()`
serialized with `serde_json::to_string_pretty` by a temporary `println!` added
to the existing `config_serde_roundtrip_preserves_defaults` test, captured with
`cargo test -p v3-core --lib config_serde_roundtrip_preserves_defaults --
--nocapture`, with the marker reverted by `git checkout` before the runs (the
worktree was clean at `fdf2ecef` when both arms started). Their entire diff is
one line:

```
     "lifecycle": {
       "default_offspring_energy": 100.0,
       "energy_decay_per_tick": 0.5,
-      "genome_carry_cost_per_unit": 0.0001,
+      "genome_carry_cost_per_unit": 0.0,
```

The round trip was verified before spending the budget: 20 ticks at seed 11
with the cost config file and with no `--config` produce semantically identical
NDJSON. The only textual difference is `HashMap` key ordering inside the
mutation-operator maps; every event, counter, and float is equal.

Final sample, tick 12,000:

| Reading | Cost arm (rate `1e-4`) | Control arm (rate `0.0`) | Cost against control |
| --- | --- | --- | --- |
| `mean_genome_size` | 386.535913 | 1,093.835901 | **-64.66%** |
| `mean_mesh_nodes` | 10.126750 | 18.402314 | **-44.97%** |
| `mean_generation` | 177.345373 | 301.349388 | -41.15% |
| `population` | 12,071 | 19,537 | -38.22% |
| `mean_energy` | 31.412428 | 20.201092 | +55.50% |
| node slope `(nodes - 2) / generation` | 0.045824 | 0.054430 | drift reference 0.0595 |
| genome units per generation `(size - 111) / generation` | 1.553668 | 3.261450 | -52.36% |
| wall time | 754 s | 1,575 s | pair 2,329 s = 38.8 min, inside the 40-minute budget |

Neither arm went extinct at any of its 24 samples; the cost arm's minimum
sampled population is 1,674 at tick 1,000 and the control's is 1,047 at tick
1,500.

The predeclared direction holds at the final sample on both structural
readings, and it survives the obvious confound. The control arm is 124
generations deeper at the same tick, so the same-tick comparison flatters the
cost arm. Read per generation instead, the cost arm accumulates 1.55 genome
units per generation against the control's 3.26, and its node slope 0.045824 is
below both the control's 0.054430 and the drift note's 0.0595. Read at the
control's own interpolated generation 177.3 (between its ticks 6,000 and
6,500), the control carries 525.6 genome units in 8.97 mesh nodes against the
cost arm's 386.5 units in 10.13 nodes: at matched depth the cost arm carries
26% fewer units inside *more* nodes. That is the expected shape of a charge
that prices units rather than nodes — selection sheds intra-node bulk (VM
instructions and constants, compute nodes and edges) first, and the mesh keeps
growing. The same-tick reading the spec predeclared is met; the
matched-generation reading is recorded so the result is not read as a stronger
claim than the evidence supports. No investigation is owed, since no reading is
"not below".

The cost arm's shallower lineages and higher mean energy at the same tick are
the charge working through the body as designed: creatures carrying more
structure starve earlier, births are slower, and the survivors hold more
energy. No cognition claim.

## Success Criteria

- [x] Every creature pays `genome_carry_cost_per_unit * genome_size()` energy
      per tick in Phase 0, through the existing energy accounting and death
      rule, with the rate at 0.0 reproducing the pre-feature trajectories.
- [x] The founder pays about 2% of decay at the default rate (amended
      2026-09-07 by the orchestrator from "under 2%": that figure was
      arithmetic on a stale founder size of 96 units; the design intent,
      a founder budget barely touched, is what the criterion states, and the
      rate is unchanged), and the paired long run reads less carried
      structure under the cost than without it.
      **Both clauses met under the amended wording; the original first clause was not met as written.** The paired long
      run reads -64.66% genome size and -44.97% mesh nodes under the cost. The
      founder pays 2.22% of decay, not under 2%, because its genome is 111
      units and not the 96 the spec's sizing paragraph assumed (see the
      "Founder size correction"). The rate is left at the `1e-4` the Fixed
      design table decided before implementation; lowering it to meet a
      criterion derived from a stale founder size would be tuning after
      measurement. The orchestrator owns whether to amend the criterion or the
      rate.
- [x] Gate and goal reports stored with no severe regression; no seed
      extinct; neighborhood and drift rows identical to T03.F10's.

## Notes for AI Agents

- Founder size: the canonical V3Alpha1 founder is 111 `genome_size()` units
  (complexity 63) at this revision. The roadmap's T03 note and the depth
  research note say 96, a reading from an earlier revision; do not size
  anything from it. The "under 2%" success clause was amended to "about 2%"
  for that reason on 2026-09-07, with the rate left at `1e-4`.
- The genome is immutable after birth; do not recompute `genome_size()` in
  the tick loop. Charge the cached value.
- Do not touch `energy.complexity_cost` or `mutation.genome_size_pressure_enabled`
  beyond leaving them as they are; T11.F13 reads them against this cost.
- The control arm's config file must be the serialized default configuration
  with only the rate changed; record how it was produced and its diff.
- Wrap both long-run arms in `scripts/bench-wait` and do not start competing
  builds, tests, or servers while they run.
