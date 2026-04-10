//! Output-shape validation that is stricter than generic serialization.

use serde_json::Value;

use crate::error::{LlmFmtError, Result};

pub fn validate_tsv_value(value: &Value) -> Result<()> {
    let rows = match value {
        Value::Array(rows) => rows,
        Value::Object(map) if map.len() == 1 => match map.values().next() {
            Some(Value::Array(rows)) => rows,
            _ => {
                return Err(LlmFmtError::Message(
                    "tsv output requires a top-level array or a single-key object containing an array"
                        .to_owned(),
                ));
            }
        },
        _ => {
            return Err(LlmFmtError::Message(
                "tsv output requires a top-level array or a single-key object containing an array"
                    .to_owned(),
            ));
        }
    };

    for row in rows {
        let Value::Object(map) = row else {
            return Err(LlmFmtError::Message(
                "tsv output requires every row to be an object".to_owned(),
            ));
        };

        if map
            .values()
            .any(|value| matches!(value, Value::Array(_) | Value::Object(_)))
        {
            return Err(LlmFmtError::Message(
                "tsv output requires flat scalar fields; nested arrays and objects are not supported"
                    .to_owned(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::validate_tsv_value;

    #[test]
    fn rejects_nested_tsv_rows() {
        let value = json!([{"id": 1, "tags": ["a"]}]);
        assert!(validate_tsv_value(&value).is_err());
    }

    #[test]
    fn rejects_non_object_rows() {
        let value = json!([1, 2, 3]);
        assert!(validate_tsv_value(&value).is_err());
    }

    #[test]
    fn rejects_non_table_top_level() {
        let value = json!({"users": {"id": 1}});
        assert!(validate_tsv_value(&value).is_err());
    }

    #[test]
    fn accepts_single_key_object_with_array() {
        let value = json!({"users": [{"id": 1, "name": "alice"}]});
        assert!(validate_tsv_value(&value).is_ok());
    }
}
