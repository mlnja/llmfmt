//! Deterministic emitters for prompt-facing output formats.

use std::collections::BTreeSet;

use serde_json::{Map, Value};
use toon_format::encode_default;

use crate::error::{LlmFmtError, Result};
use crate::format::OutputFormat;
use crate::validation::validate_tsv_value;

pub fn emit_output(format: OutputFormat, value: &Value) -> Result<String> {
    match format {
        OutputFormat::Toon => emit_toon(value),
        OutputFormat::Tsv => emit_tsv(value),
        OutputFormat::Yaml => Ok(serde_yaml::to_string(value)?),
        OutputFormat::JsonCompact => Ok(serde_json::to_string(value)?),
    }
}

pub fn wrap_output(format: OutputFormat, rendered: &str) -> String {
    format!("```{}\n{}\n```\n", format.label(), rendered.trim_end())
}

pub fn canonicalize_value(value: Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_value).collect()),
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut entries = map.into_iter().collect::<Vec<_>>();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            for (key, value) in entries {
                sorted.insert(key, canonicalize_value(value));
            }
            Value::Object(sorted)
        }
        other => other,
    }
}

fn emit_toon(value: &Value) -> Result<String> {
    Ok(encode_default(value)?)
}

fn emit_tsv(value: &Value) -> Result<String> {
    validate_tsv_value(value)?;
    let (_, rows) = top_level_table(value)?;
    let fields = union_fields(rows);

    let mut out = String::new();
    out.push_str(&fields.join("\t"));
    out.push('\n');
    for row in rows {
        let Value::Object(map) = row else {
            return Err(LlmFmtError::Message(
                "tsv output requires object rows".to_owned(),
            ));
        };
        let values = fields
            .iter()
            .map(|field| encode_scalar(map.get(field).unwrap_or(&Value::Null), '\t'))
            .collect::<Vec<_>>();
        out.push_str(&values.join("\t"));
        out.push('\n');
    }
    Ok(out)
}

fn top_level_table(value: &Value) -> Result<(String, &[Value])> {
    match value {
        Value::Array(rows) => Ok(("rows".to_owned(), rows.as_slice())),
        Value::Object(map) if map.len() == 1 => {
            let (name, value) = map.iter().next().expect("single entry");
            match value {
                Value::Array(rows) => Ok((name.clone(), rows.as_slice())),
                _ => Err(LlmFmtError::Message(
                    "tabular output requires a top-level array or a single-key object containing an array"
                        .to_owned(),
                )),
            }
        }
        _ => Err(LlmFmtError::Message(
            "tabular output requires a top-level array or a single-key object containing an array"
                .to_owned(),
        )),
    }
}

fn union_fields(rows: &[Value]) -> Vec<String> {
    let mut fields = BTreeSet::new();
    for row in rows {
        if let Value::Object(map) = row {
            for key in map.keys() {
                fields.insert(key.clone());
            }
        }
    }
    fields.into_iter().collect()
}

fn encode_scalar(value: &Value, delimiter: char) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => escape_text(value, delimiter),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

fn escape_text(value: &str, delimiter: char) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\\' => out.push_str("\\\\"),
            c if c == delimiter => {
                out.push('\\');
                out.push(c);
            }
            other => out.push(other),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{canonicalize_value, emit_output};
    use crate::format::OutputFormat;

    #[test]
    fn canonicalizes_object_order() {
        let value = json!({"b": 1, "a": {"d": 1, "c": 2}});
        let rendered = serde_json::to_string(&canonicalize_value(value)).unwrap();
        assert_eq!(rendered, r#"{"a":{"c":2,"d":1},"b":1}"#);
    }

    #[test]
    fn emits_tsv() {
        let value = json!([{"id": 1, "name": "alice"}]);
        let output = emit_output(OutputFormat::Tsv, &value).unwrap();
        assert_eq!(output, "id\tname\n1\talice\n");
    }

    #[test]
    fn emits_toon() {
        let value = json!({"users": [{"id": 1, "name": "alice"}]});
        let output = emit_output(OutputFormat::Toon, &value).unwrap();
        assert_eq!(output, "users[1]{id,name}:\n  1,alice");
    }

    #[test]
    fn wraps_output_with_trailing_newline() {
        let wrapped = super::wrap_output(OutputFormat::Tsv, "id\tname\n1\talice\n");
        assert_eq!(wrapped, "```tsv\nid\tname\n1\talice\n```\n");
    }

    #[test]
    fn escapes_special_characters_in_tsv_cells() {
        let value = json!([{"id": 1, "text": "hello\tworld\nnext"}]);
        let output = emit_output(OutputFormat::Tsv, &value).unwrap();
        assert_eq!(output, "id\ttext\n1\thello\\tworld\\nnext\n");
    }

    #[test]
    fn supports_single_key_object_table_for_tsv() {
        let value = json!({"users": [{"id": 1, "name": "alice"}]});
        let output = emit_output(OutputFormat::Tsv, &value).unwrap();
        assert_eq!(output, "id\tname\n1\talice\n");
    }
}
