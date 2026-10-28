//! Summary retention declared with the persistence sample's types
//! (T30.F13): every sample key states, beside its field, whether the v2
//! summary keeps it whole, keeps only the full report's copy (named in
//! `omitted_details`), projects it by its own type's declarations, or is a
//! flattened record whose fields declare for themselves. The summary
//! keep-lists derive from these declarations (`artifacts::sample_retention`),
//! so a new sample block is one field, one retention entry and one reader
//! call. This module declares only; it depends on no sample type.

/// How the v2 summary keeps one persistence-sample key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retention {
    /// Kept whole on every checkpoint.
    Kept,
    /// In the full report only; the summary names it in `omitted_details`.
    FullReport,
    /// Projected by the block type's own [`CheckpointRetention`]
    /// declarations (`bundles`).
    Projected(&'static [(&'static str, CheckpointRetention)]),
    /// A flattened record whose own fields declare.
    Flattened,
}

/// How the summary keeps one field of a projected block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointRetention {
    /// On every checkpoint.
    Every,
    /// On each world's last checkpoint only.
    Last,
}

/// A struct whose every field declares its retention beside it, as
/// `field: Type => retention`; the macro emits the struct unchanged and its
/// `RETENTION` table of `(serialized key, retention)` in field order. No
/// field may be renamed in serde, since the key is the field's name; the
/// declaration tests check every serialized key against the table.
macro_rules! retained_struct {
    (
        retention: $retention:ty;
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $type:ty => $kept:expr
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field: $type,
            )*
        }

        impl $name {
            /// Every field's serialized key and its summary retention, in
            /// field order.
            pub const RETENTION: &'static [(&'static str, $retention)] =
                &[$((stringify!($field), $kept)),*];
        }
    };
}

pub(crate) use retained_struct;
