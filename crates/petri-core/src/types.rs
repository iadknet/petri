use std::collections::HashMap;

use petri_graph::{ActionOutputs, ComputationGraph, ControllerPalette, SensorInputs};
use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

use crate::config::WorldConfig;

new_key_type! {
    pub struct CreatureId;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CreatureEventKind {
    AteFood,
    Reproduced,
    Starved,
    Moved,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CreatureEvent {
    pub kind: CreatureEventKind,
    pub tick: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatureSnapshot {
    pub id: u64,
    pub lineage_id: u64,
    pub parent_id: Option<u64>,
    pub x: u32,
    pub y: u32,
    pub energy: f32,
    pub age: u64,
    pub generation: u32,
    pub node_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldFrame {
    pub tick: u64,
    pub width: u32,
    pub height: u32,
    pub food: Vec<u8>,
    pub barrier_bits: Vec<u8>,
    pub creatures: Vec<CreatureSnapshot>,
    pub population: usize,
    pub average_energy: f32,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct WorldDiagnostics {
    pub moves: u64,
    pub eats: u64,
    pub reproductions: u64,
    pub deaths: u64,
    #[serde(default)]
    pub illegal_actions: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum InventoryItem {
    Food(f32),
    Barrier,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IllegalActionKind {
    Move,
    Eat,
    Reproduce,
    InventoryPickup,
    InventoryPut,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IllegalActionReason {
    MoveBlocked,
    MoveOutOfBounds,
    EatNoFood,
    ReproduceLowEnergy,
    ReproduceNoSpace,
    ReproduceMaxCreatures,
    SlotMissing,
    SlotFull,
    SlotEmpty,
    TargetOutOfBounds,
    TargetOccupied,
    TargetHasBarrier,
    NoPickupableMaterial,
    FoodOverflow,
    TargetIncompatible,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IllegalActionAttempt {
    pub action: IllegalActionKind,
    pub reason: IllegalActionReason,
    pub tick: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatureStateSnapshot {
    pub id: u64,
    pub lineage_id: u64,
    pub parent_id: Option<u64>,
    pub x: u32,
    pub y: u32,
    pub energy: f32,
    pub age: u64,
    pub generation: u32,
    pub controller: ComputationGraph,
    #[serde(default)]
    pub memory_register: Vec<bool>,
    #[serde(default)]
    pub last_move_blocked: bool,
    #[serde(default)]
    pub last_inputs: SensorInputs,
    #[serde(default)]
    pub last_outputs: ActionOutputs,
    #[serde(default)]
    pub slot_capacity: u8,
    #[serde(default)]
    pub slots: Vec<Option<InventoryItem>>,
    #[serde(default)]
    pub illegal_attempts: Vec<IllegalActionAttempt>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreatureDetail {
    pub id: u64,
    pub lineage_id: u64,
    pub parent_id: Option<u64>,
    pub x: u32,
    pub y: u32,
    pub energy: f32,
    pub age: u64,
    pub generation: u32,
    pub node_count: u32,
    pub last_move_blocked: bool,
    pub last_inputs: SensorInputs,
    pub last_outputs: ActionOutputs,
    pub events: Vec<CreatureEvent>,
    #[serde(default)]
    pub slot_capacity: u8,
    #[serde(default)]
    pub slots: Vec<Option<InventoryItem>>,
    #[serde(default)]
    pub illegal_attempts: Vec<IllegalActionAttempt>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub tick: u64,
    pub config: WorldConfig,
    pub palette: ControllerPalette,
    pub cells_food: Vec<f32>,
    #[serde(default)]
    pub cells_barrier: Vec<bool>,
    pub creatures: Vec<CreatureStateSnapshot>,
    pub diagnostics: WorldDiagnostics,
    pub lineage_tree: HashMap<u64, Vec<u64>>,
    pub next_lineage_id: u64,
}
