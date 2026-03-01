pub mod actions;
#[allow(dead_code)] // Wired in Stage 2
pub(crate) mod outcomes;
pub mod seeding;
#[allow(clippy::module_inception)]
pub mod simulation;
pub mod stats;
pub mod tick;
pub use seeding::seed_simulation;
pub use simulation::Simulation;
pub use stats::SimStats;
pub use tick::run_tick;
