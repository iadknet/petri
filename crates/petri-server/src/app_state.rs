use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};

use petri_core::{World, WorldConfig};

#[derive(Clone)]
pub struct AppState {
    pub world: Arc<RwLock<World>>,
    pub frames_tx: broadcast::Sender<Vec<u8>>,
}

impl AppState {
    pub fn new(seed: u64, config: WorldConfig) -> Self {
        let world = World::new(config, seed);
        let (frames_tx, _) = broadcast::channel(256);
        Self {
            world: Arc::new(RwLock::new(world)),
            frames_tx,
        }
    }

    pub fn new_for_tests() -> Self {
        let config = WorldConfig::default();
        Self::new(1, config)
    }
}
