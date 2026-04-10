//! `llmfmt` is a deterministic formatter for structured data headed toward LLM prompts.
//!
//! The crate is intentionally small and opinionated:
//!
//! - parse JSON, YAML, TOML, CSV, and TOON into `serde_json::Value`
//! - canonicalize object key ordering for deterministic behavior
//! - analyze the shape of the data
//! - route through a frozen heuristic profile
//! - emit TOON, TSV, YAML, or compact JSON
//!
//! Size reporting is byte-based estimation only. The crate does not claim exact tokenizer counts.

pub mod analysis;
pub mod cli;
pub mod error;
pub mod format;
pub mod io;
pub mod output;
pub mod profile;
pub mod stats;
pub mod validation;

use std::path::Path;

use analysis::analyze;
use error::Result;
use format::{InputFormat, OutputFormat};
use io::{detect_input_format, parse_input};
use output::{emit_output, wrap_output};
use profile::resolve_profile;
use stats::ConversionStats;

/// Options for a single in-process conversion.
///
/// This is the integration-test and library-facing equivalent of the CLI flags.
#[derive(Debug, Clone)]
pub struct ConvertOptions {
    pub input_format: Option<InputFormat>,
    pub output_format: Option<OutputFormat>,
    pub profile: Option<String>,
    pub wrap: bool,
}

impl Default for ConvertOptions {
    fn default() -> Self {
        Self {
            input_format: None,
            output_format: None,
            profile: Some("latest".to_owned()),
            wrap: false,
        }
    }
}

/// Result of one conversion pass.
///
/// `rendered` is the raw formatted payload. `payload` includes code fences when `wrap=true`.
#[derive(Debug)]
pub struct ConversionResult {
    pub payload: String,
    pub rendered: String,
    pub input_format: InputFormat,
    pub output_format: OutputFormat,
    pub stats: ConversionStats,
}

/// Convert one structured document into the selected prompt-ready output format.
pub fn convert(
    input: &str,
    path_hint: Option<&Path>,
    options: &ConvertOptions,
) -> Result<ConversionResult> {
    let input_format = match options.input_format {
        Some(format) => format,
        None => detect_input_format(path_hint, input)?,
    };

    let value = parse_input(input_format, input)?;
    let canonical = output::canonicalize_value(value);
    let analysis = analyze(&canonical);
    let profile = resolve_profile(options.profile.as_deref())?;
    let forced = options.output_format.is_some();
    let output_format = options
        .output_format
        .unwrap_or_else(|| profile.select_format(&analysis));
    let rendered = emit_output(output_format, &canonical)?;
    let payload = if options.wrap {
        wrap_output(output_format, &rendered)
    } else {
        rendered.clone()
    };

    let stats = ConversionStats::new(
        input_format,
        output_format,
        profile.id(),
        forced,
        input.len(),
        rendered.len(),
    );

    Ok(ConversionResult {
        payload,
        rendered,
        input_format,
        output_format,
        stats,
    })
}
