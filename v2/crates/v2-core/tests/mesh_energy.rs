use v2_core::energy::{charge_action, charge_backend_graph, charge_dispatch_entry, meter_vm_ops};

fn approx_eq(a: f32, b: f32) {
    assert!((a - b).abs() < 1e-6, "expected {a} ~= {b}");
}

#[test]
fn dispatch_entry_energy_charge_is_applied() {
    let result = charge_dispatch_entry(0.10, 0.03);
    approx_eq(result.remaining_energy, 0.07);
    approx_eq(result.charged_energy, 0.03);
    assert!(!result.exhausted);
}

#[test]
fn graph_static_tariff_is_charged_per_dispatch() {
    let result = charge_backend_graph(0.08, 0.05);
    approx_eq(result.remaining_energy, 0.03);
    approx_eq(result.charged_energy, 0.05);
    assert!(!result.exhausted);
}

#[test]
fn vm_loop_is_bounded_by_energy_exhaustion() {
    let result = meter_vm_ops(0.07, 0.03, usize::MAX);
    assert_eq!(result.executed_ops, 3);
    approx_eq(result.remaining_energy, 0.0);
    assert!(result.exhausted);
}

#[test]
fn action_cost_is_charged_after_world_action_commit() {
    let result = charge_action(0.04, 0.02);
    approx_eq(result.remaining_energy, 0.02);
    approx_eq(result.charged_energy, 0.02);
    assert!(!result.exhausted);
}
