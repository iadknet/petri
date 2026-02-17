/// Energy costs for creature actions.
#[derive(Clone, Debug)]
pub struct EnergyCosts {
    /// Energy spent when a creature moves.
    pub move_cost: u32,
    /// Energy spent when a creature eats (typically 0).
    pub eat_cost: u32,
    /// Energy spent when a creature does nothing (typically 0).
    pub noop_cost: u32,
    /// Energy gained per unit of food consumed.
    pub eat_reward_per_food: u32,
}

impl Default for EnergyCosts {
    fn default() -> Self {
        Self {
            move_cost: 1,
            eat_cost: 0,
            noop_cost: 0,
            eat_reward_per_food: 1,
        }
    }
}
