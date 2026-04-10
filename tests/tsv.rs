use llmfmt::format::OutputFormat;
use llmfmt::{ConvertOptions, convert};

#[test]
fn rejects_nested_values_for_tsv() {
    let err = convert(
        r#"[{"id":1,"tags":["red","blue"]}]"#,
        None,
        &ConvertOptions {
            output_format: Some(OutputFormat::Tsv),
            ..ConvertOptions::default()
        },
    )
    .unwrap_err();

    assert!(
        err.to_string()
            .contains("tsv output requires flat scalar fields")
    );
}

#[test]
fn preserves_null_cells_in_tsv_output() {
    let result = convert(
        r#"[{"id":1,"name":null},{"id":2,"name":"bob"}]"#,
        None,
        &ConvertOptions {
            output_format: Some(OutputFormat::Tsv),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.payload, "id\tname\n1\t\n2\tbob\n");
}

#[test]
fn escapes_tabs_and_newlines_in_tsv_cells() {
    let result = convert(
        r#"[{"id":1,"text":"hello\tworld\nnext"}]"#,
        None,
        &ConvertOptions {
            output_format: Some(OutputFormat::Tsv),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.payload, "id\ttext\n1\thello\\tworld\\nnext\n");
}

#[test]
fn fills_missing_fields_with_empty_cells() {
    let result = convert(
        r#"[{"id":1,"name":"alice"},{"id":2}]"#,
        None,
        &ConvertOptions {
            output_format: Some(OutputFormat::Tsv),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.payload, "id\tname\n1\talice\n2\t\n");
}

#[test]
fn supports_single_key_object_with_array_for_tsv() {
    let result = convert(
        r#"{"users":[{"id":1,"name":"alice"}]}"#,
        None,
        &ConvertOptions {
            output_format: Some(OutputFormat::Tsv),
            ..ConvertOptions::default()
        },
    )
    .unwrap();

    assert_eq!(result.payload, "id\tname\n1\talice\n");
}
