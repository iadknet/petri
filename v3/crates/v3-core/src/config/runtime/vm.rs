/// VM execution configuration.
#[derive(Clone, Debug)]
pub struct VmConfig {
    /// Global multiplier applied to all per-opcode base costs before energy
    /// deduction. Base costs are the f32 values from the ISA spec cost table.
    /// With `opcode_cost_multiplier = 1`, VM execution is effectively free
    /// relative to food/decay values. Increase to make cognition more expensive.
    pub opcode_cost_multiplier: u32,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            opcode_cost_multiplier: 1,
        }
    }
}
