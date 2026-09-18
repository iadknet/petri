# Why Orchards Collapses (2026-09-17)

Investigation of the Orchards in grassland goal case (seed 11, 1600², 10,000 founders, 2,000 ticks) on main at `7f5d8994` (the T16.F01 closure). The goal reports since T11.F21 read final population 8 / 10 / 7; T02.F04 read 10,949 with a minimum of 80; T11.F19 and earlier read final populations between 8,800 and 20,100 with minima above 1,100. Evidence: the committed goal summaries' `population_persistence.per_seed[0].samples` and `surviving_clade_profiles`, plus one re-run of the case with a temporary in-vitro probe (Appendix A) that reproduces the committed trajectory exactly (96,879 / 1,350 / 131 / 59 / 7 at ticks 100 / 200 / 300 / 400 / 2,000).

## Verdict

**The population is not starving. It is sterile.** After the tick-100 boom-and-bust, every survivor carries a lesion on the founder's reproduce circuit. From tick 250 on there are zero births for 1,750 ticks while grass regrows to 1.46 M units (2.5× the tick-zero standing crop of 592 k) and the survivors sit at the 200-energy cap eating and moving every tick. The population can only decline from there; it does, by attrition, from 322 at tick 250 to 7.

The chain, in order:

1. **Boom.** The diffuse grass standing crop (65% coverage × 0.4 density × 4 energy per unit) carries 10,000 founders to the 100,000 cap by tick 75. Every world does this (`experiments/worlds/README.md`).
2. **Bust, deepened by grazing.** Grass falls from 592 k units to 10 k by tick 150 and the modifier under the boom is bitten to its 0.05 floor with a 1,000-tick recovery (T02.F04 defaults 0.5 / 0.05 / 1000), so regrowth stays suppressed through the famine. Before grazing (T11.F19) the trough was 1,118 at tick ~200 with grass back to 420 k by tick 300; with grazing it is 59–131 at ticks 300–400 with grass at 114 k by tick 300. The trough is over ten times deeper and about 150 ticks longer.
3. **The famine selects against fertility.** The founder reproduces whenever energy > 30 and age ≥ 20, handing 20 energy to an offspring that starves. A fertile creature therefore never holds more than ~30 energy during the famine; a creature whose reproduce circuit is broken keeps every bite. In the probe, fertile mean energy is 9–16 versus sterile 37–40 from tick 100 to 200, and the in-vitro-fertile share of the population falls 96% → 89% → 74% → 40% → 15% → 7.5% at ticks 100 / 150 / 175 / 200 / 225 / 250. The last births happen between ticks 200 and 225.
4. **At tick 400 all 59 survivors are sterile in vivo**: lifetime `Reproduce` attempts 0, offspring 0, for every one (Appendix B). 53 are sterile from a fresh runtime state; the other 6 carry `GraphEnableHebbian` on the energy or age gate edge and are fertile from a fresh state, but their learned edge weight in the world sits at the negative clamp (−1.09 to −4.35), so the gate never fires and, with the gate output constant, never relearns. `GraphEnableHebbian` is the only way a fertile-looking genome survived the famine, and it survived for the same reason: it behaved sterile.
5. **Zero births, then attrition.** With no fertile genome left, the world's recovery is irrelevant. The 59 wander (food- and barrier-blind, ~half their ticks eating), and the ones that leave a meadow die of move charges and decay one at a time.

T02.F04's own run went through the same trough (87 at tick 400, mean energy 131, only 23 births between ticks 300 and 400) but kept at least one fertile lineage, whose descendants rebuilt the population (generation 2.6 → 8.5 → 22 by ticks 400 / 500 / 1,000). T11.F21's draw re-mapping perturbed the trajectory enough that the last unlesioned fertile creatures (five at tick 250, all at 0–6 energy, one with 55 lifetime reproduce attempts) were gone by tick 300; T11.F22 and T16.F01 each moved the trajectory again (final 10 and 7) and landed in the same basin. Whether Orchards recovers is a coin flip on which handful survives a trough of ~60–130, and the coin is weighted against fertility.

## What each survivor lost

Every tick-400 survivor's genome, diffed against the founder, breaks one link in `EnergyCurrent → Threshold(30) → Multiply ← Threshold(19.5) ← AgeTicks → CustomOutput(1) → node 1 slot 1 → CmpGt → Reproduce`:

| Lesion | Operators seen | Survivors (of 59) |
| --- | --- | --- |
| Node 1's `input_refs[1]` (the can-reproduce slot) re-pointed to an unwritten upstream slot (8–22) | `InputRefSwap`, `InputRefRawFieldMutation` | 13 |
| Node 0's `EnergyCurrent` or `AgeTicks` reference swapped to `Generation` (1–6, never past either threshold) or `EnergyConsumedThisTick` | `InputRefSwap`, `InputRefPrune` | 14 |
| CN2 (the Multiply) removed, leaving `output_sinks[1]` pointing at compute node 65535, or CN0/CN1 removed and the survivors re-indexed | `GraphRemoveInternalGraphNode`, `GraphCopyInternalNode` | 11 |
| Energy edge, age edge, or the sink edge deleted | `GraphRemoveGraphEdge` | 6 |
| Gate input re-pointed at `food_here` (0–1) or shared memory | `GraphRetargetGraphEdge`, `GraphRawFieldMutation` | 7 |
| Hebbian plasticity on a gate edge, weight learned to the negative clamp in vivo | `GraphEnableHebbian` | 6 (incl. one unmutated copy of a carrier) |
| VM `ReadInput dst 1 → 2` or an inserted motif shifting the reproduce branch | `VmInstructionRawFieldMutation`, `VmInsertLoadCompareMotif` | 2 |

(Some survivors carry two lesions; the table counts one per survivor, in row order.) These are ordinary single-event mutations at the T11.F19 supply rate; nothing here is a mutation-operator defect. It is the founder's reproduce rule that makes the lesion adaptive for 150 ticks.

## Timeline from the probe

| Tick | Population | Fertile in vitro | Fertile / sterile mean energy | Births in window | Grass units (fruit's constant 468 k excluded) | Gen-0 founders alive |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 10,000 | 10,000 | 20.0 / – | – | 591,861 | 10,000 |
| 75 | 100,000 | 96,429 | 22.7 / 39.2 | 46,475 | 364,059 | 8,193 |
| 100 | 96,879 | 92,878 | 15.9 / 38.3 | 35,264 | 159,782 | 4,360 |
| 150 | 18,879 | 16,769 | 10.3 / 39.4 | 4,088 | 9,957 | 148 |
| 200 | 1,350 | 541 | 10.2 / 40.1 | 64 | 10,491 | 1 |
| 225 | 586 | 85 | 17.9 / 37.9 | 8 | 21,956 | 0 |
| 250 | 322 | 24 | 28.7 / 36.5 | 0 | 41,353 | 0 |
| 300 | 131 | 13 | 28.2 / 38.9 | 0 | 113,716 | 0 |
| 400 | 59 | 6 (all Hebbian, 0 in vivo) | 134.2 / 156.9 | 0 | 422,864 | 0 |
| 1,000 | 20 | 4 | 189.5 / 180.6 | 0 | 903,809 | 0 |
| 2,000 | 7 | 1 | 198.0 / 112.5 | 0 | 1,461,452 | 0 |

"Fertile in vitro" is whether the genome emits `Reproduce` from a fresh runtime state at energy 150, age 100, food on every side. Births are `reproduction_actions_spawned_total` deltas. Deaths after tick 400 are `ActionMove` and `LifecycleDecay`, plus one `GenomeCarrying`.

## What this is not

- Not a reproduce-gate bug: `reproduction_actions_rejected_total` is flat at 598,041 from tick 250 (no attempts, so nothing to reject), and `lifetime_invalid_reproduce_count` is 0 on every survivor.
- Not grazing alone: grazing sets the trough depth, but the same trough in T02.F04's run recovered. Grazing turned a recovery the pre-T02.F04 worlds always made (trough ~1,100, fertile share never near zero) into one that depends on which ~60 creatures survive.
- Not Orchards-specific in mechanism: the founder's rule and the grazing floor apply in every world, and Confluence in T11.F21's run fell to 12 at tick 1,000 and recovered to 8,105 only because a fertile lineage was among the 12. Orchards is the world where the diffuse-grass boom is largest relative to the meadows that feed the plateau, so its trough is deepest and the coin flip is worst.

## Options (the user's call)

1. **Make the founder's reproduce rule energy-reserving**, e.g. gate on energy > 30 + transfer (so a parent never drops below the threshold by reproducing) or raise the threshold. This is the founder-behavior lever (`crates/v3-core/src/creature/founder.rs`, `cgp_founder.rs`); `cargo test -p v3-core --test viability` first, and it moves every trajectory.
2. **Shallow the Orchards trough in the recipe**: less diffuse grass (`initial_coverage` 0.65 → lower) or a faster grazing recovery for the Orchards grass type, so the boom is smaller and the famine shorter. World-only; `inputs_changed` on the Orchards case.
3. **Accept it as a world finding** and read Orchards' collapse as the T02/T12 world-fragility signal it already is in the T11.F21 ruling.

Option 1 is the one that addresses the cause; options 2 and 3 address the trough.

## Appendix A: the probe

Dropped into `crates/v3-core/tests/zz_probe_orchards.rs` and removed after the run, following the earlier notes' pattern. Public `v3_core` APIs only. Run with `cargo test --release -p v3-core --test zz_probe_orchards probe_orchards_collapse -- --nocapture --exact` (174 s on the recording host); `PETRI_TICKS` and `PETRI_DUMP` shorten the run and choose dump ticks; `probe_hebbian_drift` reads a saved log via `PETRI_LOG`.

```rust
//! TEMPORARY PROBE (not for commit): why does the Orchards goal world collapse?
use std::collections::BTreeMap;

use v3_core::config::{resolve_config, RuntimeConfig, SimulationConfig};
use v3_core::contracts::WorldAction;
use v3_core::creature::genome::CreatureGenome;
use v3_core::creature::state::{CreatureState, GraphRuntimeState};
use v3_core::runtime::mesh::execute_creature_mesh;
use v3_core::sensors::perception::{PerceptionSnapshot, SensorSnapshot};
use v3_core::sensors::static_inputs::StaticInputs;
use v3_core::sensors::typed_food::TypedFoodLocalSnapshot;
use v3_core::simulation::energy_accounting::DeathCause;
use v3_core::simulation::seed_simulation;
use v3_core::simulation::tick::run_tick;

fn orchards_config() -> SimulationConfig {
    let recipe: serde_json::Value = serde_json::from_str(include_str!(
        "../../../experiments/worlds/orchards-in-grassland.json"
    ))
    .unwrap();
    let mut config = resolve_config(&SimulationConfig::default(), recipe).unwrap();
    config.world.width = 1600;
    config.world.height = 1600;
    config.population.initial_creatures = 10_000;
    config.normalize();
    config.apply_startup_overrides();
    config
}

/// Generous in-vitro conditions: energy far above the founder's 30 gate, age
/// far past the 20-tick gate, food here and on every neighbor.
fn generous_sensors(food_types: usize) -> SensorSnapshot {
    SensorSnapshot {
        local: StaticInputs {
            food_here: 1.0,
            neighbor_food: [0.5; 8],
            neighbor_barrier: [0.0; 8],
            neighbor_occupied: [0.0; 8],
            generation: 1.0,
            age_ticks: 100.0,
        },
        typed_local_food: TypedFoodLocalSnapshot {
            food_here_by_type: vec![1.0; food_types],
            neighbor_food_by_type: vec![[0.5; 8]; food_types],
        },
        perception: PerceptionSnapshot::zeroed(food_types),
    }
}

fn emits_reproduce(
    genome: &CreatureGenome,
    sensors: &SensorSnapshot,
    runtime: &RuntimeConfig,
    energy: f32,
    age: u64,
) -> bool {
    let mut energy = energy;
    let mut shared = [0.0f32; 16];
    let prev = [0.0f32; 16];
    let mut gr = GraphRuntimeState::new();
    gr.begin_tick(&genome.nodes, age);
    let out = execute_creature_mesh(genome, sensors, &mut energy, &mut shared, &prev, &mut gr, runtime);
    out.actions
        .iter()
        .any(|a| matches!(a, WorldAction::Reproduce { .. }))
}

fn survivor_line(c: &CreatureState, fertile: bool, tick: u64, sensors: &SensorSnapshot, runtime: &RuntimeConfig) -> serde_json::Value {
    // Same generous inputs, but the creature's own learned runtime state and shared memory.
    let mut energy = 150.0f32;
    let mut shared = c.shared_memory;
    let prev = c.prev_shared_memory;
    let mut gr = c.graph_runtime.clone();
    gr.begin_tick(&c.genome.nodes, c.age);
    let out = execute_creature_mesh(&c.genome, sensors, &mut energy, &mut shared, &prev, &mut gr, runtime);
    let fertile_with_learned_state = out.actions.iter().any(|a| matches!(a, WorldAction::Reproduce { .. }));
    let weights: Vec<Vec<f32>> = c.graph_runtime.plasticity_weights.get(0).map(|n| n.iter().map(|w| w.to_vec()).collect()).unwrap_or_default();
    serde_json::json!({
        "fertile_with_learned_state": fertile_with_learned_state,
        "node0_plasticity_weights": weights,
        "kind": "survivor",
        "tick": tick,
        "id": format!("{:?}", c.id),
        "generation": c.generation,
        "energy": c.energy,
        "age": c.age,
        "genome_size": c.cached_genome_size,
        "fertile_in_vitro": fertile,
        "birth_ops": c.birth_mutation_operators.iter().map(|o| format!("{o:?}")).collect::<Vec<_>>(),
        "actions_by_type": c.lifetime_actions_attempted_by_type,
        "invalid_reproduce": c.lifetime_invalid_reproduce_count,
        "offspring": c.offspring_spawned_count,
        "blocked_moves": c.lifetime_blocked_move_count,
        "position": [c.position.x, c.position.y],
        "genome": c.genome,
    })
}

#[test]
fn probe_orchards_collapse() {
    let config = orchards_config();
    let food_types = config.world.food.types.len();
    let runtime = config.runtime.clone();
    let sensors = generous_sensors(food_types);
    let ticks: u64 = std::env::var("PETRI_TICKS").ok().and_then(|v| v.parse().ok()).unwrap_or(2000);
    let dump_ticks: Vec<u64> = std::env::var("PETRI_DUMP")
        .ok()
        .map(|v| v.split(',').filter_map(|t| t.parse().ok()).collect())
        .unwrap_or_else(|| vec![250, 300, 400, 2000]);
    let mut sample_ticks: Vec<u64> = vec![0, 25, 50, 75, 100, 125, 150, 175, 200, 225, 250, 275, 300, 350, 400, 450, 500];
    let mut t = 600;
    while t <= ticks {
        sample_ticks.push(t);
        t += 100;
    }

    let mut sim = seed_simulation(config, 11);
    // Founder control: the unmutated founder must emit Reproduce under these inputs.
    let founder = sim.creatures.values().next().unwrap().genome.clone();
    assert!(emits_reproduce(&founder, &sensors, &runtime, 150.0, 100));
    println!("{}", serde_json::json!({"kind": "founder", "genome": founder}));

    let mut last_deaths = [0u64; DeathCause::ALL.len()];
    let mut last_births = 0u64;
    let sample = |sim: &v3_core::simulation::Simulation, last_deaths: &mut [u64; 17], last_births: &mut u64, dump: bool| {
        let tick = sim.tick;
        let mut by_gen: BTreeMap<u64, (u64, u64, f64, f64)> = BTreeMap::new();
        let mut fertile_total = 0u64;
        let mut fertile_energy = 0f64;
        let mut sterile_energy = 0f64;
        let mut fertile_repro_attempts = 0u64;
        let mut sterile_repro_attempts = 0u64;
        for c in sim.creatures.values() {
            let fertile = emits_reproduce(&c.genome, &sensors, &runtime, 150.0, 100);
            let e = by_gen.entry(c.generation.min(5)).or_default();
            e.0 += 1;
            if fertile {
                e.1 += 1;
                e.2 += f64::from(c.energy);
                fertile_total += 1;
                fertile_energy += f64::from(c.energy);
                fertile_repro_attempts += c.lifetime_actions_attempted_by_type[3];
            } else {
                e.3 += f64::from(c.energy);
                sterile_energy += f64::from(c.energy);
                sterile_repro_attempts += c.lifetime_actions_attempted_by_type[3];
            }
            if dump {
                println!("{}", survivor_line(c, fertile, tick, &sensors, &runtime));
            }
        }
        let pop = sim.creatures.len() as u64;
        let sterile_total = pop - fertile_total;
        let deaths: BTreeMap<String, u64> = DeathCause::ALL
            .iter()
            .enumerate()
            .filter_map(|(i, cause)| {
                let d = sim.stats.mortality.by_cause[i] - last_deaths[i];
                (d > 0).then(|| (format!("{cause:?}"), d))
            })
            .collect();
        *last_deaths = sim.stats.mortality.by_cause;
        let births = sim.stats.reproduction_actions_spawned_total;
        let gens: BTreeMap<String, serde_json::Value> = by_gen
            .iter()
            .map(|(g, (n, f, fe, se))| {
                (
                    format!("gen{g}"),
                    serde_json::json!({
                        "n": n,
                        "fertile": f,
                        "fertile_mean_energy": if *f > 0 { fe / *f as f64 } else { 0.0 },
                        "sterile_mean_energy": if n - f > 0 { se / (n - f) as f64 } else { 0.0 },
                    }),
                )
            })
            .collect();
        println!(
            "{}",
            serde_json::json!({
                "kind": "sample",
                "tick": tick,
                "population": pop,
                "fertile": fertile_total,
                "sterile": sterile_total,
                "fertile_mean_energy": if fertile_total > 0 { fertile_energy / fertile_total as f64 } else { 0.0 },
                "sterile_mean_energy": if sterile_total > 0 { sterile_energy / sterile_total as f64 } else { 0.0 },
                "fertile_repro_attempts_lifetime": fertile_repro_attempts,
                "sterile_repro_attempts_lifetime": sterile_repro_attempts,
                "births_since_last": births - *last_births,
                "births_total": births,
                "repro_attempted_total": sim.stats.reproduction_actions_attempted_total,
                "repro_rejected_total": sim.stats.reproduction_actions_rejected_total,
                "deaths_since_last": deaths,
                "food_total": sim.world.total_food(),
                "by_generation": gens,
            })
        );
        *last_births = births;
    };

    sample(&sim, &mut last_deaths, &mut last_births, false);
    for _ in 0..ticks {
        run_tick(&mut sim, &mut None);
        let tick = sim.tick;
        if sample_ticks.contains(&tick) || dump_ticks.contains(&tick) {
            sample(&sim, &mut last_deaths, &mut last_births, dump_ticks.contains(&tick));
        }
    }
}

/// Hebbian carriers: fertile from a fresh runtime state, but do they stay
/// fertile once the plasticity has run for a few hundred ticks? Constant
/// generous inputs, persistent graph runtime state, age advancing.
#[test]
fn probe_hebbian_drift() {
    let log = std::env::var("PETRI_LOG").expect("PETRI_LOG");
    let config = orchards_config();
    let runtime = config.runtime.clone();
    let sensors = generous_sensors(config.world.food.types.len());
    for line in std::fs::read_to_string(&log).unwrap().lines() {
        if !line.contains("\"kind\":\"survivor\"") {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        if v["tick"].as_u64() != Some(400) || v["fertile_in_vitro"] != true {
            continue;
        }
        let genome: CreatureGenome = serde_json::from_value(v["genome"].clone()).unwrap();
        let mut shared = [0.0f32; 16];
        let mut gr = GraphRuntimeState::new();
        let mut first_sterile_tick = None;
        let mut repro_ticks = 0u64;
        for age in 0..400u64 {
            let mut energy = 150.0f32;
            let prev = shared;
            gr.begin_tick(&genome.nodes, age);
            let out = execute_creature_mesh(&genome, &sensors, &mut energy, &mut shared, &prev, &mut gr, &runtime);
            let repro = out.actions.iter().any(|a| matches!(a, WorldAction::Reproduce { .. }));
            if repro {
                repro_ticks += 1;
            } else if age >= 20 && first_sterile_tick.is_none() {
                first_sterile_tick = Some(age);
            }
        }
        let weights: Vec<Vec<f32>> = gr.plasticity_weights.get(0).map(|n| n.iter().map(|w| w.to_vec()).collect()).unwrap_or_default();
        println!(
            "{}",
            serde_json::json!({
                "kind": "hebbian",
                "generation": v["generation"],
                "birth_ops": v["birth_ops"],
                "repro_ticks_of_400": repro_ticks,
                "first_sterile_tick": first_sterile_tick,
                "node0_plasticity_weights_final": weights,
            })
        );
    }
}
```

## Appendix B: the six Hebbian carriers at tick 400

Under constant generous inputs from a fresh state, five of the six keep the gate edge at the positive clamp and reproduce on all 400 ticks (the sixth, an `AntiHebb` carrier, goes sterile at tick 20). In the world, all six have the weight at the negative clamp:

| Generation | Birth operators | Weight learned in vivo | Reproduce attempts, lifetime |
| ---: | --- | ---: | ---: |
| 1 | `GraphEnableHebbian` | −1.091 | 0 |
| 1 | `GraphEnableHebbian` | −4.349 | 0 |
| 1 | `GraphEnableHebbian`, `InputRefSwap` | −1.709 | 0 |
| 1 | `GraphEnableHebbian`, `GraphMutateGraphOperatorParam`, `TopologyMutateGateBias` | −4.050 | 0 |
| 2 | none (unmutated copy of an `AntiHebb` carrier) | −0.996 | 0 |
| 1 | `GraphEnableHebbian` | −3.966 | 0 |
