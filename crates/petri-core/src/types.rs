use serde::{Deserialize, Serialize};
use slotmap::new_key_type;

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
    pub x: u32,
    pub y: u32,
    pub energy: f32,
    pub age: u64,
    pub generation: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldFrame {
    pub tick: u64,
    pub width: u32,
    pub height: u32,
    pub food: Vec<f32>,
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
}
