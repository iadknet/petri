//! T03.F10 — activity-ramped compute cost.
//!
//! The k-th instruction executed within one node dispatch costs its base opcode
//! charge plus `step_ramp_cost * max(0, k - step_ramp_allowance)`, accumulated
//! locally and settled against the creature's energy exactly once.

use super::*;
use crate::contracts::DynamicIntrospectionKey;
use crate::runtime::traced_vm::execute_vm_node_traced;
use crate::runtime::vm::step_charge;
use proptest::prelude::*;

/// Closed form of a dispatch's total charge: the summed base costs plus the
/// triangular ramp over the `m` steps past the allowance.
fn closed_form_total(steps: u32, base_cost: f32, cost_mult: f32, allowance: u32, ramp: f32) -> f64 {
    let m = f64::from(steps.saturating_sub(allowance));
    f64::from(steps) * f64::from(base_cost) * f64::from(cost_mult)
        + f64::from(ramp) * m * (m + 1.0) / 2.0
}

fn ramp_config(allowance: u32, ramp: f32, cost_mult: f32, max_steps: u32) -> RuntimeConfig {
    let mut cfg = config();
    cfg.max_vm_steps = max_steps;
    cfg.vm.opcode_cost_multiplier = cost_mult;
    cfg.vm.step_ramp_allowance = allowance;
    cfg.vm.step_ramp_cost = ramp;
    cfg
}

/// Runs `steps` Noops (control falls off the end of the program, so the step
/// count is exactly the program length) and returns the energy actually spent.
fn charge_for_noops(steps: u32, energy: f32, cfg: &RuntimeConfig) -> f32 {
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Noop; steps as usize],
    };
    let ss = empty_sensor_snapshot();
    let mut e = energy;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let result = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        cfg,
        &mut side_outputs,
    );
    assert!(!result.energy_exhausted, "dispatch must not exhaust");
    assert_eq!(side_outputs.work_counters.vm_steps, steps);
    energy - e
}

/// Per-step charges recorded by the traced executor for a Noop program.
fn traced_noop_steps(steps: u32, energy: f32, cfg: &RuntimeConfig) -> Vec<(f32, f32)> {
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Noop; steps as usize],
    };
    let ss = empty_sensor_snapshot();
    let mut e = energy;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let (_result, trace) = execute_vm_node_traced(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        cfg,
        &mut side_outputs,
    );
    trace
        .steps
        .iter()
        .map(|step| (step.energy_cost, step.energy_after))
        .collect()
}

// ── Charge shape ──────────────────────────────────────────────────────────

#[test]
fn a_dispatch_within_the_allowance_charges_base_costs_only() {
    // 8 Noops, allowance 10, ramp 1.0: no step is past the allowance.
    let cfg = ramp_config(10, 1.0, 1.0, 10_000);
    let charged = charge_for_noops(8, 1_000.0, &cfg);

    assert!(
        (f64::from(charged) - 8.0 * 0.05).abs() < 1e-4,
        "charged {charged} for 8 Noops within the allowance",
    );
}

#[test]
fn the_mth_step_past_the_allowance_charges_base_plus_m() {
    // Allowance 3, ramp 1.0: steps 4, 5, 6 pay 1.0, 2.0, 3.0 on top of Noop's 0.05.
    let cfg = ramp_config(3, 1.0, 1.0, 10_000);
    let steps = traced_noop_steps(6, 1_000.0, &cfg);

    let costs: Vec<f32> = steps.iter().map(|(cost, _)| *cost).collect();
    assert_eq!(costs.len(), 6);
    for (index, cost) in costs.iter().enumerate() {
        let k = index as u32 + 1;
        let expected = 0.05 + f32::from(u8::try_from(k.saturating_sub(3)).unwrap());
        assert!(
            (cost - expected).abs() < 1e-5,
            "step {k} charged {cost}, expected {expected}",
        );
    }
}

#[test]
fn zero_allowance_ramps_from_the_first_step() {
    let cfg = ramp_config(0, 1.0, 1.0, 10_000);
    let costs: Vec<f32> = traced_noop_steps(3, 1_000.0, &cfg)
        .iter()
        .map(|(cost, _)| *cost)
        .collect();

    assert!(
        (costs[0] - 1.05).abs() < 1e-5,
        "first step charged {}",
        costs[0]
    );
    assert!(
        (costs[1] - 2.05).abs() < 1e-5,
        "second step charged {}",
        costs[1]
    );
    assert!(
        (costs[2] - 3.05).abs() < 1e-5,
        "third step charged {}",
        costs[2]
    );
}

#[test]
fn a_zero_ramp_cost_charges_base_costs_only() {
    let cfg = ramp_config(3, 0.0, 1.0, 10_000);
    let charged = charge_for_noops(64, 1_000.0, &cfg);

    assert!(
        (f64::from(charged) - 64.0 * 0.05).abs() < 1e-3,
        "charged {charged} with the ramp disabled",
    );
}

#[test]
fn a_capped_dispatch_settles_the_closed_form_total_once() {
    // The Jump loop runs to the step cap; the charge is the closed form.
    let cfg = ramp_config(4, 1.0, 1.0, 20);
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Jump { offset: -1 }],
    };
    let ss = empty_sensor_snapshot();
    let mut e = 100_000.0f32;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let result = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut side_outputs,
    );

    assert!(!result.terminal && !result.energy_exhausted);
    assert_eq!(side_outputs.work_counters.vm_steps, 20);
    // Jump base 0.10 * 20 = 2.0, ramp 16 * 17 / 2 = 136.0.
    let expected = closed_form_total(20, 0.10, 1.0, 4, 1.0);
    assert!(
        (f64::from(100_000.0f32 - e) - expected).abs() < 0.02,
        "charged {} for a capped dispatch, expected {expected}",
        100_000.0f32 - e,
    );
}

// ── Production defaults ───────────────────────────────────────────────────

#[test]
fn at_production_defaults_a_capped_dispatch_costs_a_lethal_share_of_energy() {
    // 10,000 steps, allowance 100, ramp 1e-6: 1e-6 * 9900 * 9901 / 2 ≈ 49.01.
    let cfg = config();
    assert_eq!(cfg.max_vm_steps, 10_000);
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Jump { offset: -1 }],
    };
    let ss = empty_sensor_snapshot();
    let mut e = 200.0f32;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let result = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut side_outputs,
    );

    assert!(!result.energy_exhausted);
    let charged = 200.0f32 - e;
    assert!(
        (48.9..49.2).contains(&charged),
        "a capped dispatch charged {charged}, expected about 49",
    );
}

#[test]
fn at_production_defaults_a_thousand_step_dispatch_costs_about_one_tick_of_decay() {
    let mut cfg = config();
    cfg.max_vm_steps = 1_000;
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![VmInstruction::Jump { offset: -1 }],
    };
    let ss = empty_sensor_snapshot();
    let mut e = 200.0f32;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let _ = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut side_outputs,
    );

    let charged = 200.0f32 - e;
    assert!(
        (0.40..0.41).contains(&charged),
        "a 1,000-step dispatch charged {charged}, expected about 0.405",
    );
}

#[test]
fn at_production_defaults_the_settled_charge_survives_f32_rounding() {
    // 200 Noops from energy 20 cost 200 * 0.05 * 1e-6 + 1e-6 * 100 * 101 / 2.
    // Subtracting step by step would round every one of those charges away.
    let charged = charge_for_noops(200, 20.0, &config());
    let expected = closed_form_total(200, 0.05, 1e-6, 100, 1e-6);

    assert!(charged > 0.0, "a 200-step dispatch charged nothing");
    assert!(
        (f64::from(charged) - expected).abs() < 1e-6,
        "charged {charged}, expected {expected}",
    );
}

#[test]
fn at_production_defaults_a_short_program_is_nearly_free() {
    let charged = charge_for_noops(20, 20.0, &config());

    assert!(
        charged < 1e-5,
        "20 steps within the allowance charged {charged}",
    );
}

// ── Exhaustion, bids, and the trace sink ──────────────────────────────────

#[test]
fn a_dispatch_that_exhausts_mid_loop_leaves_energy_at_or_below_zero() {
    // Allowance 0, ramp 1.0: the k-th step costs about k, so energy 10 is gone
    // partway through the loop, well before the step cap.
    let cfg = ramp_config(0, 1.0, 1.0, 1_000);
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![10.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::StoreSlotImm {
                slot_idx: 7,
                src: 0,
            },
            VmInstruction::Jump { offset: -3 },
        ],
    };
    let ss = empty_sensor_snapshot();
    let mut e = 10.0f32;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let result = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut side_outputs,
    );

    assert!(result.energy_exhausted, "the dispatch must exhaust");
    assert!(e <= 0.0, "energy settled at {e}, expected at or below zero");
    assert!(
        side_outputs.work_counters.vm_steps < 1_000,
        "exhaustion must arrive before the step cap",
    );
    assert!(
        mem[7].abs() < f32::EPSILON,
        "an exhausted dispatch must not commit shared memory",
    );
}

#[test]
fn a_priority_bid_is_capped_at_the_effective_energy() {
    // Allowance 0, ramp 1.0: two steps owe about 1.05 + 2.05 before the bid,
    // so a bid of 100 can only take what is left of energy 10.
    let cfg = ramp_config(0, 1.0, 1.0, 1_000);
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![100.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::SetPriorityBid { src: 0 },
            VmInstruction::Halt,
        ],
    };
    let ss = empty_sensor_snapshot();
    let mut e = 10.0f32;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let result = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut side_outputs,
    );

    // LoadConst charges 0.08 + 1.0; the bid may take at most 10 - 1.08 - 2.10.
    assert!(result.energy_exhausted, "spending the remainder exhausts");
    assert!(
        e.abs() < 1e-4,
        "energy settled at {e}, expected zero within the settlement's rounding",
    );
    assert!(
        side_outputs.priority_bid <= 10.0,
        "bid {} exceeded the creature's energy",
        side_outputs.priority_bid,
    );
}

#[test]
fn a_bid_below_the_effective_energy_is_paid_in_full() {
    let cfg = ramp_config(0, 1.0, 1.0, 1_000);
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![4.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::SetPriorityBid { src: 0 },
            VmInstruction::Halt,
        ],
    };
    let ss = empty_sensor_snapshot();
    let mut e = 100.0f32;
    let mut mem = [0.0f32; 16];
    let prev_mem = [0.0f32; 16];
    let mut side_outputs = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let result = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut e,
        0.0,
        &mut mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut side_outputs,
    );

    assert!(!result.energy_exhausted);
    assert!((side_outputs.priority_bid - 4.0).abs() < 1e-5);
    // 3 steps at allowance 0 cost 1 + 2 + 3 in ramp plus 0.08 + 0.20 + 0.05.
    let expected = 100.0 - 4.0 - 6.0 - 0.33;
    assert!(
        (e - expected).abs() < 1e-3,
        "energy {e}, expected {expected}"
    );
}

#[test]
fn the_traced_executor_reports_the_effective_energy_per_step() {
    let cfg = ramp_config(2, 1.0, 1.0, 10_000);
    let steps = traced_noop_steps(5, 50.0, &cfg);

    let mut running = 50.0f64;
    for (index, (cost, after)) in steps.iter().enumerate() {
        running -= f64::from(*cost);
        assert!(
            (f64::from(*after) - running).abs() < 1e-4,
            "step {index} reported energy_after {after}, expected {running}",
        );
    }
}

#[test]
fn the_traced_and_untraced_executors_settle_the_same_energy() {
    let cfg = ramp_config(3, 0.25, 1.0, 500);
    let def = VmBackendDef {
        register_count: 2,
        constants: vec![1.0],
        program: vec![
            VmInstruction::LoadConst {
                dst: 0,
                const_idx: 0,
            },
            VmInstruction::Add { dst: 1, a: 1, b: 0 },
            VmInstruction::StoreSlotImm {
                slot_idx: 3,
                src: 1,
            },
            VmInstruction::Jump { offset: -4 },
        ],
    };
    let ss = empty_sensor_snapshot();
    let prev_mem = [0.0f32; 16];

    let mut plain_energy = 60.0f32;
    let mut plain_mem = [0.0f32; 16];
    let mut plain_side = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let plain = execute_vm_node(
        &def,
        &[],
        &zeroed_upstream(),
        &mut plain_energy,
        0.0,
        &mut plain_mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut plain_side,
    );

    let mut traced_energy = 60.0f32;
    let mut traced_mem = [0.0f32; 16];
    let mut traced_side = MeshSideOutputs::new(cfg.max_actions_per_turn);
    let (traced, trace) = execute_vm_node_traced(
        &def,
        &[],
        &zeroed_upstream(),
        &mut traced_energy,
        0.0,
        &mut traced_mem,
        &prev_mem,
        &ss,
        &cfg,
        &mut traced_side,
    );

    assert_eq!(plain_energy, traced_energy);
    assert_eq!(plain.energy_exhausted, traced.energy_exhausted);
    assert_eq!(plain_mem, traced_mem);
    assert_eq!(
        plain_side.work_counters.vm_steps,
        traced_side.work_counters.vm_steps,
    );
    let traced_total: f64 = trace.steps.iter().map(|s| f64::from(s.energy_cost)).sum();
    assert!(
        (traced_total - f64::from(60.0f32 - plain_energy)).abs() < 1e-3,
        "trace charges sum to {traced_total}, settlement took {}",
        60.0f32 - plain_energy,
    );
}

#[test]
fn a_mid_dispatch_energy_read_sees_the_effective_energy() {
    // ReadInput of EnergyCurrent after two ramped steps must read energy minus
    // what the dispatch already owes, not the unsettled starting energy.
    let cfg = ramp_config(0, 1.0, 1.0, 1_000);
    let def = VmBackendDef {
        register_count: 1,
        constants: vec![],
        program: vec![
            VmInstruction::Noop,
            VmInstruction::ReadInput {
                dst: 0,
                ref_idx: 0,
                sub_idx: 0,
            },
            VmInstruction::WriteInternalPayload {
                slot_idx: 0,
                src: 0,
            },
            VmInstruction::Halt,
        ],
    };
    let refs = vec![InputReference::DynamicIntrospection(
        DynamicIntrospectionKey::EnergyCurrent,
    )];
    let (result, _energy, _side) = run_vm_with_config(
        def.program.clone(),
        1,
        vec![],
        &refs,
        zeroed_upstream(),
        50.0,
        cfg,
    );

    // Step 1 (Noop) owes 1.05, step 2 (ReadInput) owes 2.12: 50 - 3.17.
    let seen = result.output_slots[0];
    assert!(
        (seen - 46.83).abs() < 0.05,
        "the brain read {seen} energy, expected about 46.83",
    );
}

// ── Properties of the pure charge ─────────────────────────────────────────

proptest! {
    /// Summing the per-step charge reproduces the closed form for any allowance,
    /// step count, and ramp rate.
    #[test]
    fn summed_step_charges_match_the_closed_form(
        steps in 0u32..600,
        allowance in 0u32..400,
        ramp in 0.0f32..2.0,
        cost_mult in 0.0f32..2.0,
    ) {
        let base = 0.12f32;
        let summed: f64 = (1..=steps as usize)
            .map(|k| step_charge(base, cost_mult, k, allowance, ramp))
            .sum();
        let expected = closed_form_total(steps, base, cost_mult, allowance, ramp);

        prop_assert!(
            (summed - expected).abs() <= 1e-9 * expected.abs().max(1.0),
            "summed {summed}, closed form {expected}",
        );
    }

    /// The total charge never decreases when a dispatch runs one more step or
    /// when the ramp rate rises.
    #[test]
    fn the_total_charge_is_monotone_in_steps_and_in_ramp_rate(
        steps in 0u32..400,
        allowance in 0u32..300,
        ramp in 0.0f32..2.0,
        extra in 0.0f32..2.0,
    ) {
        let base = 0.12f32;
        let total = closed_form_total(steps, base, 1.0, allowance, ramp);
        let one_more = closed_form_total(steps + 1, base, 1.0, allowance, ramp);
        let steeper = closed_form_total(steps, base, 1.0, allowance, ramp + extra);

        prop_assert!(one_more >= total, "{one_more} < {total} for one more step");
        prop_assert!(steeper >= total, "{steeper} < {total} for a steeper ramp");
        prop_assert!(step_charge(base, 1.0, steps as usize + 1, allowance, ramp) >= 0.0);
    }

    /// Steps within the allowance carry the base charge and nothing else, at any
    /// ramp rate.
    #[test]
    fn steps_within_the_allowance_carry_no_ramp(
        allowance in 1u32..500,
        offset in 0u32..500,
        ramp in 0.0f32..2.0,
        cost_mult in 0.0f32..2.0,
    ) {
        let k = (offset % allowance) as usize + 1;
        let base = 0.12f32;

        prop_assert!(
            (step_charge(base, cost_mult, k, allowance, ramp)
                - f64::from(base) * f64::from(cost_mult))
                .abs()
                < 1e-12,
        );
    }

    /// A whole dispatch settles the closed-form total against the creature's
    /// energy in one subtraction, so what the mesh attributes to the node is the
    /// closed form within one ulp of the starting energy.
    #[test]
    fn a_settled_dispatch_charges_the_closed_form_within_one_ulp(
        steps in 1u32..300,
        allowance in 0u32..200,
        // Bounded so the closed-form total (at most 300 * 301 / 2 * 1e-4 ≈ 4.5)
        // always fits inside the drawn energy: this property is about the
        // settlement, and an exhausted dispatch never settles the whole total.
        ramp in 0.0f32..1e-4,
        energy in 20.0f32..200.0,
    ) {
        let cfg = ramp_config(allowance, ramp, 1e-6, 10_000);
        let charged = charge_for_noops(steps, energy, &cfg);
        let expected = closed_form_total(steps, 0.05, 1e-6, allowance, ramp);
        // One ulp of the starting energy, plus one of the settled debt.
        let tolerance = f64::from(f32::EPSILON) * (f64::from(energy) + expected);

        prop_assert!(
            (f64::from(charged) - expected).abs() <= tolerance,
            "charged {charged}, closed form {expected}, tolerance {tolerance}",
        );
    }
}
