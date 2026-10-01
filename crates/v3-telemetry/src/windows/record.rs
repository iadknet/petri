//! The compact `creature.window` log record (T23.F10).
//!
//! Every ended sample emits one, traced or not, so a run's window store keeps
//! its whole history; a job in the telemetry stack thins the stored records by
//! their `petri.keep` level. Floats are serialized as `f32`, so each keeps its
//! shortest round-trip form; a value that was not captured is omitted.

use serde::Serialize;
use v3_core::runtime::trace::domain::{BackendTrace, MeshHopTrace, TickTrace};
use v3_core::runtime::trace::recording::ActiveTrace;

use super::{action_label, died, truncation, Ending, SampleMeta};

/// The body's schema version.
const VERSION: u8 = 1;
/// The most hops one record keeps; past it they are cut and `truncated` set.
pub(crate) const MAX_RECORD_HOPS: usize = 2_048;

/// Sample `index`'s keep level: `2` when divisible by ten, else `1` when even,
/// else `0`. The window store keeps level 0 for the newest 300 indexes, level
/// 1 for the next 300, and level 2 beyond.
pub(crate) const fn keep_level(index: u64) -> u8 {
    if index.is_multiple_of(10) {
        2
    } else if index.is_multiple_of(2) {
        1
    } else {
        0
    }
}

#[derive(Serialize)]
struct Body<'a> {
    v: u8,
    window: u64,
    policy: &'static str,
    creature: String,
    ticks_requested: u32,
    lineage: u32,
    generation: u64,
    genome_size: u32,
    genome_hash: &'a str,
    age_start: u64,
    end: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    died: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    truncated: Option<Truncated>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    config_changed: bool,
    ticks: Vec<Tick<'a>>,
}

#[derive(Serialize)]
struct Truncated {
    reason: &'static str,
    tick: u64,
}

/// One hop: node, backend (`v` or `g`) and its summed absolute votes.
type Hop = (u32, &'static str, f32);

#[derive(Serialize)]
struct Tick<'a> {
    t: u64,
    e0: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    e1: Option<f32>,
    here: f32,
    food: &'a [f32; 8],
    #[serde(skip_serializing_if = "Option::is_none")]
    food_by_type: Option<&'a [[f32; 8]]>,
    barrier: &'a [f32; 8],
    occupied: &'a [f32; 8],
    sel: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    res: Option<Vec<&'static str>>,
    hops: Vec<Hop>,
}

fn hop(hop: &MeshHopTrace) -> Hop {
    let backend = match hop.backend_trace {
        BackendTrace::Vm(_) => "v",
        BackendTrace::Graph(_) => "g",
    };
    let votes = hop.vote_contribution.iter().map(|vote| vote.abs()).sum();
    (hop.node_id.0, backend, votes)
}

/// The tick's entry with at most `left` of its hops, taken from `left`;
/// returns whether any were cut.
fn tick<'a>(record: &'a TickTrace, left: &mut usize) -> (Tick<'a>, bool) {
    let outcome = record.outcome.as_ref();
    let sensed = &record.static_inputs;
    let kept = record.hops.len().min(*left);
    *left -= kept;
    let entry = Tick {
        t: record.tick_number + 1,
        e0: record.energy_before,
        e1: outcome
            .and_then(|o| o.after.as_ref())
            .map(|after| after.energy),
        here: sensed.food_here,
        food: &sensed.neighbor_food,
        food_by_type: outcome
            .filter(|o| o.uses_typed_local_food)
            .map(|o| &o.typed_local_food.neighbor_food_by_type[..]),
        barrier: &sensed.neighbor_barrier,
        occupied: &sensed.neighbor_occupied,
        sel: record.final_actions.iter().map(action_label).collect(),
        res: outcome.map(|o| {
            o.applied
                .iter()
                .map(|applied| applied.entry.result.as_key())
                .collect()
        }),
        hops: record.hops[..kept].iter().map(hop).collect(),
    };
    (entry, kept < record.hops.len())
}

/// The record body of `trace`, started as `meta` and ended as `ending`.
pub(crate) fn body(meta: &SampleMeta, trace: &ActiveTrace, ending: &Ending) -> String {
    let mut left = MAX_RECORD_HOPS;
    let mut cut = None;
    let ticks = trace
        .ticks
        .iter()
        .map(|record| {
            let (entry, was_cut) = tick(record, &mut left);
            if was_cut && cut.is_none() {
                cut = Some(entry.t);
            }
            entry
        })
        .collect();
    let start = &meta.start;
    let body = Body {
        v: VERSION,
        window: meta.index,
        policy: meta.policy.as_str(),
        creature: meta.creature_id.to_string(),
        ticks_requested: meta.ticks_requested,
        lineage: start.lineage,
        generation: start.generation,
        genome_size: start.genome_size,
        genome_hash: &start.genome_hash,
        age_start: start.age,
        end: ending.reason.as_str(),
        died: died(trace).map(|cause| cause.as_key()),
        truncated: truncation(trace)
            .or(cut.map(|tick| ("hops", tick)))
            .map(|(reason, tick)| Truncated { reason, tick }),
        config_changed: meta.config_changed,
        ticks,
    };
    serde_json::to_string(&body).expect("window records serialize")
}
