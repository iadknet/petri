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
| Rate | `energy.lifecycle.genome_carry_cost_per_unit`, `f32`, default `1e-4` energy per unit per tick. Finite and non-negative; invalid values fall back to `1e-4`; `0.0` disables the charge. Sizing from the evidence: the founder's 96 units pay 0.0096 per tick, 1.9% of the 0.5 decay; a live median executed core (`complexity()` 532) would pay 0.053 (10.6% of decay); the deep run's median genome (6,246) would pay 0.625, 1.25 times decay, so a creature carrying that much junk pays 2.25 times the founder's maintenance; a typical junk mesh node (about 55 units) pays 0.0055 per tick, about 1% of decay, which at goal populations of 1,000 to 4,000 is a selection coefficient the population can see. At the goal profile's depth (about 3 units of growth per generation, so about 250 units by generation 50) the charge is about 5% of decay, so persistence is not expected to move much. |
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

- [ ] Add `genome_carry_cost_per_unit` to `EnergyLifecycleConfig` with a serde
      default, normalization, the config-spec row and note, and the frontend
      type, fixtures, panel row, and control-bar fixture; failing tests
      first.
- [ ] Add `cached_genome_size` to `CreatureState` (computed in `new`, copied
      through `new_with_cached_fields` from the parent on the fast path) and
      charge it in `run_phase_0` as the one combined subtraction above; tests
      first, `cargo test -p v3-core --test viability` first after the charge
      lands.
- [ ] Property tests for the pure charge: the per-tick charge is
      `decay + rate * size` within one ulp, monotone non-decreasing in size
      and in rate, equal to decay when rate is 0.0 or size is 0, and never
      negative. Example tests (plumbing, not an invariant): `cached_genome_size`
      equals a fresh `genome_size()` for every founder profile and for a
      mutated child.
- [ ] Add the three `tick_sample` fields to `v3-cli` with a test that reads
      them from a short run, then run the paired long run and record it.
- [ ] Update `v3-tick-orchestration-spec.md`, `docs/progress.md`, and
      `docs/progress/benchmark-series.json`; store the gate and goal reports.

## Verification

- [ ] `cargo test -p v3-core --test viability` first after the charge lands;
      `cargo check --workspace --all-targets` after coherent Rust edits;
      focused suites `cargo test -p v3-core --lib simulation`,
      `cargo test -p v3-core --lib config`, `cargo test -p v3-core --lib
      creature`, `cargo test -p v3-cli`, and `npm --prefix frontend test --
      --run` pass.
- [ ] Unit tests: at rate 1.0 and decay 0.0 a creature loses exactly its
      genome size in energy per tick; at the default rate the founder loses
      `0.5 + 96e-4` per tick within one ulp; at rate 0.0 Phase 0 is
      byte-identical to the pre-feature charge; a creature whose charge
      crosses zero is removed in the same tick; a newborn on the no-mutation
      fast path carries a cached size equal to a fresh `genome_size()`;
      the config round-trips, normalizes NaN and negative to the default,
      and defaults to `1e-4`.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after the simplify pass;
      record the summary line, output path, and every survivor's resolution.
- [ ] `make bench PROFILE=gate FEATURE=t03-f08-genome-size-maintenance-cost`
      stores `docs/progress/features/t03-f08-genome-size-maintenance-cost.json`;
      one `make bench PROFILE=goal FEATURE=t03-f08-genome-size-maintenance-cost`
      stores the `-goal.json` report. Record every predeclared reading above,
      the observation budgets, and the compute comparisons.
- [ ] Paired long run, once, as predeclared: the two `v3-cli run` command
      lines, the control config's diff from the defaults, the final-sample
      readings of both arms, and the wall time of each arm, recorded below.
- [ ] Second goal run: Not applicable by the 2026-09-05 workflow decision;
      `crates/v3-core/tests/reproducibility.rs` covers cross-process
      reproducibility inside `make check`.
- [ ] `make roadmap-check` on document edits; final `make check` exits 0 on
      the closure content, with the tested commit reported in the parent task.

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

## Success Criteria

- [ ] Every creature pays `genome_carry_cost_per_unit * genome_size()` energy
      per tick in Phase 0, through the existing energy accounting and death
      rule, with the rate at 0.0 reproducing the pre-feature trajectories.
- [ ] The founder pays under 2% of decay at the default rate, and the paired
      long run reads less carried structure under the cost than without it.
- [ ] Gate and goal reports stored with no severe regression; no seed
      extinct; neighborhood and drift rows identical to T03.F10's.

## Notes for AI Agents

- The genome is immutable after birth; do not recompute `genome_size()` in
  the tick loop. Charge the cached value.
- Do not touch `energy.complexity_cost` or `mutation.genome_size_pressure_enabled`
  beyond leaving them as they are; T11.F13 reads them against this cost.
- The control arm's config file must be the serialized default configuration
  with only the rate changed; record how it was produced and its diff.
- Wrap both long-run arms in `scripts/bench-wait` and do not start competing
  builds, tests, or servers while they run.
