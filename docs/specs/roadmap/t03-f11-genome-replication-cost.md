# T03.F11 — Genome Replication Cost

**Status**: In Progress
**Last updated**: 2026-09-15
**Feature**: T03.F11
**Track**: [T03 — Functional Traits and Metabolism](../../roadmaps/t03-functional-traits-and-metabolism.md)

## Goal

Copying a longer genome costs more. The parent's reproduce charge at Step 5
of `apply_reproduce` is multiplied by `1 + rate × max(genome_size() − 111, 0)`,
where 111 is the canonical V3Alpha1 founder's `genome_size()`, so the founder
pays exactly what it pays today and every unit of structure above the
founder's raises what a parent burns at each birth. The extra energy is burned
through the existing `action_charges.reproduce` flow; none of it reaches the
offspring. Natural analog: replication time and cost scale with genome length,
which is Avida's and Tierra's original size brake, since a longer genome takes
proportionally more CPU cycles to copy (the
[per-unit mutation supply research note](../../strategy/per-unit-mutation-supply-research-2026-09-14.md),
Section 4). It reaches creatures through the body only: energy at
reproduction, no sensor, no score, no cap. With T03.F08 charging carried
structure per tick and T03.F10 charging activity, this is the third cost on
structure and the one selection counts directly, at the birth.

## Non-Goals

- No cap, pruning rule, or new mutation operator; no change to the mutation
  engine, the per-unit supply rate, or the drift walk (T11.F20 is separate).
- `energy.complexity_cost` is not edited, enabled, or reused; it already sits
  in the same multiplier chain through `cached_complexity` and excludes junk.
- `reproduce_cost` (0.1), `min_reproduce_energy` (30), the offspring transfer
  (Step 7), the order of the reproduction steps, and T03.F08's carrying rate
  (1e-4) are unchanged.
- The rate is not sized to make cognition expensive and is not raised without
  a recorded reason; a pair that shows no plateau is reported, not tuned.

## Inputs and Invariants

Sources of truth: the track's T03.F11 row and note (charge site, formula,
founder anchor, cached size, `complexity_cost` untouched, the fraction
predeclared for a 386-unit genome, the verification list); the T03.F08 spec
(unit, the 1e-4 carrying rate, the paired-run pattern and its
`docs/progress/sweeps/t03-f08/` reading of 386.5 units at generation 177.3);
the T11.F19 spec and readings (previous closure and pinned goal-worlds epoch);
the T14.F12 spec (the neighborhood read and its standing floors); the T11
track's 2026-09-14 floor amendment (the drift walk is not this feature's
instrument); `crates/v3-core/src/simulation/actions/reproduction.rs` (Steps
5–8); `crates/v3-core/src/config/simulation.rs` (`EnergyLifecycleConfig`,
`adjusted_action_cost`, `normalize`); `crates/v3-core/src/creature/state.rs`
(`cached_genome_size`, the founder assertion
`the_canonical_founder_genome_is_one_hundred_eleven_units`);
`experiments/worlds/orchards-in-grassland.json`; `v3-cli run --config`
(`load_config` resolves a partial recipe over `SimulationConfig::default()`).

What the code does today, verified: Step 5 subtracts
`adjusted_action_cost(reproduce_cost, cached_complexity, age)` from the parent
and records it through `observe_energy` into `action_charges.reproduce`; Step
6 then rejects a parent below `min_reproduce_energy` and Step 7 rejects a
transfer the parent cannot cover, so an attempt that passes Steps 1–4 pays the
charge whether or not a child is born. The age multiplier (up to 10× at age
500) already scales the charge; `complexity_cost` is disabled. The founder
reproduces only above an energy threshold, so its attempts are conditional;
evolved populations reject most attempts (the T11.F19 Orchards probe rejected
83% at tick 6,000), and the stored samples do not split rejections by reason.

Evidence that sizes the rate: T03.F08's cost arm carried 386.5 units in 10.13
nodes at tick 12,000 (generation 177.3, 67.7 ticks per generation) against the
control's 1,093.8, and neither arm levelled off; the per-unit supply's Orchards
arm carried 443.6 units in 14.6 nodes at tick 6,000 (generation 137) with no
plateau (note, Section 5.3); the T11.F19 goal worlds ended at 340.9 / 300.4 /
309.0 units in 6.91 / 6.26 / 7.06 nodes (Orchards / Canyon / Confluence,
generation 45–50), with the neighborhood read's executed nodes at 3.10 / 3.34
/ 3.02 against the founder's 2.

Invariants: energy is conserved and behavior-backed (the charge is a
subtraction from the parent and an addition to `action_charges.reproduce`,
nothing else); the genome is immutable after birth, so `cached_genome_size` is
always current and no path walks a genome; the multiplier is exactly 1.0 for
every genome of 111 units or fewer and for every genome when the rate is 0.0,
so the founder's energy trajectory and a rate-0.0 world are byte-identical to
the pre-feature engine; the multiplier is non-decreasing in genome size.

Fixed design, decided before implementation:

| Decision | Value |
| --- | --- |
| Formula | `charge = adjusted_action_cost(reproduce_cost, cached_complexity, age) × (1 + rate × max(cached_genome_size − 111, 0))`, one `f32` product computed at the Step 5 site in `reproduction.rs` (or a helper it calls) from the rate and the founder constant; `config` does not import `creature`. The factor composes multiplicatively with the existing complexity and age multipliers because it is a scale on the same act; `complexity_cost` is not touched. |
| Rate | `energy.lifecycle.genome_replication_cost_per_unit`, `f32`, default `0.1` per `genome_size()` unit above the founder, `#[serde(default)]`; finite and non-negative, invalid values fall back to `0.1`; `0.0` disables the charge. |
| Founder anchor | `pub const FOUNDER_GENOME_SIZE_UNITS: u32 = 111` beside `founder_genome` in `creature/founder.rs`, read by the multiplier; the existing state test `the_canonical_founder_genome_is_one_hundred_eleven_units` asserts the founder's `genome_size()` equals the constant, so the 111 is never a bare literal in a second place. Not a config field: the allowance is a property of the founder, and a knob would let it drift from the anchor that keeps the founder neutral. |
| Unit | `genome_size()` units, for T03.F08's reason: junk lives inside nodes as well as between them, and per-node pricing would reward packing. Same rate for every unit above the anchor; a discount for executed structure would need the reachability analysis whose blind spot on junk is why `complexity_cost` is not this lever. |
| Charge on rejected attempts | Kept as today: Step 5 charges before the Step 6 and 7 energy gates, so an attempt those gates reject burns the multiplied charge. This is the track's stated order (a larger charge also tightens the gate), and it is the analog: the copy loop consumes cycles whether or not the divide succeeds. The founder gates its own attempts on energy, so it is unaffected. |
| Cache | Reads `cached_genome_size` from `CreatureState`, the field Step 13 already hands to the child. No new cached field. |
| Frontend | `LifecycleEnergyConfig` gains the field; `EnergyLifecycleSection.tsx` gains one row beside "Carry Cost / Unit" (label "Replication Cost / Unit", min 0, max 1, step 0.001, default 0.1, tooltip naming the per-birth multiplier above the founder's 111 units); `test/fixtures.ts` and the `ControlBar.test.tsx` fixture carry it. |
| Reference docs | `v3-runtime-config-spec.md` Section 4 gains the row, a "Genome replication cost" note (formula, anchor, site, composition with the age and complexity multipliers, charged before the energy gates), and the reproduction-transfer sequencing step 2 names the factor; `v3-reproduction-spec.md` step 5 names it. |
| Determinism | Every charged attempt by a parent above 111 units moves that parent's energy, so the gate and goal `deterministic` blocks differ from every prior report; the founder neighborhood, drift-walk, and `recruitment_paths` blocks run no reproduction action and are predeclared identical to T11.F19's. The new field enters every `config_digest`, so the goal recipes' digests in `crates/v3-cli/tests/bench_artifacts.rs` are re-pinned and the comparator reports `inputs_changed`, comparable. The short-run identity hash in `crates/v3-core/tests/baseline_worlds.rs` moves only if that run contains a charged attempt by a parent above 111 units; if it moves it is re-pinned once after two runs agree, the T11.F19 procedure. The applied-trajectory digest in `crates/v3-core/tests/applied_trajectory.rs` forces 2 to 4 mutation events per birth over 64 ticks, so mutated parents above the anchor reproduce inside it and pay the charge: its digest moves for the same reason and is re-pinned once by the same two-run procedure, with the founder bit-identity check at rate 0.1 against 0.0 as the evidence that the founder is untouched. |

Rate sizing, recorded before any run. The reference genome is the T03.F08
cost arm's 386 units at generation 177: 275 units above the founder, factor
28.5, so **it pays 27.5 times more per charged attempt than the founder does**
(2.85 energy against the founder's 0.1 at the base charge, both scaled alike
by age). Against the 100-energy offspring transfer that is a 2.75% surcharge
per birth. The carrying cost charges the same 275 units 1.86 energy per
generation at that arm's cadence (275 × 1e-4 × 67.7 ticks), so the birth
charge alone is 1.5 times the carrying brake on the excess, and the two
together charge it about 4.7 energy per generation, 2.5 times what T03.F08
alone charged; a typical junk mesh node of about 55 units costs 0.55 energy
per charged attempt. At the goal worlds' T11.F19 terminal sizes the factor is
20 to 24 (2.0 to 2.4 energy per charged attempt), and at the per-unit probe's
443-unit Orchards genome it is 34 (3.4 energy). A working core that grows to
150 units pays 0.04 extra, so function is not taxed at this rate. The pure
Avida proportion (factor `genome_size / 111`, rate 1/111 ≈ 0.009) was
considered and rejected: on a 0.1 base charge it costs the reference genome
0.25 energy per birth, an eighth of the carrying brake, which is not the
stronger brake the track asks for. A rate of 0.2 was considered and not
adopted: it would charge the goal worlds' terminal genomes 4 to 4.8 energy
per attempt, twice this rate's, and every attempt the energy gates reject
burns it too, before the pair has shown what 0.1 does. Research summary: the
mechanism is mandated by the track; the options were the rate value and the
anchor's placement, decided above from the stored T03.F08 and T11.F19
readings; no external source was fetched, and the Avida/Tierra precedent is
cited from the note's Section 4, not re-read here.

## Implementation Tasks

- [x] Add `FOUNDER_GENOME_SIZE_UNITS` and extend the founder-size test to pin it.
- [x] Add `genome_replication_cost_per_unit` to `EnergyLifecycleConfig` with
      default, serde default, and normalization; add the multiplier and apply it
      at Step 5, with unit tests (founder 1.0, 386 units 28.5, rate 0.0, sizes
      below the anchor) and proptest invariants (factor ≥ 1, non-decreasing in
      size, exactly 1.0 at or below the anchor).
- [x] Frontend field, row, fixtures; reference docs; re-pin the recipe digests
      and, if moved, the short-run identity hash (it did not move; the
      `applied_trajectory` digest did and is re-pinned).
- [x] Run the paired Orchards run and store its artifacts under
      `docs/progress/sweeps/t03-f11/`.

## Verification

- [ ] `cargo test -p v3-core --test viability` first, then `make check` ->
      results in [`docs/progress/readings/t03-f11.md`](../../progress/readings/t03-f11.md).
- [x] Focused tests: the founder pin, the multiplier unit and property tests,
      a reproduction test showing a parent above the anchor is charged the
      factor and the child receives the unchanged transfer, and a rate-0.0
      identity check -> test names and results in the readings file.
- [x] Persistence re-read: goal persistence per world beside T11.F19's finals
      (13,808 / 10,546 / 13,789) and plateaus and the T11.F04 `w1600` sweep
      (11,610 / 10,398 / 11,093) -> readings file. No world extinct.
- [x] Neighborhood-read guard: `neighborhood_read.changed_per_all_births` per
      world against the standing floors and `mean_executed_nodes` against the
      threshold in the table below; the goal
      `reachable_structure_size_distribution` and terminal `mean_genome_size` /
      `mean_mesh_nodes` per world beside T11.F19's -> readings file. All pass.
- [x] Paired 12,000-tick Orchards run, seed 11, cost against control, as
      specified below -> artifacts under `docs/progress/sweeps/t03-f11/`, final
      and per-sample table in the readings file. Per the user decision below,
      the control arm is stopped at tick 4,500 and the comparison is read
      there; the cost arm is complete to 12,000 and answers the levelling-off
      question alone; `timing.txt` states the truncation. Cost strictly below
      control at the common tick; neither arm extinct; configs differ only in
      the one field. Levelling-off ratio 2.29 (increments 4,000–8,000 = 4.02
      units, 8,000–12,000 = 9.21 units) — **not** levelling off; the ratio
      is a quotient of increments of 0.85% and 1.9% of the base while
      `mean_genome_size` sits in a 462–506 band from tick 4,500, and
      `mean_mesh_nodes` rises 10.44 → 25.29 with no plateau (one 0.08-node
      dip at tick 7,500; units per node falling ~45 → ~19). Full tables in
      the readings file.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` at 9eee4583, run mode
      `fresh`; `missed.txt` and `timeout.txt` empty; survivor list: none.

  ```text
  14 mutants tested in 5m: 13 caught, 1 unviable
  rust-mutants: no survivors
  output: ~/.local/share/petri-tools/mutants/t03-f11/mutants.out
  ```
- [x] Benchmark summaries stored at `docs/progress/features/t03-f11-genome-replication-cost.json`
      and `-goal.json`, local raw hash/byte count and verification time checked,
      series entry points to the summary, and no new full report staged. See
      readings file for byte counts, exit statuses, and severe flags.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: replication
cost proportional to genome length, reaching creatures through the body as
energy burned at each reproduction attempt that passes the target, cap, and
age gates. Expected compute cost: one `f32` multiply-add per charged
reproduction attempt; wall time per creature-tick flat; no compute severe and
no epoch re-pin budgeted for either profile. Gate references: the T11.F19 gate
and the gate epoch `remove-complementary-nutrition`. Goal references: the
T11.F19 goal, which is also the pinned goal-worlds epoch. The six work
counters are ecology counters read under the usual thresholds (flag +10%,
severe +50%); the charge changes every large parent's energy, so the
population's composition can move them, and `plasticity_updates` in
particular has read severe from clade composition at T13.F03 and T11.F19. A
severe there is read against the per-world evidence and, if it is
composition, is reported for the user's decision; it is not absorbed and not
attributed to compute. Every indicator not in the table (lineage diversity,
memory and temporal sensitivity, cognition, recruitment, drift-walk rows) is
reported with no direction.

| Reading | Reference (Orchards 11 / Canyon 22 / Confluence 33) | Predeclaration |
| --- | --- | --- |
| Goal persistence | T11.F19 final 13,808 / 10,546 / 13,789, plateau 10,244.8 / 9,985.4 / 13,547.8; T11.F04 `w1600` final 11,610 / 10,398 / 11,093 | No world extinct; final, minimum, and plateau reported, no direction. |
| `neighborhood_read.changed_per_all_births` (closure indicator) | Standing floors 0.146400 / 0.130000 / 0.143400 (T14.F12); T11.F19 read 0.278000 / 0.243600 / 0.274000 | Not below the floor on any world (strict); a world below its floor rejects the default rate. Against T11.F19 reported with no direction: fewer carried units mean fewer events per birth under the per-unit supply, so a fall that stays above the floor is the brake, not a delivery failure. |
| `neighborhood_read.mean_executed_nodes` (the executed core) | T11.F19 3.10 / 3.34 / 3.02; founder 2 | Not below 2.5 on any world; a world below 2.5 whose `neighborhood_read.mean_genome_size` is also below T11.F19's (355.4 / 287.8 / 344.7) rejects the default rate as shrinking the core, not only junk. Reported beside `mean_reachable_nodes` and `mean_total_nodes`. |
| `neighborhood_read.dead_per_all_births` | T11.F19 0.0066 / 0.0032 / 0.0104 | Reported; above 0.0150 on any world is investigated before closure. |
| Goal terminal `mean_genome_size` / `mean_mesh_nodes` | T11.F19 340.9 / 300.4 / 309.0 units; 6.91 / 6.26 / 7.06 nodes | Expected direction down or flat on each world; a reading above 1.10 times T11.F19's is investigated as the brake not biting at generation 50, not a rejection. |
| Goal `reachable_structure_size_distribution` | T11.F19 goal report | Reported, no direction. |
| Goal `action_charges.reproduce` per world | T11.F19 67,120.8 / 78,802.8 / 90,566.9 energy | Up on every world, multiple reported (the terminal factor is 20 to 24; the run's mean factor is lower because early genomes sit near the anchor): this is the applied-behavior evidence that the charge is on in every standard world. |
| Work counters and wall time, both profiles | T11.F19 and the epochs | Flag +10%, severe +50% on counters; wall flags/severe at +25%/+100% on a matching host; no compute severe budgeted. |
| Founder neighborhood, drift-walk, `recruitment_paths` blocks | T11.F19 goal | Byte-identical, field for field (none runs a reproduction action). |
| Paired Orchards run, seed 11, 12,000 ticks | T03.F08 pair on plains (386.5 vs 1,093.8 units at tick 12,000) is a reference, not this pair's control | Two `v3-cli run` arms from the release binary, each under `scripts/bench-wait`, sequential: `--ticks 12000 --sample-every 500 --seed 11 --config <recipe>`, the cost arm on `experiments/worlds/orchards-in-grassland.json` unchanged (defaults carry the rate) and the control arm on a stored copy of that recipe with `energy.lifecycle.genome_replication_cost_per_unit` 0.0 as its only addition. At tick 12,000 the cost arm's `mean_genome_size` and `mean_mesh_nodes` are strictly below the control's; neither arm extinct at any sample; `mean_generation`, units per generation `(size − 111) / generation`, and node slope `(nodes − 2) / generation` reported. Closure question, answered either way: the cost arm's `mean_genome_size` increment over ticks 8,000–12,000 against its increment over 4,000–8,000; below one quarter reads as levelling off. Budget 3 hours wall for the pair (the T11.F19 probe's 6,000-tick Orchards arm ran 2,052 s with the population at 85,000); a pair that exceeds it is reported and stops there. Each arm runs with `--save-config`, and the two applied configs are checked to differ in that one value only. Artifacts `cost.ndjson`, `control.ndjson`, `control-recipe.json`, `cost-config.json`, `control-config.json`, `timing.txt`, `run.sh` under `docs/progress/sweeps/t03-f11/`. |
| Observation budgets | Workflow caps | Founder neighborhood below 10 s; summed evolved below 180 s; neighborhood read below 10 s summed; drift walk below 30 s; goal run below 15 minutes. |

**Measured verdict.** Gate and goal both exit 0, `severe=false`, all six work
counters `ok` (goal counters all decreased; `plasticity_updates` -57% is
composition, not compute). All observation budgets under cap. No world
extinct; persistence, neighborhood-read guard, terminal
`mean_genome_size`/`mean_mesh_nodes`, and `action_charges.reproduce` all read
as predeclared, no ceiling crossed. Founder neighborhood, drift-walk, and
`recruitment_paths` blocks byte-identical to T11.F19 (config echo excepted).
Paired Orchards run: cost complete to tick 12,000, control stopped by user
decision at tick 4,500 — control-citing readings are truncated there. Cost strictly below control at the common tick; cost arm
alone does **not** level off (ratio 2.29 from increments of 4.02 and 9.21
units, 0.85% and 1.9% of a `mean_genome_size` held in a 462–506 band from
tick 4,500), while `mean_mesh_nodes` rises 10.44 → 25.29 with no plateau
(units per node ~45 → ~19). Full detail in
[`docs/progress/readings/t03-f11.md`](../../progress/readings/t03-f11.md).

- Summaries: [gate](../../progress/features/t03-f11-genome-replication-cost.json),
  [goal](../../progress/features/t03-f11-genome-replication-cost-goal.json).
- Full readings: [`docs/progress/readings/t03-f11.md`](../../progress/readings/t03-f11.md).

## Success Criteria

- [ ] A parent above 111 units pays the multiplied reproduce charge, the
      founder pays exactly the pre-feature charge, and the offspring transfer
      is unchanged, shown by tests and by `action_charges.reproduce` rising in
      all three standard worlds.
- [ ] `changed_per_all_births` is not below its floor and
      `mean_executed_nodes` is not below 2.5 on any world; no world extinct.
- [ ] The paired Orchards run is stored and read as predeclared, with the
      levelling-off question answered either way.
- [ ] Mutation gate and benchmark summaries recorded as the Verification items
      require.

## Notes for AI Agents

- Decision: (user, 2026-09-14) the charge is a multiplier on the Step 5
  reproduce charge anchored at the founder's 111 units, cached size is read,
  `complexity_cost` stays untouched, and the feature is sequenced directly
  after T11.F19 and depends on it; the drift walk is not its instrument.
- Decision: (user, 2026-09-15) the paired Orchards run exceeds the spec's
  3-hour wall budget: the cost arm completed 12,000 ticks in 8,506 s, and the
  control arm, at tick 4,000 after 32 minutes with 68,592 creatures carrying
  1,501.3 units in 43.19 nodes (generation 80.97; the cost arm at the same
  tick: 80,131 creatures, 473.5 units, 10.44 nodes, generation 93.3), then
  produced no sample for over an hour as its genomes grew (4.2 GB resident,
  CPU-bound on a quiet host), projecting more than 14 hours to finish. The
  user chose: the control arm runs until 12:00 local time and is then
  stopped; the size and node comparison is read at the last tick the control
  reached (at least 4,000), the cost arm is read in full to 12,000 and the
  levelling-off question is answered from it alone, and the control's wall
  time is recorded to the stop with its truncation stated. The truncated
  control is a limit on the pair's comparison depth, stated wherever the
  reading is cited. The 3-hour budget was the spec owner's sizing guard from
  the T11.F19 probe (2,052 s for 6,000 ticks at a population of 20,746 at
  tick 4,000), and the overrun is the population and genome growth on this
  substrate, not the host. Applies to this closure only.
