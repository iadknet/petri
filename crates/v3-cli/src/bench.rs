//! Deterministic benchmark harness (T10.F10).
//!
//! Runs a fixed set of seeded simulations and emits a full JSON report
//! per feature. The report's `deterministic` block is byte-identical for the
//! same commit and inputs; the `environment` block records host identity and
//! wall-clock as an unasserted secondary signal.

pub mod artifacts;
mod comparison;
mod indicators;
mod profiles;
mod run;
pub use run::detect_git_revision;
mod schema;
#[cfg(test)]
mod tests;
mod tracking;

pub use comparison::{
    apply_comparisons, apply_comparisons_for_outputs, compare_against, compare_against_path,
    default_gate_references, default_goal_references, BenchmarkSeriesIndex, Comparison,
    ComparisonInputs, MeasuredIdentity, ReferenceSelection, SeriesIndex,
};
use comparison::{
    FLAG_PERCENT, SEVERE_PERCENT, WALL_CLOCK_FLAG_PERCENT, WALL_CLOCK_SEVERE_PERCENT,
};
pub use profiles::{
    build_config, gate_profile_params, goal_profile_params, GoalCase, NeighborhoodSizes,
    ProfileParams, Recipe, COUNTER_NAMES, GOAL_WORLD_SET, SAMPLE_EVERY_TICKS, SCHEMA_VERSION,
};
pub use run::{
    build_report, build_report_with_threads, deterministic_block_json, report_json_pretty,
    rfc3339_now, run_deterministic, throughput_rates, RunTimings,
};
pub use schema::{
    ByReaderState, CaseComparison, CaseReadingComparison, CohortLadder, CohortLineageRow,
    ComparisonLevel, CounterComparison, Deterministic, DriftDepth, DriftDepthCheckpoint,
    Environment, EvolvedNeighborhoodHalf, FounderRow, GenerationDistribution, GoalCaseObservation,
    GoalIndicators, Host, Indicator, LineageDiversity, LineageDiversitySeed, LineageOpportunityRow,
    MemorySensitivity, MemorySensitivitySeed, MeshExecution, ModuleRecruitment,
    MovesBlockedByCause, MutationOpportunities, MutationSupply, MutationalNeighborhood,
    NeighborhoodBattery, NeighborhoodBirthBucket, NeighborhoodBirths, NeighborhoodCompanions,
    NeighborhoodEvolvedSeed, NeighborhoodFounderHalf, NeighborhoodOperatorRow, NeighborhoodRead,
    NeighborhoodReadGenome, NeighborhoodRequestedBirthBucket, NeighborhoodSampledGenome,
    NeighborhoodTally, OperatorOpportunityRow, PerCreatureTick, PerSeed, PopulationPersistence,
    PopulationPersistenceSeed, ProfileBlock, ReferenceComparison, Report, RetentionRow,
    SeedFinalStateObservation, SeedPhaseWallClock, SeedThroughput, SeedWallClock,
    StructuralCompanionsCensus, StructuralCompanionsSeed, StructureSizeDistribution,
    TemporalMemorySensitivity, TemporalMemorySensitivitySeed, Throughput, ThroughputRates,
    TimeToFirstRow, Totals, TrackedFractions, WallClockComparison, LINEAGE_DIVERSITY_VERSION,
    MEMORY_SENSITIVITY_VERSION, NEIGHBORHOOD_READ_VERSION, REACHABLE_STRUCTURE_VERSION,
};
pub use tracking::{
    ActionChargeTracking, CognitionTracking, EnergyFlowTracking, MortalityTracking,
    MutationOutcomeTotals, OccupancyGrid, PersistenceSample, PredationTracking,
    ReproductiveSuccessByCognitiveClassTracking, ReproductiveSuccessTotalsTracking, SensorCensus,
    WorldInputCensusRow, WorldTracking,
};
