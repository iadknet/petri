//! Server-owned wire forms of the genome's vocabulary (T30.F01): the input
//! reference, the VM instruction, the vote sink and the graph's node, sink
//! and source kinds, shared by the brain view and the sampler. Each keeps
//! the JSON the core type had, except the input reference, which is one flat
//! record; each converts from the core type by an exhaustive match.

use serde::Serialize;
use v3_core::contracts::votes as core_vote;
use v3_core::contracts::OutcomeChannel as CoreOutcomeChannel;
use v3_core::contracts::{
    Direction, DynamicIntrospectionKey, InputReference, NodeId, OrdinaryFoodTypeId,
    StaticIntrospectionKey, WorldAction as CoreWorldAction, WorldInputKey,
};
use v3_core::genome::cgp as core_cgp;
use v3_core::genome::mesh_annotations as core_mesh;
use v3_core::genome::{HebbianRule as CoreHebbianRule, VmInstruction as CoreVmInstruction};

// ── Input references ────────────────────────────────────────────────────────

/// Which kind of input an [`InputRef`] reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "snake_case")]
pub enum InputFamily {
    World,
    Static,
    Dynamic,
    Upstream,
    ActionQueue,
    Decision,
}

/// An input key by its variant name, as the catalog fixtures key it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum InputKey {
    FoodHere,
    NeighborFoodRing,
    NeighborBarrierRing,
    NeighborOccupiedRing,
    AreaFoodSummary,
    AreaBarrierSummary,
    AreaOccupancySummary,
    NearbyCreature,
    AgeTicks,
    OwnRed,
    OwnGreen,
    OwnBlue,
    EnergyCurrent,
    EnergyConsumedThisTick,
    HopsThisTick,
    UpstreamSlot,
    UpstreamBus,
    ActionQueue,
    ActionVotes,
    PreviousPassVotes,
    CommitCounts,
    PreviousOutcome,
}

/// One input reference (T30.F01 invariant 6): its family, its key, the food
/// type of a typed food sensor and the slot of an upstream reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct InputRef {
    pub family: InputFamily,
    pub key: InputKey,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub food_type: Option<OrdinaryFoodTypeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub slot: Option<usize>,
}

impl InputRef {
    const fn of(family: InputFamily, key: InputKey) -> Self {
        Self {
            family,
            key,
            food_type: None,
            slot: None,
        }
    }

    const fn food(key: InputKey, food_type: OrdinaryFoodTypeId) -> Self {
        Self {
            food_type: Some(food_type),
            ..Self::of(InputFamily::World, key)
        }
    }
}

impl From<&InputReference> for InputRef {
    fn from(reference: &InputReference) -> Self {
        match reference {
            InputReference::World(key) => match *key {
                WorldInputKey::FoodHere { type_idx } => Self::food(InputKey::FoodHere, type_idx),
                WorldInputKey::NeighborFoodRing { type_idx } => {
                    Self::food(InputKey::NeighborFoodRing, type_idx)
                }
                WorldInputKey::AreaFoodSummary { type_idx } => {
                    Self::food(InputKey::AreaFoodSummary, type_idx)
                }
                WorldInputKey::NeighborBarrierRing => {
                    Self::of(InputFamily::World, InputKey::NeighborBarrierRing)
                }
                WorldInputKey::NeighborOccupiedRing => {
                    Self::of(InputFamily::World, InputKey::NeighborOccupiedRing)
                }
                WorldInputKey::AreaBarrierSummary => {
                    Self::of(InputFamily::World, InputKey::AreaBarrierSummary)
                }
                WorldInputKey::AreaOccupancySummary => {
                    Self::of(InputFamily::World, InputKey::AreaOccupancySummary)
                }
                WorldInputKey::NearbyCreature => {
                    Self::of(InputFamily::World, InputKey::NearbyCreature)
                }
            },
            InputReference::StaticIntrospection(key) => Self::of(
                InputFamily::Static,
                match key {
                    StaticIntrospectionKey::AgeTicks => InputKey::AgeTicks,
                    StaticIntrospectionKey::OwnRed => InputKey::OwnRed,
                    StaticIntrospectionKey::OwnGreen => InputKey::OwnGreen,
                    StaticIntrospectionKey::OwnBlue => InputKey::OwnBlue,
                },
            ),
            InputReference::DynamicIntrospection(key) => Self::of(
                InputFamily::Dynamic,
                match key {
                    DynamicIntrospectionKey::EnergyCurrent => InputKey::EnergyCurrent,
                    DynamicIntrospectionKey::EnergyConsumedThisTick => {
                        InputKey::EnergyConsumedThisTick
                    }
                    DynamicIntrospectionKey::HopsThisTick => InputKey::HopsThisTick,
                },
            ),
            InputReference::UpstreamSlot(slot) => Self {
                slot: Some(*slot),
                ..Self::of(InputFamily::Upstream, InputKey::UpstreamSlot)
            },
            InputReference::UpstreamBus => Self::of(InputFamily::Upstream, InputKey::UpstreamBus),
            InputReference::ActionQueue => {
                Self::of(InputFamily::ActionQueue, InputKey::ActionQueue)
            }
            InputReference::ActionVotes => Self::of(InputFamily::Decision, InputKey::ActionVotes),
            InputReference::PreviousPassVotes => {
                Self::of(InputFamily::Decision, InputKey::PreviousPassVotes)
            }
            InputReference::CommitCounts => Self::of(InputFamily::Decision, InputKey::CommitCounts),
            InputReference::PreviousOutcome => {
                Self::of(InputFamily::Decision, InputKey::PreviousOutcome)
            }
        }
    }
}

// ── Votes ───────────────────────────────────────────────────────────────────

/// A directed vote kind (a bundle's kind, a directed sink's kind).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
pub enum DirectedVoteKind {
    Move,
    Reproduce,
    StealEnergy,
    GiveEnergy,
}

impl From<core_vote::DirectedVoteKind> for DirectedVoteKind {
    fn from(kind: core_vote::DirectedVoteKind) -> Self {
        match kind {
            core_vote::DirectedVoteKind::Move => Self::Move,
            core_vote::DirectedVoteKind::Reproduce => Self::Reproduce,
            core_vote::DirectedVoteKind::StealEnergy => Self::StealEnergy,
            core_vote::DirectedVoteKind::GiveEnergy => Self::GiveEnergy,
        }
    }
}

/// One vote sink of the catalog; the number is a `Direction::ALL` index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
pub enum VoteSink {
    Eat,
    Directed(DirectedVoteKind, u8),
    Drive(DirectedVoteKind),
    Terminate,
    Decide,
}

impl From<core_vote::VoteSink> for VoteSink {
    fn from(sink: core_vote::VoteSink) -> Self {
        match sink {
            core_vote::VoteSink::Eat => Self::Eat,
            core_vote::VoteSink::Directed(kind, d) => Self::Directed(kind.into(), d),
            core_vote::VoteSink::Drive(kind) => Self::Drive(kind.into()),
            core_vote::VoteSink::Terminate => Self::Terminate,
            core_vote::VoteSink::Decide => Self::Decide,
        }
    }
}

/// One action-parameter field the commit decoder reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
pub enum ActionParamField {
    EatFoodType,
    EatLeaveShare,
    ReproduceTransferFraction,
    StealEffort,
    GiveEffort,
}

impl From<core_vote::ActionParamField> for ActionParamField {
    fn from(field: core_vote::ActionParamField) -> Self {
        match field {
            core_vote::ActionParamField::EatFoodType => Self::EatFoodType,
            core_vote::ActionParamField::EatLeaveShare => Self::EatLeaveShare,
            core_vote::ActionParamField::ReproduceTransferFraction => {
                Self::ReproduceTransferFraction
            }
            core_vote::ActionParamField::StealEffort => Self::StealEffort,
            core_vote::ActionParamField::GiveEffort => Self::GiveEffort,
        }
    }
}

/// The action a creature commits.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
pub enum WorldAction {
    NoOp,
    Eat {
        type_idx: OrdinaryFoodTypeId,
        leave: f32,
    },
    Move(Direction),
    Reproduce {
        direction: Direction,
        energy_transfer_fraction: f32,
    },
    StealEnergy {
        direction: Direction,
        effort: f32,
    },
    GiveEnergy {
        direction: Direction,
        effort: f32,
    },
}

impl From<CoreWorldAction> for WorldAction {
    fn from(action: CoreWorldAction) -> Self {
        match action {
            CoreWorldAction::NoOp => Self::NoOp,
            CoreWorldAction::Eat { type_idx, leave } => Self::Eat { type_idx, leave },
            CoreWorldAction::Move(direction) => Self::Move(direction),
            CoreWorldAction::Reproduce {
                direction,
                energy_transfer_fraction,
            } => Self::Reproduce {
                direction,
                energy_transfer_fraction,
            },
            CoreWorldAction::StealEnergy { direction, effort } => {
                Self::StealEnergy { direction, effort }
            }
            CoreWorldAction::GiveEnergy { direction, effort } => {
                Self::GiveEnergy { direction, effort }
            }
        }
    }
}

// ── Graph vocabulary ────────────────────────────────────────────────────────

/// A compute node's kind.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
pub enum ComputeNodeKind {
    Add,
    Multiply,
    Negate,
    Abs,
    Min,
    Max,
    WeightedSum,
    Sigmoid,
    Tanh,
    Relu,
    Clamp01,
    Threshold(f32),
    GreaterThan,
    Select,
    DecayIntegrator(f32),
    Momentum(f32),
    Oscillator(f32),
    AdaptiveGain,
    Constant(f32),
}

impl From<core_cgp::ComputeNodeKind> for ComputeNodeKind {
    fn from(kind: core_cgp::ComputeNodeKind) -> Self {
        use core_cgp::ComputeNodeKind as K;
        match kind {
            K::Add => Self::Add,
            K::Multiply => Self::Multiply,
            K::Negate => Self::Negate,
            K::Abs => Self::Abs,
            K::Min => Self::Min,
            K::Max => Self::Max,
            K::WeightedSum => Self::WeightedSum,
            K::Sigmoid => Self::Sigmoid,
            K::Tanh => Self::Tanh,
            K::Relu => Self::Relu,
            K::Clamp01 => Self::Clamp01,
            K::Threshold(v) => Self::Threshold(v),
            K::GreaterThan => Self::GreaterThan,
            K::Select => Self::Select,
            K::DecayIntegrator(v) => Self::DecayIntegrator(v),
            K::Momentum(v) => Self::Momentum(v),
            K::Oscillator(v) => Self::Oscillator(v),
            K::AdaptiveGain => Self::AdaptiveGain,
            K::Constant(v) => Self::Constant(v),
        }
    }
}

/// An output sink's kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
pub enum OutputSinkKind {
    CustomOutput(u8),
    RouterGate(u8),
    WriteSlot(u8),
    ClearSlot(u8),
    ActionVote(VoteSink),
    ActionParam(ActionParamField),
}

impl From<core_cgp::OutputSinkKind> for OutputSinkKind {
    fn from(kind: core_cgp::OutputSinkKind) -> Self {
        use core_cgp::OutputSinkKind as K;
        match kind {
            K::CustomOutput(slot) => Self::CustomOutput(slot),
            K::RouterGate(slot) => Self::RouterGate(slot),
            K::WriteSlot(slot) => Self::WriteSlot(slot),
            K::ClearSlot(slot) => Self::ClearSlot(slot),
            K::ActionVote(sink) => Self::ActionVote(sink.into()),
            K::ActionParam(field) => Self::ActionParam(field.into()),
        }
    }
}

/// Where a graph edge reads from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
pub enum GraphSource {
    /// `input_refs[ref_idx]`, sub-value `sub_idx`.
    InputLeaf { ref_idx: u16, sub_idx: u16 },
    /// A shared memory slot, this tick's or the previous tick's.
    SharedMemory { slot: u8, previous: bool },
    /// The output of a compute node.
    ComputeNode(u16),
}

impl From<core_cgp::GraphSource> for GraphSource {
    fn from(source: core_cgp::GraphSource) -> Self {
        match source {
            core_cgp::GraphSource::InputLeaf { ref_idx, sub_idx } => {
                Self::InputLeaf { ref_idx, sub_idx }
            }
            core_cgp::GraphSource::SharedMemory { slot, previous } => {
                Self::SharedMemory { slot, previous }
            }
            core_cgp::GraphSource::ComputeNode(index) => Self::ComputeNode(index),
        }
    }
}

/// A Hebbian rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
pub enum HebbianRule {
    Classic,
    Oja,
    AntiHebb,
    Covariance,
}

impl From<CoreHebbianRule> for HebbianRule {
    fn from(rule: CoreHebbianRule) -> Self {
        match rule {
            CoreHebbianRule::Classic => Self::Classic,
            CoreHebbianRule::Oja => Self::Oja,
            CoreHebbianRule::AntiHebb => Self::AntiHebb,
            CoreHebbianRule::Covariance => Self::Covariance,
        }
    }
}

/// An outcome channel a reward-modulated learner reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
pub enum OutcomeChannel {
    EnergyDelta,
    ActionSuccess,
    DamageDelta,
    OffspringSuccess,
}

impl From<CoreOutcomeChannel> for OutcomeChannel {
    fn from(channel: CoreOutcomeChannel) -> Self {
        match channel {
            CoreOutcomeChannel::EnergyDelta => Self::EnergyDelta,
            CoreOutcomeChannel::ActionSuccess => Self::ActionSuccess,
            CoreOutcomeChannel::DamageDelta => Self::DamageDelta,
            CoreOutcomeChannel::OffspringSuccess => Self::OffspringSuccess,
        }
    }
}

// ── Mesh ────────────────────────────────────────────────────────────────────

/// A class of input a node reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
#[serde(rename_all = "snake_case")]
pub enum MeshReadClass {
    Food,
    Neighbor,
    Barrier,
    Occupancy,
    Introspection,
    Upstream,
    ActionQueue,
    Decision,
}

impl From<core_mesh::MeshReadClass> for MeshReadClass {
    fn from(class: core_mesh::MeshReadClass) -> Self {
        use core_mesh::MeshReadClass as C;
        match class {
            C::Food => Self::Food,
            C::Neighbor => Self::Neighbor,
            C::Barrier => Self::Barrier,
            C::Occupancy => Self::Occupancy,
            C::Introspection => Self::Introspection,
            C::Upstream => Self::Upstream,
            C::ActionQueue => Self::ActionQueue,
            C::Decision => Self::Decision,
        }
    }
}

/// A class of output a node writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
#[serde(rename_all = "snake_case")]
pub enum MeshWriteClass {
    Route,
    Action,
    Memory,
    Payload,
}

impl From<core_mesh::MeshWriteClass> for MeshWriteClass {
    fn from(class: core_mesh::MeshWriteClass) -> Self {
        use core_mesh::MeshWriteClass as C;
        match class {
            C::Route => Self::Route,
            C::Action => Self::Action,
            C::Memory => Self::Memory,
            C::Payload => Self::Payload,
        }
    }
}

/// A candidate routing target of a node.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct RouteTarget {
    pub target_id: NodeId,
    pub slot: u8,
    pub gate_bias: f32,
}

impl From<&v3_core::contracts::RouteTarget> for RouteTarget {
    fn from(target: &v3_core::contracts::RouteTarget) -> Self {
        Self {
            target_id: target.target_id,
            slot: target.slot,
            gate_bias: target.gate_bias,
        }
    }
}

// ── VM ──────────────────────────────────────────────────────────────────────

/// One VM instruction, one variant per opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS, serde::Deserialize))]
pub enum VmInstruction {
    Noop,
    LoadConst {
        dst: u8,
        const_idx: u8,
    },
    Move {
        dst: u8,
        src: u8,
    },
    Add {
        dst: u8,
        a: u8,
        b: u8,
    },
    Sub {
        dst: u8,
        a: u8,
        b: u8,
    },
    Mul {
        dst: u8,
        a: u8,
        b: u8,
    },
    Div {
        dst: u8,
        a: u8,
        b: u8,
    },
    Min {
        dst: u8,
        a: u8,
        b: u8,
    },
    Max {
        dst: u8,
        a: u8,
        b: u8,
    },
    Abs {
        dst: u8,
        src: u8,
    },
    Neg {
        dst: u8,
        src: u8,
    },
    Clamp01 {
        dst: u8,
        src: u8,
    },
    CmpGt {
        dst: u8,
        a: u8,
        b: u8,
    },
    CmpLt {
        dst: u8,
        a: u8,
        b: u8,
    },
    CmpEq {
        dst: u8,
        a: u8,
        b: u8,
        eps: u8,
    },
    And {
        dst: u8,
        a: u8,
        b: u8,
    },
    Or {
        dst: u8,
        a: u8,
        b: u8,
    },
    Not {
        dst: u8,
        src: u8,
    },
    ToI32 {
        dst: u8,
        src: u8,
    },
    ToU8 {
        dst: u8,
        src: u8,
    },
    ToBool {
        dst: u8,
        src: u8,
    },
    JumpIfZero {
        cond: u8,
        offset: i32,
    },
    Jump {
        offset: i32,
    },
    ReadInput {
        dst: u8,
        ref_idx: u16,
        sub_idx: u16,
    },
    WriteInternalPayload {
        slot_idx: u8,
        src: u8,
    },
    WriteActionParam {
        field_idx: u8,
        src: u8,
    },
    AddVote {
        sink: u8,
        src: u8,
    },
    WriteRouteGate {
        slot: u8,
        src: u8,
    },
    ReadActionQueueLength {
        dst: u8,
    },
    ReadActionQueueType {
        index_src: u8,
        dst: u8,
    },
    ReadActionQueueParam {
        index_src: u8,
        param_slot: u8,
        dst: u8,
    },
    SetPriorityBid {
        src: u8,
    },
    Halt,
    LoadSlot {
        dst: u8,
        slot_reg: u8,
    },
    StoreSlot {
        slot_reg: u8,
        src: u8,
    },
    LoadSlotImm {
        dst: u8,
        slot_idx: u8,
    },
    StoreSlotImm {
        slot_idx: u8,
        src: u8,
    },
    LoadSlotPrev {
        dst: u8,
        slot_idx: u8,
    },
    ClearSlot {
        slot_idx: u8,
    },
}

impl From<&CoreVmInstruction> for VmInstruction {
    fn from(instruction: &CoreVmInstruction) -> Self {
        use CoreVmInstruction as I;
        match *instruction {
            I::Noop => Self::Noop,
            I::LoadConst { dst, const_idx } => Self::LoadConst { dst, const_idx },
            I::Move { dst, src } => Self::Move { dst, src },
            I::Add { dst, a, b } => Self::Add { dst, a, b },
            I::Sub { dst, a, b } => Self::Sub { dst, a, b },
            I::Mul { dst, a, b } => Self::Mul { dst, a, b },
            I::Div { dst, a, b } => Self::Div { dst, a, b },
            I::Min { dst, a, b } => Self::Min { dst, a, b },
            I::Max { dst, a, b } => Self::Max { dst, a, b },
            I::Abs { dst, src } => Self::Abs { dst, src },
            I::Neg { dst, src } => Self::Neg { dst, src },
            I::Clamp01 { dst, src } => Self::Clamp01 { dst, src },
            I::CmpGt { dst, a, b } => Self::CmpGt { dst, a, b },
            I::CmpLt { dst, a, b } => Self::CmpLt { dst, a, b },
            I::CmpEq { dst, a, b, eps } => Self::CmpEq { dst, a, b, eps },
            I::And { dst, a, b } => Self::And { dst, a, b },
            I::Or { dst, a, b } => Self::Or { dst, a, b },
            I::Not { dst, src } => Self::Not { dst, src },
            I::ToI32 { dst, src } => Self::ToI32 { dst, src },
            I::ToU8 { dst, src } => Self::ToU8 { dst, src },
            I::ToBool { dst, src } => Self::ToBool { dst, src },
            I::JumpIfZero { cond, offset } => Self::JumpIfZero { cond, offset },
            I::Jump { offset } => Self::Jump { offset },
            I::ReadInput {
                dst,
                ref_idx,
                sub_idx,
            } => Self::ReadInput {
                dst,
                ref_idx,
                sub_idx,
            },
            I::WriteInternalPayload { slot_idx, src } => {
                Self::WriteInternalPayload { slot_idx, src }
            }
            I::WriteActionParam { field_idx, src } => Self::WriteActionParam { field_idx, src },
            I::AddVote { sink, src } => Self::AddVote { sink, src },
            I::WriteRouteGate { slot, src } => Self::WriteRouteGate { slot, src },
            I::ReadActionQueueLength { dst } => Self::ReadActionQueueLength { dst },
            I::ReadActionQueueType { index_src, dst } => {
                Self::ReadActionQueueType { index_src, dst }
            }
            I::ReadActionQueueParam {
                index_src,
                param_slot,
                dst,
            } => Self::ReadActionQueueParam {
                index_src,
                param_slot,
                dst,
            },
            I::SetPriorityBid { src } => Self::SetPriorityBid { src },
            I::Halt => Self::Halt,
            I::LoadSlot { dst, slot_reg } => Self::LoadSlot { dst, slot_reg },
            I::StoreSlot { slot_reg, src } => Self::StoreSlot { slot_reg, src },
            I::LoadSlotImm { dst, slot_idx } => Self::LoadSlotImm { dst, slot_idx },
            I::StoreSlotImm { slot_idx, src } => Self::StoreSlotImm { slot_idx, src },
            I::LoadSlotPrev { dst, slot_idx } => Self::LoadSlotPrev { dst, slot_idx },
            I::ClearSlot { slot_idx } => Self::ClearSlot { slot_idx },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deserializer that answers nothing but records the serialized variant
    /// names a derived `Deserialize` declares for an enum.
    struct VariantNames<'a>(&'a mut &'static [&'static str]);

    impl<'de> serde::Deserializer<'de> for VariantNames<'_> {
        type Error = serde::de::value::Error;

        fn deserialize_any<V: serde::de::Visitor<'de>>(
            self,
            _: V,
        ) -> Result<V::Value, Self::Error> {
            Err(serde::de::Error::custom("not an enum"))
        }

        fn deserialize_enum<V: serde::de::Visitor<'de>>(
            self,
            _name: &'static str,
            variants: &'static [&'static str],
            _: V,
        ) -> Result<V::Value, Self::Error> {
            *self.0 = variants;
            Err(serde::de::Error::custom("variant names read"))
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes
            byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map
            struct identifier ignored_any
        }
    }

    /// `T`'s serialized variant names, in declaration order.
    fn variant_names<T: serde::de::DeserializeOwned>() -> &'static [&'static str] {
        let mut names: &'static [&'static str] = &[];
        let _ = T::deserialize(VariantNames(&mut names));
        assert!(
            !names.is_empty(),
            "{} is not an enum",
            std::any::type_name::<T>()
        );
        names
    }

    /// The wire mirror has exactly the core enum's variants under the core
    /// enum's serialized names, so neither a renamed nor a surplus wire
    /// variant passes (a missing one fails the exhaustive `From`). The
    /// declaration order is pinned too: a mirror lists its variants as the
    /// core enum does.
    fn assert_same_variants<C, W>()
    where
        C: serde::de::DeserializeOwned,
        W: serde::de::DeserializeOwned,
    {
        assert_eq!(
            variant_names::<C>(),
            variant_names::<W>(),
            "{} mirrors {}",
            std::any::type_name::<W>(),
            std::any::type_name::<C>()
        );
    }

    /// The mirror of `core` serializes as `core` does.
    fn assert_same_json<C: Serialize, W: Serialize>(core: &C, wire: &W) {
        assert_eq!(
            serde_json::to_value(core).unwrap(),
            serde_json::to_value(wire).unwrap()
        );
    }

    /// `all` names every variant of `C` once, and each serializes
    /// identically through its mirror `W`.
    fn assert_mirrors_all<C, W>(all: &[C])
    where
        C: Copy + Serialize + serde::de::DeserializeOwned,
        W: From<C> + Serialize,
    {
        assert_eq!(all.len(), variant_names::<C>().len());
        for &core in all {
            assert_same_json(&core, &W::from(core));
        }
    }

    /// Every enum this module and `brain` mirror has the core enum's variant
    /// set and serialized names, and every core value an `ALL` list or the
    /// fixed sink catalog enumerates serializes identically through its
    /// mirror. `Site`, `WeightSurface`, `MemberPolicy` and `LearnerSource` are
    /// wire-named (their core sources carry no serde form), as are
    /// `InputFamily` and `InputKey` (the flat reference, tested below); their
    /// exhaustive `From` is their pin.
    #[test]
    fn mirrors_serialize_as_the_core_types_do() {
        use crate::transport::brain::{BundleSource, Wiring};

        assert_same_variants::<core_vote::DirectedVoteKind, DirectedVoteKind>();
        assert_same_variants::<core_vote::VoteSink, VoteSink>();
        assert_same_variants::<core_vote::ActionParamField, ActionParamField>();
        assert_same_variants::<CoreWorldAction, WorldAction>();
        assert_same_variants::<core_cgp::ComputeNodeKind, ComputeNodeKind>();
        assert_same_variants::<core_cgp::OutputSinkKind, OutputSinkKind>();
        assert_same_variants::<core_cgp::GraphSource, GraphSource>();
        assert_same_variants::<CoreHebbianRule, HebbianRule>();
        assert_same_variants::<CoreOutcomeChannel, OutcomeChannel>();
        assert_same_variants::<core_mesh::MeshReadClass, MeshReadClass>();
        assert_same_variants::<core_mesh::MeshWriteClass, MeshWriteClass>();
        assert_same_variants::<CoreVmInstruction, VmInstruction>();
        assert_same_variants::<core_cgp::BundleSource, BundleSource>();
        assert_same_variants::<core_cgp::Wiring, Wiring>();

        assert_mirrors_all::<_, DirectedVoteKind>(&core_vote::DirectedVoteKind::ALL);
        assert_mirrors_all::<_, ActionParamField>(&core_vote::ActionParamField::ALL);
        assert_mirrors_all::<_, HebbianRule>(&CoreHebbianRule::ALL);
        assert_mirrors_all::<_, OutcomeChannel>(&CoreOutcomeChannel::ALL);
        assert_mirrors_all::<_, MeshReadClass>(&core_mesh::MeshReadClass::ALL);
        assert_mirrors_all::<_, MeshWriteClass>(&core_mesh::MeshWriteClass::ALL);
        // The fixed sink catalog holds every sink kind and every vote sink.
        for sink in core_cgp::CgpGraphBackendDef::new_with_fixed_outputs()
            .output_sinks()
            .iter()
        {
            assert_same_json(&sink.kind, &OutputSinkKind::from(sink.kind));
        }

        // The payload-carrying enums have no catalog: their names are pinned
        // above, and these samples pin some payload shapes.
        for kind in [
            core_cgp::ComputeNodeKind::Threshold(0.5),
            core_cgp::ComputeNodeKind::Add,
            core_cgp::ComputeNodeKind::Constant(-1.0),
        ] {
            assert_same_json(&kind, &ComputeNodeKind::from(kind));
        }
        for instruction in [
            CoreVmInstruction::Noop,
            CoreVmInstruction::JumpIfZero {
                cond: 1,
                offset: -3,
            },
            CoreVmInstruction::ReadActionQueueParam {
                index_src: 1,
                param_slot: 2,
                dst: 3,
            },
        ] {
            assert_same_json(&instruction, &VmInstruction::from(&instruction));
        }
        let source = core_cgp::GraphSource::SharedMemory {
            slot: 2,
            previous: true,
        };
        assert_same_json(&source, &GraphSource::from(source));
    }

    #[test]
    fn an_input_reference_is_a_flat_record() {
        let typed = InputReference::World(WorldInputKey::AreaFoodSummary {
            type_idx: OrdinaryFoodTypeId::new(1),
        });
        assert_eq!(
            serde_json::to_value(InputRef::from(&typed)).unwrap(),
            serde_json::json!({"family": "world", "key": "AreaFoodSummary", "food_type": 1})
        );
        assert_eq!(
            serde_json::to_value(InputRef::from(&InputReference::UpstreamSlot(3))).unwrap(),
            serde_json::json!({"family": "upstream", "key": "UpstreamSlot", "slot": 3})
        );
        assert_eq!(
            serde_json::to_value(InputRef::from(&InputReference::ActionVotes)).unwrap(),
            serde_json::json!({"family": "decision", "key": "ActionVotes"})
        );
    }
}
