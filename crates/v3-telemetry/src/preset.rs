//! Named sampling presets (T21.F05). `PETRI_TELEMETRY_PRESET` names the base
//! values of the five sampling settings; a setting's own variable, when set,
//! overrides that setting only. Read only when telemetry resolves on.

use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use crate::{
    resolve_metrics_interval, resolve_tick_traces, Switch, WindowSettings, CREATURE_WINDOWS_ENV,
    DEFAULT_METRICS_INTERVAL, METRICS_INTERVAL_ENV, TICK_TRACES_ENV, WINDOW_INTERVAL_ENV,
    WINDOW_TICKS_ENV,
};

/// The variable that names the sampling preset.
pub const PRESET_ENV: &str = "PETRI_TELEMETRY_PRESET";

/// A named set of base sampling settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Preset {
    /// Snapshots only.
    Minimal,
    /// Snapshots and tick traces.
    Phases,
    /// Snapshots, tick traces and creature windows: the F04 defaults.
    #[default]
    Standard,
    /// The opt-in detail level: faster cadences and longer windows.
    Dense,
}

/// The five sampling settings a run exports with, and the preset they start
/// from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    pub preset: Preset,
    pub metrics_interval: Duration,
    pub tick_traces: Switch,
    pub windows: WindowSettings,
}

impl Preset {
    /// Every preset, in increasing detail.
    pub const ALL: [Self; 4] = [Self::Minimal, Self::Phases, Self::Standard, Self::Dense];

    /// The preset's `PETRI_TELEMETRY_PRESET` value.
    pub fn name(self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Phases => "phases",
            Self::Standard => "standard",
            Self::Dense => "dense",
        }
    }

    /// The preset's base values.
    pub fn settings(self) -> Settings {
        let standard = WindowSettings::default();
        let windows_off = WindowSettings {
            switch: Switch::Off,
            ..standard
        };
        let (metrics_interval, tick_traces, windows) = match self {
            Self::Minimal => (DEFAULT_METRICS_INTERVAL, Switch::Off, windows_off),
            Self::Phases => (DEFAULT_METRICS_INTERVAL, Switch::On, windows_off),
            Self::Standard => (DEFAULT_METRICS_INTERVAL, Switch::On, standard),
            Self::Dense => (
                Duration::from_millis(250),
                Switch::On,
                WindowSettings {
                    switch: Switch::On,
                    ticks: 16,
                    interval: Duration::from_secs(2),
                },
            ),
        };
        Settings {
            preset: self,
            metrics_interval,
            tick_traces,
            windows,
        }
    }
}

impl FromStr for Preset {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|preset| preset.name() == value)
            .ok_or_else(|| {
                format!(
                    "invalid {PRESET_ENV}: expected minimal, phases, standard or dense, \
                     found `{value}`"
                )
            })
    }
}

impl fmt::Display for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl Settings {
    /// Resolves the preset and the five settings from `lookup` (a variable's
    /// value by name): an unset or empty variable takes the preset's value,
    /// and any invalid value refuses.
    pub fn resolve(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let value = |name| lookup(name).filter(|value| !value.trim().is_empty());
        let preset = match value(PRESET_ENV) {
            Some(name) => name.trim().parse()?,
            None => Preset::default(),
        };
        let base = preset.settings();
        let metrics_interval = match value(METRICS_INTERVAL_ENV) {
            Some(set) => resolve_metrics_interval(Some(&set))?,
            None => base.metrics_interval,
        };
        let tick_traces = match value(TICK_TRACES_ENV) {
            Some(set) => resolve_tick_traces(Some(&set))?,
            None => base.tick_traces,
        };
        let windows = WindowSettings::resolve_over(
            base.windows,
            value(CREATURE_WINDOWS_ENV).as_deref(),
            value(WINDOW_TICKS_ENV).as_deref(),
            value(WINDOW_INTERVAL_ENV).as_deref(),
        )?;
        Ok(Self {
            preset,
            metrics_interval,
            tick_traces,
            windows,
        })
    }
}

#[cfg(test)]
mod tests;
