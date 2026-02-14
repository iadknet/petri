use v2_core::energy::vm_opcode_base_cost;
use v2_core::mesh::VmInstruction;

#[test]
fn opcode_cost_table_has_expected_non_zero_defaults() {
    assert!(vm_opcode_base_cost(&VmInstruction::Noop) > 0.0);
    assert!(
        vm_opcode_base_cost(&VmInstruction::ReadSensorCreature {
            dst: 0,
            dx: 0,
            dy: 0,
            field: v2_core::mesh::SensorCreatureField::PresentFlag,
        }) > vm_opcode_base_cost(&VmInstruction::Noop)
    );
}
