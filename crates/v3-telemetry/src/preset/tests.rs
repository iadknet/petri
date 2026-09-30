use std::collections::BTreeMap;
use std::time::Duration;

use proptest::prelude::*;

use super::*;
use crate::{
    Switch, WindowSettings, CREATURE_WINDOWS_ENV, DEFAULT_METRICS_INTERVAL, METRICS_INTERVAL_ENV,
    TICK_TRACES_ENV, WINDOW_INTERVAL_ENV, WINDOW_TICKS_ENV,
};

fn lookup(vars: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
    let vars: BTreeMap<String, String> = vars
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    move |name| vars.get(name).cloned()
}

fn settings(
    preset: Preset,
    interval_ms: u64,
    tick_traces: Switch,
    windows: Switch,
    ticks: u32,
    window_ms: u64,
) -> Settings {
    Settings {
        preset,
        metrics_interval: Duration::from_millis(interval_ms),
        tick_traces,
        windows: WindowSettings {
            switch: windows,
            ticks,
            interval: Duration::from_millis(window_ms),
        },
    }
}

#[test]
fn each_preset_names_the_spec_table_values() {
    use Switch::{Off, On};
    let table = [
        (
            "minimal",
            settings(Preset::Minimal, 1000, Off, Off, 8, 10_000),
        ),
        ("phases", settings(Preset::Phases, 1000, On, Off, 8, 10_000)),
        (
            "standard",
            settings(Preset::Standard, 1000, On, On, 8, 10_000),
        ),
        ("dense", settings(Preset::Dense, 250, On, On, 16, 2_000)),
    ];
    for (name, expected) in table {
        let preset: Preset = name.parse().unwrap();
        assert_eq!(preset.to_string(), name);
        assert_eq!(preset.settings(), expected, "{name}");
        assert_eq!(
            Settings::resolve(lookup(&[(PRESET_ENV, name)])),
            Ok(expected),
            "{name}"
        );
    }
    assert_eq!(Preset::ALL.len(), table.len());
}

#[test]
fn the_default_preset_is_standard_and_standard_is_the_f04_defaults() {
    let standard = Preset::Standard.settings();
    assert_eq!(Preset::default(), Preset::Standard);
    assert_eq!(standard.metrics_interval, DEFAULT_METRICS_INTERVAL);
    assert_eq!(standard.tick_traces, Switch::On);
    assert_eq!(standard.windows, WindowSettings::default());
    for unset in [&[][..], &[(PRESET_ENV, "")], &[(PRESET_ENV, "  ")]] {
        assert_eq!(Settings::resolve(lookup(unset)), Ok(standard));
    }
    assert_eq!(
        Settings::resolve(lookup(&[(PRESET_ENV, " dense ")])),
        Ok(Preset::Dense.settings())
    );
}

#[test]
fn anything_but_the_four_names_refuses() {
    for invalid in ["Standard", "full", "none", "0", "standard,dense"] {
        let error = Settings::resolve(lookup(&[(PRESET_ENV, invalid)])).unwrap_err();
        assert!(error.contains(PRESET_ENV), "{error}");
        assert!(error.contains(invalid), "{error}");
    }
}

#[test]
fn an_invalid_setting_still_refuses_under_every_preset() {
    for preset in Preset::ALL {
        let name = preset.to_string();
        for (var, value) in [
            (METRICS_INTERVAL_ENV, "9"),
            (TICK_TRACES_ENV, "yes"),
            (CREATURE_WINDOWS_ENV, "1"),
            (WINDOW_TICKS_ENV, "65"),
            (WINDOW_INTERVAL_ENV, "3600001"),
        ] {
            let error =
                Settings::resolve(lookup(&[(PRESET_ENV, &name), (var, value)])).unwrap_err();
            assert!(error.contains(var), "{error}");
        }
    }
}

fn switch() -> impl Strategy<Value = Switch> {
    prop_oneof![Just(Switch::On), Just(Switch::Off)]
}

proptest! {
    /// An explicitly set variable replaces its setting's preset value and
    /// leaves the other four at the preset's.
    #[test]
    fn an_explicit_variable_overrides_its_setting_only(
        preset in proptest::sample::select(Preset::ALL.to_vec()),
        interval in proptest::option::of(10_u64..=3_600_000),
        traces in proptest::option::of(switch()),
        windows in proptest::option::of(switch()),
        ticks in proptest::option::of(1_u32..=64),
        window_ms in proptest::option::of(10_u64..=3_600_000),
    ) {
        let name = preset.to_string();
        let (interval_s, traces_s, windows_s, ticks_s, window_s) = (
            interval.map(|v| v.to_string()),
            traces.map(|v| v.to_string()),
            windows.map(|v| v.to_string()),
            ticks.map(|v| v.to_string()),
            window_ms.map(|v| v.to_string()),
        );
        let mut vars = vec![(PRESET_ENV, name.as_str())];
        for (var, value) in [
            (METRICS_INTERVAL_ENV, &interval_s),
            (TICK_TRACES_ENV, &traces_s),
            (CREATURE_WINDOWS_ENV, &windows_s),
            (WINDOW_TICKS_ENV, &ticks_s),
            (WINDOW_INTERVAL_ENV, &window_s),
        ] {
            if let Some(value) = value {
                vars.push((var, value.as_str()));
            }
        }
        let base = preset.settings();
        let expected = Settings {
            preset,
            metrics_interval: interval.map_or(base.metrics_interval, Duration::from_millis),
            tick_traces: traces.unwrap_or(base.tick_traces),
            windows: WindowSettings {
                switch: windows.unwrap_or(base.windows.switch),
                ticks: ticks.unwrap_or(base.windows.ticks),
                interval: window_ms.map_or(base.windows.interval, Duration::from_millis),
            },
        };
        prop_assert_eq!(Settings::resolve(lookup(&vars)), Ok(expected));
    }
}
