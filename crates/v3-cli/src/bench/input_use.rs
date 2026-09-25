//! The per-world `input_use` block (T20.F01, `input-use-v1`): the input-use
//! funnel per T11.F26 cohort, family and channel, projected from
//! `v3_core::neighborhood::input_use`.
//!
//! Each row is one space-separated string, `ROW_FORMAT`, so the committed
//! summary stays inside its storage budget: the family, the channel (`*`
//! marks an `ActionQueue` channel past the draw width), then the parents at
//! each stage in `ROW_FIELDS` order, `-` for a stage the family does not have
//! (shared memory has no declaration or causal stage).

use serde::{Deserialize, Serialize};
use v3_core::neighborhood::input_use::{self as funnel, CohortUse, Row};
use v3_core::neighborhood::mutation_effects::{self as effects, contexts};
use v3_core::neighborhood::BATTERY_VERSION;

use super::mutation_effects::RecordedGroup;
use super::schema::Indicator;
use crate::UNDEFINED;

#[cfg(test)]
mod tests;

pub(super) fn undefined_input_use() -> Indicator<InputUse> {
    Indicator::Undefined(UNDEFINED.to_string())
}

/// The names of a row's count positions, in order.
pub const ROW_FIELDS: [&str; 9] = [
    "declared",
    "connected",
    "executed",
    "causal",
    "causal_original",
    "executed_outside_live",
    "causal_outside_live",
    "retention_pairs",
    "retained_causal_pairs",
];

/// The row string's layout.
pub const ROW_FORMAT: &str =
    "family channel[*] declared connected executed causal causal_original \
                              executed_outside_live causal_outside_live retention_pairs \
                              retained_causal_pairs; * marks an ActionQueue channel past the \
                              draw width 12, - a stage the family does not have";

/// One decoded row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowBlock {
    pub family: String,
    pub channel: u16,
    /// An `ActionQueue` channel at or past the mutation draw width 12.
    pub beyond_draw_width: bool,
    /// Parents at each stage, in `ROW_FIELDS` order.
    pub counts: Vec<Option<u32>>,
}

impl RowBlock {
    fn of(row: &Row) -> Self {
        Self {
            family: row.channel.family.label(),
            channel: row.channel.channel,
            beyond_draw_width: row.channel.beyond_draw_width(),
            counts: vec![
                row.declared,
                Some(row.connected),
                Some(row.executed),
                row.causal,
                row.causal_original,
                Some(row.executed_outside_live),
                row.causal_outside_live,
                row.retention_pairs,
                row.retained_causal_pairs,
            ],
        }
    }

    /// The row string.
    #[must_use]
    pub fn encode(&self) -> String {
        let mut text = format!(
            "{} {}{}",
            self.family,
            self.channel,
            if self.beyond_draw_width { "*" } else { "" }
        );
        for count in &self.counts {
            text.push(' ');
            match count {
                Some(count) => text.push_str(&count.to_string()),
                None => text.push('-'),
            }
        }
        text
    }

    /// Decode a row string; `None` when it is not one.
    #[must_use]
    pub fn decode(text: &str) -> Option<Self> {
        let mut fields = text.split(' ');
        let family = fields.next()?.to_string();
        let channel = fields.next()?;
        let (channel, beyond_draw_width) = match channel.strip_suffix('*') {
            Some(channel) => (channel, true),
            None => (channel, false),
        };
        let counts = fields
            .map(|field| match field {
                "-" => Some(None),
                count => count.parse().ok().map(Some),
            })
            .collect::<Option<Vec<_>>>()?;
        let channel = channel.parse().ok()?;
        (counts.len() == ROW_FIELDS.len()).then_some(Self {
            family,
            channel,
            beyond_draw_width,
            counts,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyCount {
    pub family: String,
    pub consumers: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CohortBlock {
    pub cohort: String,
    pub parents_requested: u32,
    pub parents_evaluated: u32,
    /// (parent, channel) pairs causal without an executed read; expected 0.
    pub consistency_violations: u32,
    /// Decision-compound consumers reading at or past the width (0.0).
    pub out_of_width_consumers: Vec<FamilyCount>,
    pub retention_parents: u32,
    pub retention_children_requested: u32,
    pub retention_children_sampled: u32,
    /// Retained causal pairs over retention pairs, pooled over rows;
    /// undefined without a retention pair.
    pub retained_share: Indicator<RetainedShare>,
    /// Row strings in `ROW_FORMAT`, family then channel order.
    pub rows: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetainedShare {
    pub retention_pairs: u64,
    pub retained_causal_pairs: u64,
    /// Six-decimal fraction.
    pub share: String,
}

fn cohort_block(reading: &CohortUse) -> CohortBlock {
    let (pairs, retained) = reading
        .rows
        .iter()
        .fold((0u64, 0u64), |(pairs, kept), row| {
            (
                pairs + u64::from(row.retention_pairs.unwrap_or(0)),
                kept + u64::from(row.retained_causal_pairs.unwrap_or(0)),
            )
        });
    CohortBlock {
        cohort: reading.cohort.as_key().to_string(),
        parents_requested: reading.parents_requested,
        parents_evaluated: reading.parents_evaluated,
        consistency_violations: reading.consistency_violations,
        out_of_width_consumers: reading
            .out_of_width_consumers
            .iter()
            .map(|(family, &consumers)| FamilyCount {
                family: family.label(),
                consumers,
            })
            .collect(),
        retention_parents: reading.retention_parents,
        retention_children_requested: reading.retention_children_requested,
        retention_children_sampled: reading.retention_children_sampled,
        retained_share: if pairs == 0 {
            Indicator::Undefined("no retention pair".to_string())
        } else {
            Indicator::Defined(RetainedShare {
                retention_pairs: pairs,
                retained_causal_pairs: retained,
                share: crate::six(retained as f64 / pairs as f64),
            })
        },
        rows: reading
            .rows
            .iter()
            .map(|row| RowBlock::of(row).encode())
            .collect(),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scenes {
    pub original: String,
    pub original_executions: u32,
    pub extended_executions: u32,
    pub recorded: Indicator<RecordedGroup>,
    pub authored_contexts: u32,
    pub sequences: u32,
    pub sequence_ticks: u32,
    pub sequence_source: String,
}

/// One world's `input_use` block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputUse {
    pub version: String,
    pub stage_rule: String,
    pub ablation_rule: String,
    pub retention_rule: String,
    pub row_format: String,
    pub scenes: Scenes,
    pub cohorts: Vec<Indicator<CohortBlock>>,
}

#[must_use]
pub fn project(reading: &funnel::Reading) -> InputUse {
    InputUse {
        version: funnel::VERSION.to_string(),
        stage_rule: "declared: a reachable node's input_refs holds the family (UpstreamSlot: the \
                     slot); connected: a consumer on the sensor census's live walk addresses the \
                     channel; executed: a read of the channel is resolved in some scene; causal: \
                     ablating the channel changes the committed actions in some scene; counts are \
                     parents"
            .to_string(),
        ablation_rule: "one channel at a time, every consumer's reference index moved out of \
                        range so every read returns 0.0; native semantics and charges otherwise"
            .to_string(),
        retention_rule: format!(
            "per parent with a causal channel, its first {} applied, not genome-identical \
             {} proposals in proposal order (seed {} + {} * cohort + {} * (parent_index + 1) + \
             proposal_index); a pair is retained when the channel is still causal in the child \
             on the same scenes",
            reading.retention_children,
            effects::VERSION,
            effects::PROPOSAL_SEED_BASE,
            effects::COHORT_SEED_MULTIPLIER,
            effects::PARENT_SEED_MULTIPLIER,
        ),
        row_format: ROW_FORMAT.to_string(),
        scenes: Scenes {
            original: BATTERY_VERSION.to_string(),
            original_executions: reading.original_executions,
            extended_executions: reading.extended_executions,
            recorded: match &reading.recorded_contexts {
                Ok(actual) => Indicator::Defined(RecordedGroup {
                    requested: reading.recorded_requested,
                    actual: *actual,
                }),
                Err(reason) => Indicator::Undefined(reason.clone()),
            },
            authored_contexts: contexts::AUTHORED_CONTEXTS as u32,
            sequences: contexts::SEQUENCES as u32,
            sequence_ticks: contexts::SEQUENCE_TICKS as u32,
            sequence_source: reading.sequence_source.to_string(),
        },
        cohorts: vec![
            Indicator::Defined(cohort_block(&reading.founder)),
            Indicator::Defined(cohort_block(&reading.drift)),
            match &reading.selected {
                Ok(selected) => Indicator::Defined(cohort_block(selected)),
                Err(reason) => Indicator::Undefined(reason.clone()),
            },
        ],
    }
}

/// One world's block over the T11.F26 cohorts.
pub(super) fn observe_world(
    seed: u64,
    sim: &v3_core::simulation::Simulation,
    observation: super::mutation_effects::WorldObservation<'_>,
) -> InputUse {
    let parents = super::mutation_effects::selected_cohort(
        seed,
        sim,
        observation.read_sample,
        observation.sizes,
    );
    let founder =
        v3_core::creature::founder::founder_genome(v3_core::config::FounderProfile::V3Alpha1);
    let context = v3_core::neighborhood::EvalContext::from_config(observation.config);
    let reading = funnel::observe(
        effects::WorldInputs {
            founder: &founder,
            drift: &observation.drift.parents,
            selected: super::mutation_effects::selected_inputs(&parents),
            sim,
            world_seed: seed,
        },
        observation.battery,
        &observation.config.mutation,
        &context,
        observation.sizes,
        funnel::RETENTION_CHILDREN,
    );
    project(&reading)
}
