use std::path::Path;

use llmfmt::format::{InputFormat, OutputFormat};
use llmfmt::{ConvertOptions, convert};

#[test]
fn accepts_toon_input_via_official_decoder() {
    let result = convert(
        "users[2]{id,name}:\n  1,Alice\n  2,Bob",
        None,
        &ConvertOptions {
            output_format: Some(OutputFormat::JsonCompact),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.input_format, InputFormat::Toon);
    assert_eq!(
        result.payload,
        r#"{"users":[{"id":1,"name":"Alice"},{"id":2,"name":"Bob"}]}"#
    );
}

#[test]
fn detects_format_from_path_hint() {
    let result = convert(
        "name = 'alice'\ncount = 2\n",
        Some(Path::new("config.toml")),
        &ConvertOptions {
            output_format: Some(OutputFormat::JsonCompact),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.input_format, InputFormat::Toml);
    assert_eq!(result.payload, r#"{"count":2,"name":"alice"}"#);
}

#[test]
fn explicit_input_format_can_override_detection() {
    let result = convert(
        "id,name\n1,alice\n",
        None,
        &ConvertOptions {
            input_format: Some(InputFormat::Csv),
            output_format: Some(OutputFormat::JsonCompact),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.input_format, InputFormat::Csv);
    assert_eq!(result.payload, r#"[{"id":1,"name":"alice"}]"#);
}

#[test]
fn canonicalizes_object_keys_before_json_output() {
    let result = convert(
        r#"{"b":1,"a":{"d":1,"c":2}}"#,
        None,
        &ConvertOptions {
            output_format: Some(OutputFormat::JsonCompact),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.payload, r#"{"a":{"c":2,"d":1},"b":1}"#);
}

#[test]
fn path_hint_for_toon_file_selects_toon_parser() {
    let result = convert(
        "users[1]{id,name}:\n  1,Alice",
        Some(Path::new("users.toon")),
        &ConvertOptions {
            output_format: Some(OutputFormat::JsonCompact),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.input_format, InputFormat::Toon);
    assert_eq!(result.payload, r#"{"users":[{"id":1,"name":"Alice"}]}"#);
}
