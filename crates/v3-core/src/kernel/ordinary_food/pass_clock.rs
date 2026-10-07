//! Wall time of the grid passes inside food growth, for tick traces (T24.F03).
//!
//! `grow` brackets each pass with [`PassClock::start`] and [`PassClock::stop`].
//! Under `telemetry-seams` the clock reads `Instant::now()` around the pass and
//! the timings leave `grow` in [`FoodGrowthSummary`]; without the feature the
//! clock is a zero-sized no-op, so the reference build reads no clock here.
//! Timing never reorders, splits or skips a pass.

#[cfg(feature = "telemetry-seams")]
use std::time::{Duration, Instant};

use crate::config::OrdinaryFoodTypeId;

use super::ecology::FoodGrowthSummary;

/// One grid pass of food growth, in execution order. A per-type pass runs
/// once for each food type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FoodPass {
    /// The occupancy-depletion recovery and deposit call.
    DepletionRecovery,
    /// The grazing recovery call.
    GrazingRecovery,
    /// The type-inhibition weighted sums and the inhibited-cell flags.
    InhibitionSums,
    /// The full-grid density sum of one type.
    DensityTotal,
    /// The growth and spread loop of one type, with its scratch allocation.
    GrowthSpread,
    /// The recovery spawns of one type below the recovery floor.
    RecoverySpawns,
    /// The density write-back and per-type telemetry of one type.
    WriteBack,
    /// The count of inhibited cells.
    InhibitedCount,
}

impl FoodPass {
    /// The pass's span name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::DepletionRecovery => "depletion_recovery",
            Self::GrazingRecovery => "grazing_recovery",
            Self::InhibitionSums => "inhibition_sums",
            Self::DensityTotal => "density_total",
            Self::GrowthSpread => "growth_spread",
            Self::RecoverySpawns => "recovery_spawns",
            Self::WriteBack => "write_back",
            Self::InhibitedCount => "inhibited_count",
        }
    }
}

/// One pass that ran: which, for which food type (per-type passes only),
/// when it started and how long it took.
#[cfg(feature = "telemetry-seams")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FoodPassTiming {
    pub pass: FoodPass,
    pub food_type: Option<OrdinaryFoodTypeId>,
    pub started: Instant,
    pub elapsed: Duration,
}

/// The passes one `grow` call ran, in execution order.
#[cfg(feature = "telemetry-seams")]
pub type FoodPasses = Vec<FoodPassTiming>;

/// When a pass started; zero-sized without `telemetry-seams`.
#[derive(Clone, Copy)]
pub(super) struct PassStart(#[cfg(feature = "telemetry-seams")] Instant);

/// Collects the pass timings of one `grow` call.
pub(super) struct PassClock {
    #[cfg(feature = "telemetry-seams")]
    passes: FoodPasses,
}

impl PassClock {
    /// A clock for a catalog of `food_types` types: at most `4 + 4 x types`
    /// passes.
    #[cfg_attr(not(feature = "telemetry-seams"), allow(unused_variables))]
    pub(super) fn new(food_types: usize) -> Self {
        Self {
            #[cfg(feature = "telemetry-seams")]
            passes: Vec::with_capacity(4 + 4 * food_types),
        }
    }

    #[inline]
    pub(super) fn start() -> PassStart {
        PassStart(
            #[cfg(feature = "telemetry-seams")]
            Instant::now(),
        )
    }

    #[inline]
    #[cfg_attr(not(feature = "telemetry-seams"), allow(unused_variables))]
    pub(super) fn stop(
        &mut self,
        start: PassStart,
        pass: FoodPass,
        food_type: Option<OrdinaryFoodTypeId>,
    ) {
        #[cfg(feature = "telemetry-seams")]
        self.passes.push(FoodPassTiming {
            pass,
            food_type,
            started: start.0,
            elapsed: start.0.elapsed(),
        });
    }

    /// `summary` carrying the passes recorded so far.
    #[cfg_attr(not(feature = "telemetry-seams"), allow(unused_mut))]
    pub(super) fn finish(self, mut summary: FoodGrowthSummary) -> FoodGrowthSummary {
        #[cfg(feature = "telemetry-seams")]
        {
            summary.passes = self.passes;
        }
        summary
    }
}

#[cfg(all(test, feature = "telemetry-seams"))]
mod phase_seam_tests;
