//! Input reading, format detection, and parsing into `serde_json::Value`.

use std::fs;
use std::io::{self, Read};
use std::path::Path;

use csv::StringRecord;
use serde_json::{Map, Number, Value};

use crate::error::{LlmFmtError, Result};
use crate::format::InputFormat;

pub fn read_input(path: Option<&Path>) -> Result<String> {
    match path {
        Some(path) => Ok(fs::read_to_string(path)?),
        None => {
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            if buffer.trim().is_empty() {
                return Err(LlmFmtError::Message(
                    "no input provided; pass a file path or pipe data via stdin".to_owned(),
                ));
            }
            Ok(buffer)
        }
    }
}

pub fn detect_input_format(path: Option<&Path>, input: &str) -> Result<InputFormat> {
    if let Some(path) = path
        && let Some(ext) = path.extension().and_then(|ext| ext.to_str())
    {
        match ext {
            "json" => return Ok(InputFormat::Json),
            "yaml" | "yml" => return Ok(InputFormat::Yaml),
            "toml" => return Ok(InputFormat::Toml),
            "csv" | "tsv" => return Ok(InputFormat::Csv),
            "toon" => return Ok(InputFormat::Toon),
            _ => {}
        }
    }

    let trimmed = input.trim_start();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return Ok(InputFormat::Json);
    }
    if looks_like_toon(trimmed) {
        return Ok(InputFormat::Toon);
    }
    if looks_like_delimited(trimmed) {
        return Ok(InputFormat::Csv);
    }
    if trimmed.starts_with("---") || looks_like_yaml(trimmed) {
        return Ok(InputFormat::Yaml);
    }
    if toml::from_str::<toml::Value>(trimmed).is_ok() {
        return Ok(InputFormat::Toml);
    }

    Err(LlmFmtError::Message(
        "could not detect input format; use --input-format".to_owned(),
    ))
}

pub fn parse_input(format: InputFormat, input: &str) -> Result<Value> {
    match format {
        InputFormat::Json => Ok(serde_json::from_str(input)?),
        InputFormat::Yaml => Ok(serde_yaml::from_str(input)?),
        InputFormat::Toml => Ok(toml_to_json(toml::from_str::<toml::Value>(input)?)),
        InputFormat::Csv => parse_csv(input),
        InputFormat::Toon => Ok(toon_format::decode_default::<Value>(input)?),
    }
}

fn parse_csv(input: &str) -> Result<Value> {
    let delimiter = if input.lines().next().is_some_and(|line| line.contains('\t')) {
        b'\t'
    } else {
        b','
    };

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .from_reader(input.as_bytes());
    let headers = reader.headers()?.clone();

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record?;
        rows.push(record_to_value(&headers, &record));
    }
    Ok(Value::Array(rows))
}

fn record_to_value(headers: &StringRecord, record: &StringRecord) -> Value {
    let mut map = Map::new();
    for (header, value) in headers.iter().zip(record.iter()) {
        map.insert(header.to_owned(), decode_scalar(value.to_owned()));
    }
    Value::Object(map)
}

fn decode_scalar(raw: String) -> Value {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Value::Null;
    }
    if trimmed.eq_ignore_ascii_case("null") {
        return Value::Null;
    }
    if trimmed.eq_ignore_ascii_case("true") {
        return Value::Bool(true);
    }
    if trimmed.eq_ignore_ascii_case("false") {
        return Value::Bool(false);
    }
    if let Ok(number) = trimmed.parse::<i64>() {
        return Value::Number(number.into());
    }
    if let Ok(number) = trimmed.parse::<u64>() {
        return Value::Number(number.into());
    }
    if let Ok(number) = trimmed.parse::<f64>()
        && let Some(number) = Number::from_f64(number)
    {
        return Value::Number(number);
    }
    Value::String(trimmed.to_owned())
}

fn toml_to_json(value: toml::Value) -> Value {
    match value {
        toml::Value::String(value) => Value::String(value),
        toml::Value::Integer(value) => Value::Number(value.into()),
        toml::Value::Float(value) => Number::from_f64(value)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        toml::Value::Boolean(value) => Value::Bool(value),
        toml::Value::Datetime(value) => Value::String(value.to_string()),
        toml::Value::Array(values) => Value::Array(values.into_iter().map(toml_to_json).collect()),
        toml::Value::Table(table) => {
            let mut map = Map::new();
            for (key, value) in table {
                map.insert(key, toml_to_json(value));
            }
            Value::Object(map)
        }
    }
}

fn looks_like_toon(input: &str) -> bool {
    let first = input
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("");
    first.contains('[') && first.contains(':') && !first.contains('\t')
}

fn looks_like_delimited(input: &str) -> bool {
    let mut lines = input.lines().filter(|line| !line.trim().is_empty());
    let Some(first) = lines.next() else {
        return false;
    };
    let delimiter = if first.contains('\t') {
        '\t'
    } else if first.contains(',') {
        ','
    } else {
        return false;
    };
    lines.next().is_some_and(|line| line.contains(delimiter))
}

fn looks_like_yaml(input: &str) -> bool {
    let first = input
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("");
    if !first.contains(':') {
        return false;
    }
    serde_yaml::from_str::<Value>(input).is_ok()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::{detect_input_format, parse_input};
    use crate::format::InputFormat;

    #[test]
    fn detects_tsv_as_csv_input() {
        let input = "id\tname\n1\talice\n";
        assert_eq!(detect_input_format(None, input).unwrap(), InputFormat::Csv);
    }

    #[test]
    fn prefers_toon_over_comma_detection() {
        let input = "users[2]{id,name}:\n  1,Alice\n  2,Bob\n";
        assert_eq!(detect_input_format(None, input).unwrap(), InputFormat::Toon);
    }

    #[test]
    fn detects_formats_from_extensions() {
        assert_eq!(
            detect_input_format(Some(Path::new("data.toml")), "ignored").unwrap(),
            InputFormat::Toml
        );
        assert_eq!(
            detect_input_format(Some(Path::new("data.toon")), "ignored").unwrap(),
            InputFormat::Toon
        );
    }

    #[test]
    fn parses_yaml() {
        let value = parse_input(InputFormat::Yaml, "name: alice\nactive: true\n").unwrap();
        assert_eq!(value, json!({"name": "alice", "active": true}));
    }

    #[test]
    fn parses_toml() {
        let value = parse_input(InputFormat::Toml, "name = 'alice'\ncount = 2\n").unwrap();
        assert_eq!(value, json!({"name": "alice", "count": 2}));
    }

    #[test]
    fn csv_empty_cells_become_null() {
        let value = parse_input(InputFormat::Csv, "id,name\n1,\n").unwrap();
        assert_eq!(value, json!([{ "id": 1, "name": null }]));
    }

    #[test]
    fn unknown_input_format_returns_error() {
        let err = detect_input_format(None, "not actually structured data").unwrap_err();
        assert!(err.to_string().contains("could not detect input format"));
    }

    #[test]
    fn parses_csv_rows() {
        let value = parse_input(InputFormat::Csv, "id,name\n1,alice\n").unwrap();
        assert_eq!(value, json!([{ "id": 1, "name": "alice" }]));
    }

    #[test]
    fn parses_simple_toon() {
        let input = "users[2]{id,name}:\n  1,Alice\n  2,Bob\n";
        let value = parse_input(InputFormat::Toon, input).unwrap();
        assert_eq!(
            value,
            json!({"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]})
        );
    }
}
