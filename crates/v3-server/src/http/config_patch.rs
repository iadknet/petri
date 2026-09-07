//! Attribution of a rejected runtime config patch to the submitted fields.
//!
//! `PATCH /v3/simulation/config` rejects, never clamps, and never applies part
//! of a patch: if canonical normalization would rewrite anything in the merged
//! config, the whole patch is refused. This module works out *which* submitted
//! leaves caused that, so the panel can point at the control the user must fix
//! (2026-09-07 config panel apply audit, items 3 and 4).

use serde_json::Value;

use crate::error::FieldError;

/// A single leaf of a submitted patch, with the minimal patch that applies it
/// on its own.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PatchLeaf {
    /// Dotted path of the leaf, with array indices as plain segments.
    pub(crate) path: String,
    /// The submitted value at that path.
    pub(crate) value: Value,
    /// A patch containing only this leaf.
    pub(crate) patch: Value,
}

/// One path at which normalization rewrote a submitted config.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct JsonDifference {
    pub(crate) path: String,
    pub(crate) submitted: Value,
    pub(crate) canonical: Value,
}

/// The merged and normalized serializations a patch would produce, or `None`
/// when the patch does not deserialize into a config on its own.
pub(crate) type CanonicalizedPatch = Option<(Value, Value)>;

/// Names every submitted leaf whose isolated application canonical constraints
/// would rewrite.
///
/// `canonicalize` maps a patch to the `(merged, normalized)` JSON
/// serializations of the config it would produce. When no single leaf offends
/// in isolation, the whole-patch difference between `merged` and `normalized`
/// is reported instead, so a rejection always names at least one path.
pub(crate) fn attribute_patch_rejection<F>(
    patch: &Value,
    merged: &Value,
    normalized: &Value,
    canonicalize: F,
) -> Vec<FieldError>
where
    F: Fn(&Value) -> CanonicalizedPatch,
{
    let mut field_errors: Vec<FieldError> = patch_leaves(patch)
        .into_iter()
        .filter_map(|leaf| {
            let (leaf_merged, leaf_normalized) = canonicalize(&leaf.patch)?;
            let differences = json_differences(&leaf_merged, &leaf_normalized);
            if differences.is_empty() {
                return None;
            }
            let reason = rejection_reason(&leaf.path, &leaf.value, &differences);
            Some(FieldError {
                field: leaf.path,
                reason,
            })
        })
        .collect();

    if field_errors.is_empty() {
        field_errors = json_differences(merged, normalized)
            .into_iter()
            .map(|difference| FieldError {
                reason: rejection_reason(
                    &difference.path,
                    &difference.submitted,
                    std::slice::from_ref(&difference),
                ),
                field: difference.path,
            })
            .collect();
    }

    field_errors
}

fn rejection_reason(path: &str, requested: &Value, differences: &[JsonDifference]) -> String {
    let mut reason = format!("requested {requested}");
    for difference in differences {
        if difference.path == path {
            reason.push_str(&format!("; canonical value is {}", difference.canonical));
        } else {
            reason.push_str(&format!(
                "; canonical constraints move {} to {}",
                difference.path, difference.canonical
            ));
        }
    }
    reason
}

/// Flattens a patch into its leaves. Arrays are treated as single leaves,
/// matching `deep_merge`, which replaces an array wholesale.
fn patch_leaves(patch: &Value) -> Vec<PatchLeaf> {
    let mut leaves = Vec::new();
    collect_patch_leaves(patch, &mut Vec::new(), &mut leaves);
    leaves
}

fn collect_patch_leaves(value: &Value, prefix: &mut Vec<String>, leaves: &mut Vec<PatchLeaf>) {
    if let Value::Object(map) = value {
        for (key, child) in map {
            prefix.push(key.clone());
            collect_patch_leaves(child, prefix, leaves);
            prefix.pop();
        }
        return;
    }
    if prefix.is_empty() {
        return;
    }
    leaves.push(PatchLeaf {
        path: prefix.join("."),
        value: value.clone(),
        patch: nest_leaf(prefix, value.clone()),
    });
}

fn nest_leaf(path: &[String], leaf: Value) -> Value {
    path.iter().rev().fold(leaf, |nested, key| {
        Value::Object(serde_json::Map::from_iter([(key.clone(), nested)]))
    })
}

/// Lists every path at which `normalized` differs from `submitted`.
fn json_differences(submitted: &Value, normalized: &Value) -> Vec<JsonDifference> {
    let mut differences = Vec::new();
    collect_differences(submitted, normalized, &mut Vec::new(), &mut differences);
    differences
}

fn collect_differences(
    submitted: &Value,
    normalized: &Value,
    prefix: &mut Vec<String>,
    differences: &mut Vec<JsonDifference>,
) {
    match (submitted, normalized) {
        (Value::Object(left), Value::Object(right)) if left.keys().eq(right.keys()) => {
            for (key, left_child) in left {
                prefix.push(key.clone());
                collect_differences(left_child, &right[key], prefix, differences);
                prefix.pop();
            }
        }
        (Value::Array(left), Value::Array(right)) if left.len() == right.len() => {
            for (index, (left_child, right_child)) in left.iter().zip(right).enumerate() {
                prefix.push(index.to_string());
                collect_differences(left_child, right_child, prefix, differences);
                prefix.pop();
            }
        }
        _ if submitted != normalized => differences.push(JsonDifference {
            path: prefix.join("."),
            submitted: submitted.clone(),
            canonical: normalized.clone(),
        }),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Stands in for merge-and-normalize: applies `patch` over `base` by deep
    /// merge, then lets `rewrite` express a canonical constraint.
    fn canonicalizer<'a>(
        base: &'a Value,
        rewrite: &'a dyn Fn(&mut Value),
    ) -> impl Fn(&Value) -> CanonicalizedPatch + 'a {
        move |patch: &Value| {
            let mut merged = base.clone();
            crate::types::deep_merge(&mut merged, patch.clone());
            let mut normalized = merged.clone();
            rewrite(&mut normalized);
            Some((merged, normalized))
        }
    }

    #[test]
    fn patch_leaves_flattens_nested_objects_into_isolated_patches() {
        let leaves = patch_leaves(&json!({"a": {"b": 1, "c": 2}, "d": [3]}));

        assert_eq!(
            leaves,
            vec![
                PatchLeaf {
                    path: "a.b".into(),
                    value: json!(1),
                    patch: json!({"a": {"b": 1}}),
                },
                PatchLeaf {
                    path: "a.c".into(),
                    value: json!(2),
                    patch: json!({"a": {"c": 2}}),
                },
                PatchLeaf {
                    path: "d".into(),
                    value: json!([3]),
                    patch: json!({"d": [3]}),
                },
            ]
        );
    }

    #[test]
    fn patch_leaves_of_an_empty_patch_is_empty() {
        assert!(patch_leaves(&json!({})).is_empty());
        assert!(patch_leaves(&json!({"a": {}})).is_empty());
    }

    #[test]
    fn json_differences_reports_object_and_array_paths() {
        let submitted = json!({"a": 1, "b": {"c": [1, 2]}});
        let normalized = json!({"a": 1, "b": {"c": [1, 9]}});

        assert_eq!(
            json_differences(&submitted, &normalized),
            vec![JsonDifference {
                path: "b.c.1".into(),
                submitted: json!(2),
                canonical: json!(9),
            }]
        );
    }

    #[test]
    fn json_differences_reports_a_reshaped_subtree_at_its_own_path() {
        let submitted = json!({"a": {"b": 1}});
        let normalized = json!({"a": {"b": 1, "c": 2}});

        assert_eq!(
            json_differences(&submitted, &normalized),
            vec![JsonDifference {
                path: "a".into(),
                submitted: json!({"b": 1}),
                canonical: json!({"b": 1, "c": 2}),
            }]
        );
    }

    #[test]
    fn json_differences_reports_a_resized_array_at_its_own_path() {
        let submitted = json!({"a": [1, 2]});
        let normalized = json!({"a": [1]});

        assert_eq!(
            json_differences(&submitted, &normalized),
            vec![JsonDifference {
                path: "a".into(),
                submitted: json!([1, 2]),
                canonical: json!([1]),
            }]
        );
    }

    #[test]
    fn attribution_names_the_leaf_whose_own_value_is_rewritten() {
        let base = json!({"limit": 10, "cap": 4});
        let rewrite = |value: &mut Value| {
            if value["limit"].as_i64() == Some(0) {
                value["limit"] = json!(10);
            }
        };
        let canonicalize = canonicalizer(&base, &rewrite);
        let patch = json!({"limit": 0});
        let (merged, normalized) = canonicalize(&patch).unwrap();

        let errors = attribute_patch_rejection(&patch, &merged, &normalized, &canonicalize);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "limit");
        assert_eq!(errors[0].reason, "requested 0; canonical value is 10");
    }

    #[test]
    fn attribution_names_the_cross_field_path_a_leaf_would_move() {
        let base = json!({"limit": 10, "cap": 4});
        let rewrite = |value: &mut Value| {
            let limit = value["limit"].as_i64().unwrap_or_default();
            if value["cap"].as_i64().unwrap_or_default() > limit {
                value["cap"] = json!(limit);
            }
        };
        let canonicalize = canonicalizer(&base, &rewrite);
        let patch = json!({"limit": 2});
        let (merged, normalized) = canonicalize(&patch).unwrap();

        let errors = attribute_patch_rejection(&patch, &merged, &normalized, &canonicalize);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "limit");
        assert_eq!(
            errors[0].reason,
            "requested 2; canonical constraints move cap to 2"
        );
    }

    #[test]
    fn attribution_ignores_leaves_that_are_acceptable_on_their_own() {
        let base = json!({"limit": 10, "cap": 4, "rate": 0.1});
        let rewrite = |value: &mut Value| {
            if value["limit"].as_i64() == Some(0) {
                value["limit"] = json!(10);
            }
        };
        let canonicalize = canonicalizer(&base, &rewrite);
        let patch = json!({"limit": 0, "rate": 0.5});
        let (merged, normalized) = canonicalize(&patch).unwrap();

        let errors = attribute_patch_rejection(&patch, &merged, &normalized, &canonicalize);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "limit");
    }

    #[test]
    fn attribution_falls_back_to_whole_patch_differences() {
        // A constraint that only trips when both leaves are submitted together,
        // so no leaf offends in isolation.
        let base = json!({"a": 0, "b": 0, "derived": 0});
        let rewrite = |value: &mut Value| {
            if value["a"].as_i64() == Some(1) && value["b"].as_i64() == Some(1) {
                value["derived"] = json!(2);
            }
        };
        let canonicalize = canonicalizer(&base, &rewrite);
        let patch = json!({"a": 1, "b": 1});
        let (merged, normalized) = canonicalize(&patch).unwrap();

        let errors = attribute_patch_rejection(&patch, &merged, &normalized, &canonicalize);

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "derived");
        assert_eq!(errors[0].reason, "requested 0; canonical value is 2");
    }

    #[test]
    fn attribution_skips_leaves_that_do_not_deserialize_on_their_own() {
        let patch = json!({"a": 1});
        let merged = json!({"a": 1});
        let normalized = json!({"a": 1});

        let errors = attribute_patch_rejection(&patch, &merged, &normalized, |_| None);

        assert!(errors.is_empty());
    }
}
