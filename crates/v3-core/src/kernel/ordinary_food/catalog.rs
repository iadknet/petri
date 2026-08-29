use crate::config::{FoodConfig, FoodTypeConfig, OrdinaryFoodTypeId};

#[derive(Debug, Clone, PartialEq)]
pub struct OrdinaryFoodTypeEntry {
    pub id: OrdinaryFoodTypeId,
    pub config: FoodTypeConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrdinaryFoodCatalog {
    types: Vec<OrdinaryFoodTypeEntry>,
}

impl OrdinaryFoodCatalog {
    #[must_use]
    pub fn new(config: &FoodConfig) -> Self {
        let types = if config.types.is_empty() {
            vec![FoodTypeConfig::default()]
        } else {
            config.types.clone()
        };

        Self {
            types: types
                .into_iter()
                .enumerate()
                .map(|(index, config)| OrdinaryFoodTypeEntry {
                    id: OrdinaryFoodTypeId::new(index as u16),
                    config,
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.types.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    #[must_use]
    pub fn is_valid(&self, type_idx: OrdinaryFoodTypeId) -> bool {
        usize::from(type_idx.get()) < self.types.len()
    }

    #[must_use]
    pub fn primary(&self) -> &OrdinaryFoodTypeEntry {
        &self.types[0]
    }

    #[must_use]
    pub fn get(&self, type_idx: OrdinaryFoodTypeId) -> Option<&OrdinaryFoodTypeEntry> {
        self.types.get(usize::from(type_idx.get()))
    }

    #[must_use]
    pub fn entries(&self) -> &[OrdinaryFoodTypeEntry] {
        &self.types
    }
}
