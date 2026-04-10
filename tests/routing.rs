use llmfmt::format::{InputFormat, OutputFormat};
use llmfmt::{ConvertOptions, convert};

#[test]
fn auto_routes_dense_array_to_tsv() {
    let result = convert(
        r#"[{"id":1,"name":"alice"},{"id":2,"name":"bob"}]"#,
        None,
        &ConvertOptions::default(),
    )
    .unwrap();

    assert_eq!(result.input_format, InputFormat::Json);
    assert_eq!(result.output_format, OutputFormat::Tsv);
    assert_eq!(result.payload, "id\tname\n1\talice\n2\tbob\n");
    assert_eq!(result.rendered, "id\tname\n1\talice\n2\tbob\n");
}

#[test]
fn auto_routes_nested_object_to_yaml() {
    let result = convert(
        r#"{"user":{"id":1,"name":"alice"}}"#,
        None,
        &ConvertOptions::default(),
    )
    .unwrap();

    assert_eq!(result.output_format, OutputFormat::Yaml);
    assert!(result.payload.contains("user:"));
    assert!(result.payload.contains("id: 1"));
}

#[test]
fn auto_routes_deep_structure_to_compact_json() {
    let result = convert(
        r#"{"a":{"b":{"c":{"d":{"e":1}}}}}"#,
        None,
        &ConvertOptions::default(),
    )
    .unwrap();

    assert_eq!(result.output_format, OutputFormat::JsonCompact);
    assert_eq!(result.payload, r#"{"a":{"b":{"c":{"d":{"e":1}}}}}"#);
}

#[test]
fn forced_output_format_overrides_auto_routing() {
    let result = convert(
        r#"[{"id":1,"name":"alice"},{"id":2,"name":"bob"}]"#,
        None,
        &ConvertOptions {
            output_format: Some(OutputFormat::Toon),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.output_format, OutputFormat::Toon);
    assert!(result.payload.starts_with("[2]{id,name}:"));
}

#[test]
fn latest_profile_matches_explicit_profile() {
    let input = r#"[{"id":1,"name":"alice"},{"id":2,"name":"bob"}]"#;
    let latest = convert(input, None, &ConvertOptions::default()).unwrap();
    let explicit = convert(
        input,
        None,
        &ConvertOptions {
            profile: Some("20260411".to_owned()),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(latest.output_format, explicit.output_format);
    assert_eq!(latest.payload, explicit.payload);
}

#[test]
fn new_profile_routes_nested_user_records_to_yaml() {
    let input = r#"[
        {
            "id":1,
            "name":"Leanne Graham",
            "address":{"city":"Gwenborough","geo":{"lat":"-37.3159","lng":"81.1496"}},
            "company":{"name":"Romaguera-Crona","bs":"harness real-time e-markets"}
        },
        {
            "id":2,
            "name":"Ervin Howell",
            "address":{"city":"Wisokyburgh","geo":{"lat":"-43.9509","lng":"-34.4618"}},
            "company":{"name":"Deckow-Crist","bs":"synergize scalable supply-chains"}
        }
    ]"#;
    let result = convert(input, None, &ConvertOptions::default()).unwrap();
    assert_eq!(result.output_format, OutputFormat::Yaml);
}

#[test]
fn old_profile_keeps_previous_nested_user_record_behavior() {
    let input = r#"[
        {
            "id":1,
            "name":"Leanne Graham",
            "address":{"city":"Gwenborough","geo":{"lat":"-37.3159","lng":"81.1496"}},
            "company":{"name":"Romaguera-Crona","bs":"harness real-time e-markets"}
        },
        {
            "id":2,
            "name":"Ervin Howell",
            "address":{"city":"Wisokyburgh","geo":{"lat":"-43.9509","lng":"-34.4618"}},
            "company":{"name":"Deckow-Crist","bs":"synergize scalable supply-chains"}
        }
    ]"#;
    let result = convert(
        input,
        None,
        &ConvertOptions {
            profile: Some("20260410".to_owned()),
            ..ConvertOptions::default()
        },
    )
    .unwrap();
    assert_eq!(result.output_format, OutputFormat::Toon);
}
