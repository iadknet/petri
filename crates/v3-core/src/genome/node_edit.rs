//! The node's declaration edits (T25.F11, G13): every write of a node's
//! `input_refs` and every bundle creation goes through one of these owner
//! operations, which check everything before writing, so a failed edit
//! leaves the node exactly as it was.

use std::collections::BTreeSet;

use crate::contracts::{FieldSet, InputReference, SensorContext, SensorDescriptor, MEMORY_BANK};
use crate::genome::cgp::{
    bundle_capacity_admits, plans, BundleBirth, BundleId, BundleSource, GraphSource,
};
use crate::genome::mesh_annotations::{classify_input_ref, MeshReadClass};
use crate::genome::{BackendDef, NodeGenome, VmInstruction};

/// Why a node edit wrote nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeEditError {
    /// The new declaration's index does not fit `u16`.
    DeclarationIndex,
    /// The edit would exceed `u16::MAX` bundle records.
    BundleCapacity,
    /// The births do not cover the sensor's field sets exactly once on a
    /// Graph node, or are non-empty on a VM node.
    Coverage,
    /// A birth's wiring is not admissible for its field set on this graph.
    Inadmissible,
    /// The source names no declared sensor or field set.
    UnknownSource,
    /// The node has no Graph backend.
    NotGraph,
    /// The declaration index is past the end.
    OutOfRange,
    /// A backend consumer addresses the declaration.
    Addressed,
    /// The replacement is of another kind.
    KindMismatch,
}

/// The kind of an `input_refs` entry: its mesh read class and width. A
/// `swap` only ever replaces an entry with another member of the same kind,
/// so a consumer keeps reading the same modality at the same width for the
/// node's life and its copies' (T11.F22).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputRefKind {
    pub class: MeshReadClass,
    pub width: u32,
}

/// The kind partition every within-kind rule reads: `swap`, the `Swap`
/// operator, its applicability predicate and the probes.
#[must_use]
pub fn input_ref_kind(reference: &InputReference, context: &SensorContext) -> InputRefKind {
    InputRefKind {
        class: classify_input_ref(reference),
        width: SensorDescriptor::of(reference, context).width(),
    }
}

impl NodeGenome {
    /// The `input_refs` indices some consumer on the backend addresses: a
    /// `GraphSource::InputLeaf` on any edge container (a bundle's members
    /// included), or a VM `ReadInput` anywhere in the program, live or not.
    #[must_use]
    pub fn addressed_input_refs(&self) -> BTreeSet<u16> {
        match &self.backend_def {
            BackendDef::Graph(def) => def
                .edges()
                .filter_map(|edge| match edge.source() {
                    GraphSource::InputLeaf { ref_idx, .. } => Some(ref_idx),
                    GraphSource::SharedMemory { .. } | GraphSource::ComputeNode(_) => None,
                })
                .collect(),
            BackendDef::Vm(vm) => vm
                .program
                .iter()
                .filter_map(|instruction| match instruction {
                    VmInstruction::ReadInput { ref_idx, .. } => Some(*ref_idx),
                    _ => None,
                })
                .collect(),
        }
    }

    /// The field set `source` names on this node under `context`: a
    /// declared sensor's set, or a memory bank's.
    #[must_use]
    pub fn field_set(&self, source: BundleSource, context: &SensorContext) -> Option<FieldSet> {
        match source {
            BundleSource::Sensor { ref_idx, field_set } => {
                let reference = self.input_refs.get(usize::from(ref_idx))?;
                SensorDescriptor::of(reference, context).field_set(field_set)
            }
            BundleSource::Memory { .. } => Some(MEMORY_BANK),
        }
    }

    /// Declare `reference` and build one bundle per birth in one edit,
    /// returning the declaration's index. On a Graph node `births` covers
    /// the sensor's field sets, each once, every wiring admissible for its
    /// set; on a VM node it is empty.
    ///
    /// # Errors
    ///
    /// Any check that fails; nothing is written.
    pub fn add_sensor(
        &mut self,
        reference: InputReference,
        births: &[BundleBirth],
        context: &SensorContext,
    ) -> Result<u16, NodeEditError> {
        let ref_idx =
            u16::try_from(self.input_refs.len()).map_err(|_| NodeEditError::DeclarationIndex)?;
        let descriptor = SensorDescriptor::of(&reference, context);
        match &self.backend_def {
            BackendDef::Vm(_) if !births.is_empty() => return Err(NodeEditError::Coverage),
            BackendDef::Vm(_) => {}
            BackendDef::Graph(def) => {
                let covered: BTreeSet<u8> = births.iter().map(|birth| birth.field_set).collect();
                let exact = covered.len() == births.len()
                    && covered.len() == descriptor.field_set_count()
                    && covered
                        .iter()
                        .all(|&index| usize::from(index) < descriptor.field_set_count());
                if !exact {
                    return Err(NodeEditError::Coverage);
                }
                for birth in births {
                    let set = descriptor
                        .field_set(birth.field_set)
                        .ok_or(NodeEditError::Coverage)?;
                    if !plans(def, &set).contains(&birth.wiring) {
                        return Err(NodeEditError::Inadmissible);
                    }
                }
                if !bundle_capacity_admits(def.bundles().len(), births.len()) {
                    return Err(NodeEditError::BundleCapacity);
                }
            }
        }
        self.input_refs.push(reference);
        for &birth in births {
            let source = BundleSource::Sensor {
                ref_idx,
                field_set: birth.field_set,
            };
            self.add_bundle(source, birth, context)
                .expect("a birth checked against the declared sensor");
        }
        Ok(ref_idx)
    }

    /// Build one bundle reading `source` (a declared sensor's field set or a
    /// memory bank) with `birth`'s wiring and scale.
    ///
    /// # Errors
    ///
    /// The node has no Graph backend, the source does not resolve or names
    /// another set than `birth`, the wiring is not admissible, or the graph
    /// is at capacity; nothing is written.
    pub fn add_bundle(
        &mut self,
        source: BundleSource,
        birth: BundleBirth,
        context: &SensorContext,
    ) -> Result<BundleId, NodeEditError> {
        let set = self
            .field_set(source, context)
            .filter(|_| birth.field_set == source.field_set())
            .ok_or(NodeEditError::UnknownSource)?;
        let BackendDef::Graph(def) = &mut self.backend_def else {
            return Err(NodeEditError::NotGraph);
        };
        def.add_bundle(source, set, birth)
    }

    /// Delete declaration `ref_idx`, which no backend consumer addresses,
    /// renumbering every higher index in the backend and every bundle source
    /// in the same edit.
    ///
    /// # Errors
    ///
    /// `ref_idx` is past the end or addressed; nothing is written.
    pub fn prune(&mut self, ref_idx: u16) -> Result<(), NodeEditError> {
        if usize::from(ref_idx) >= self.input_refs.len() {
            return Err(NodeEditError::OutOfRange);
        }
        if self.addressed_input_refs().contains(&ref_idx) {
            return Err(NodeEditError::Addressed);
        }
        self.input_refs.remove(usize::from(ref_idx));
        self.backend_def.reindex_input_refs_after_removal(ref_idx);
        Ok(())
    }

    /// Replace declaration `ref_idx` with `reference`, a member of the same
    /// [`InputRefKind`], so every consumer keeps its width.
    ///
    /// # Errors
    ///
    /// `ref_idx` is past the end or `reference` is of another kind; nothing
    /// is written.
    pub fn swap(
        &mut self,
        ref_idx: u16,
        reference: InputReference,
        context: &SensorContext,
    ) -> Result<(), NodeEditError> {
        let entry = self
            .input_refs
            .get_mut(usize::from(ref_idx))
            .ok_or(NodeEditError::OutOfRange)?;
        if input_ref_kind(entry, context) != input_ref_kind(&reference, context) {
            return Err(NodeEditError::KindMismatch);
        }
        *entry = reference;
        Ok(())
    }
}
