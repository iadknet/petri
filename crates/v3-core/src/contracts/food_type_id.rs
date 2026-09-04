/// Stable identifier for a configured ordinary-food type in a run.
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(transparent)]
pub struct OrdinaryFoodTypeId(pub u16);

impl OrdinaryFoodTypeId {
    #[inline]
    #[must_use]
    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    #[inline]
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}
