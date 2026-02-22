pub mod actions;
pub mod seeding;
#[allow(clippy::module_inception)]
pub mod simulation;
pub mod tick;
pub use seeding::seed_simulation;
pub use simulation::Simulation;
pub use tick::run_tick;
