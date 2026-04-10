use llmfmt::format::OutputFormat;
use llmfmt::{ConvertOptions, convert};

#[test]
fn wraps_forced_toon_and_emits_stats_json() {
    let result = convert(
        r#"{"users":[{"id":1,"name":"alice","role":"admin"},{"id":2,"name":"bob","role":"user"}]}"#,
        None,
        &ConvertOptions {
            output_format: Some(OutputFormat::Toon),
            wrap: true,
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.output_format, OutputFormat::Toon);
    assert!(
        result
            .payload
            .starts_with("```toon\nusers[2]{id,name,role}:")
    );
    assert!(result.payload.ends_with("```\n"));
    assert!(
        result
            .stats
            .render_json()
            .unwrap()
            .contains(r#""output_format":"toon""#)
    );
}

#[test]
fn forced_yaml_preserves_forced_stats_mode() {
    let result = convert(
        r#"{"user":{"id":1,"name":"alice"}}"#,
        None,
        &ConvertOptions {
            output_format: Some(OutputFormat::Yaml),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.output_format, OutputFormat::Yaml);
    assert!(result.payload.contains("user:"));
    assert!(
        result
            .stats
            .render_text()
            .contains("[forced|estimate:bytes|profile:20260411]")
    );
}

#[test]
fn json_stats_include_forced_flag() {
    let result = convert(
        r#"{"id":1}"#,
        None,
        &ConvertOptions {
            output_format: Some(OutputFormat::JsonCompact),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    let stats = result.stats.render_json().unwrap();
    assert!(stats.contains(r#""forced":true"#));
    assert!(stats.contains(r#""estimate_kind":"bytes""#));
}

#[test]
fn reports_unknown_profile() {
    let err = convert(
        r#"{"id":1}"#,
        None,
        &ConvertOptions {
            profile: Some("19990101".to_owned()),
            ..ConvertOptions::default()
        },
    )
    .unwrap_err();

    assert!(err.to_string().contains("unknown profile"));
}

#[test]
fn invalid_unstructured_input_returns_error() {
    let err = convert(
        "definitely not a supported structured document",
        None,
        &ConvertOptions::default(),
    )
    .unwrap_err();
    assert!(err.to_string().contains("could not detect input format"));
}

#[test]
fn unwrapped_output_matches_rendered_payload() {
    let result = convert(
        r#"{"id":1,"name":"alice"}"#,
        None,
        &ConvertOptions {
            output_format: Some(OutputFormat::JsonCompact),
            wrap: false,
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.payload, result.rendered);
}
