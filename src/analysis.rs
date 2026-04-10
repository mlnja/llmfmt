//! Data-shape analysis used by heuristic profiles.

use std::collections::BTreeSet;

use serde::Serialize;
use serde_json::Value;

/// Summary of the parsed input used to select an output format.
#[derive(Debug, Clone, Serialize)]
pub struct DataAnalysis {
    pub depth: usize,
    pub row_count: usize,
    pub field_count: usize,
    pub sparsity: f32,
    pub uniformity: f32,
    pub scalar_field_count: usize,
    pub nested_field_count: usize,
    pub nested_value_field_count: usize,
    pub has_nested_arrays: bool,
    pub is_uniform_object_array: bool,
    pub is_flat_object_array: bool,
    pub is_deeply_nested: bool,
}

pub fn analyze(value: &Value) -> DataAnalysis {
    let depth = max_depth(value);
    let has_nested_arrays = has_nested_arrays(value, false);
    let (
        row_count,
        field_count,
        sparsity,
        uniformity,
        scalar_field_count,
        nested_field_count,
        nested_value_field_count,
        is_uniform_object_array,
        is_flat_object_array,
    ) = analyze_top_level_array(value);

    DataAnalysis {
        depth,
        row_count,
        field_count,
        sparsity,
        uniformity,
        scalar_field_count,
        nested_field_count,
        nested_value_field_count,
        has_nested_arrays,
        is_uniform_object_array,
        is_flat_object_array,
        is_deeply_nested: depth >= 5,
    }
}

fn max_depth(value: &Value) -> usize {
    match value {
        Value::Array(values) => 1 + values.iter().map(max_depth).max().unwrap_or(0),
        Value::Object(map) => 1 + map.values().map(max_depth).max().unwrap_or(0),
        _ => 0,
    }
}

fn has_nested_arrays(value: &Value, inside_array: bool) -> bool {
    match value {
        Value::Array(values) => {
            if inside_array && !values.is_empty() {
                return true;
            }
            values.iter().any(|value| has_nested_arrays(value, true))
        }
        Value::Object(map) => map
            .values()
            .any(|value| has_nested_arrays(value, inside_array)),
        _ => false,
    }
}

fn analyze_top_level_array(
    value: &Value,
) -> (usize, usize, f32, f32, usize, usize, usize, bool, bool) {
    let Value::Array(rows) = value else {
        return (0, 0, 0.0, 0.0, 0, 0, 0, false, false);
    };
    if rows.is_empty() {
        return (0, 0, 0.0, 1.0, 0, 0, 0, false, false);
    }

    let object_rows: Vec<_> = rows
        .iter()
        .filter_map(|row| match row {
            Value::Object(map) => Some(map),
            _ => None,
        })
        .collect();
    if object_rows.len() != rows.len() {
        return (rows.len(), 0, 1.0, 0.0, 0, 0, 0, false, false);
    }

    let mut all_fields = BTreeSet::new();
    let mut signatures = Vec::with_capacity(object_rows.len());
    let mut empty_cells = 0usize;
    let mut non_flat = false;
    let mut scalar_field_count = 0usize;
    let mut nested_field_count = 0usize;
    let mut nested_value_field_count = 0usize;

    for row in &object_rows {
        let mut fields = BTreeSet::new();
        for (key, value) in row.iter() {
            all_fields.insert(key.clone());
            fields.insert(key.clone());
            if is_nested(value) {
                non_flat = true;
                nested_field_count += 1;
                nested_value_field_count += count_leaf_scalars(value);
            } else {
                scalar_field_count += 1;
            }
        }
        signatures.push(fields);
    }

    let total_fields = all_fields.len();
    if total_fields == 0 {
        return (rows.len(), 0, 0.0, 1.0, 0, 0, 0, true, true);
    }

    for signature in &signatures {
        empty_cells += total_fields - signature.len();
    }

    let baseline = &signatures[0];
    let matching = signatures.iter().filter(|sig| *sig == baseline).count();
    let uniformity = matching as f32 / rows.len() as f32;
    let sparsity = empty_cells as f32 / (rows.len() * total_fields) as f32;
    let uniform = signatures.iter().all(|sig| sig == baseline);

    (
        rows.len(),
        total_fields,
        sparsity,
        uniformity,
        scalar_field_count,
        nested_field_count,
        nested_value_field_count,
        uniform,
        uniform && !non_flat,
    )
}

fn is_nested(value: &Value) -> bool {
    matches!(value, Value::Array(_) | Value::Object(_))
}

fn count_leaf_scalars(value: &Value) -> usize {
    match value {
        Value::Array(values) => values.iter().map(count_leaf_scalars).sum(),
        Value::Object(map) => map.values().map(count_leaf_scalars).sum(),
        _ => 1,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::analyze;

    #[test]
    fn detects_flat_uniform_array() {
        let value = json!([
            {"id": 1, "name": "a"},
            {"id": 2, "name": "b"}
        ]);
        let analysis = analyze(&value);
        assert!(analysis.is_uniform_object_array);
        assert!(analysis.is_flat_object_array);
        assert_eq!(analysis.field_count, 2);
        assert_eq!(analysis.scalar_field_count, 4);
        assert_eq!(analysis.nested_field_count, 0);
        assert_eq!(analysis.nested_value_field_count, 0);
    }

    #[test]
    fn detects_irregular_array() {
        let value = json!([
            {"id": 1, "name": "a"},
            {"id": 2, "role": "b"}
        ]);
        let analysis = analyze(&value);
        assert!(!analysis.is_uniform_object_array);
        assert!(analysis.sparsity > 0.0);
    }

    #[test]
    fn detects_nested_arrays_and_deep_nesting() {
        let value = json!({
            "outer": [
                {"items": [{"id": 1}]}
            ]
        });
        let analysis = analyze(&value);
        assert!(analysis.has_nested_arrays);
        assert!(!analysis.is_flat_object_array);
    }

    #[test]
    fn marks_deep_values() {
        let value = json!({"a":{"b":{"c":{"d":{"e":1}}}}});
        let analysis = analyze(&value);
        assert!(analysis.is_deeply_nested);
        assert!(analysis.depth >= 5);
    }

    #[test]
    fn counts_nested_and_scalar_fields_for_object_rows() {
        let value = json!([
            {"id": 1, "name": "alice", "address": {"city": "x"}, "tags": ["a"]},
            {"id": 2, "name": "bob", "address": {"city": "y"}, "tags": ["b"]}
        ]);
        let analysis = analyze(&value);
        assert_eq!(analysis.scalar_field_count, 4);
        assert_eq!(analysis.nested_field_count, 4);
        assert_eq!(analysis.nested_value_field_count, 4);
        assert!(!analysis.is_flat_object_array);
    }
}
