//! Size-estimate reporting for conversions.

use serde::Serialize;

use crate::error::Result;
use crate::format::{InputFormat, OutputFormat};

#[derive(Debug, Serialize)]
pub struct ConversionStats {
    input_format: InputFormat,
    output_format: OutputFormat,
    profile: &'static str,
    forced: bool,
    estimate_kind: &'static str,
    input_bytes: usize,
    output_bytes: usize,
    delta_percent: f32,
}

impl ConversionStats {
    pub fn new(
        input_format: InputFormat,
        output_format: OutputFormat,
        profile: &'static str,
        forced: bool,
        input_bytes: usize,
        output_bytes: usize,
    ) -> Self {
        let delta_percent = if input_bytes == 0 {
            0.0
        } else {
            ((output_bytes as f32 - input_bytes as f32) / input_bytes as f32) * 100.0
        };
        Self {
            input_format,
            output_format,
            profile,
            forced,
            estimate_kind: "bytes",
            input_bytes,
            output_bytes,
            delta_percent,
        }
    }

    pub fn render_text(&self) -> String {
        let direction = if self.delta_percent <= 0.0 { "" } else { "+" };
        let mode = if self.forced { "forced" } else { "auto" };
        format!(
            "{} | size {}B→{}B ({}{:.0}%) [{}|estimate:{}|profile:{}]",
            self.output_format.label(),
            self.input_bytes,
            self.output_bytes,
            direction,
            self.delta_percent,
            mode,
            self.estimate_kind,
            self.profile
        )
    }

    pub fn render_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::ConversionStats;
    use crate::format::{InputFormat, OutputFormat};

    #[test]
    fn renders_text_for_auto_mode() {
        let stats = ConversionStats::new(
            InputFormat::Json,
            OutputFormat::Tsv,
            "20260410",
            false,
            100,
            50,
        );
        assert_eq!(
            stats.render_text(),
            "tsv | size 100B→50B (-50%) [auto|estimate:bytes|profile:20260410]"
        );
    }

    #[test]
    fn renders_text_for_forced_growth() {
        let stats = ConversionStats::new(
            InputFormat::Json,
            OutputFormat::Yaml,
            "20260410",
            true,
            100,
            125,
        );
        assert_eq!(
            stats.render_text(),
            "yaml | size 100B→125B (+25%) [forced|estimate:bytes|profile:20260410]"
        );
    }

    #[test]
    fn renders_json_fields() {
        let stats = ConversionStats::new(
            InputFormat::Toon,
            OutputFormat::JsonCompact,
            "20260410",
            false,
            33,
            44,
        );
        let rendered = stats.render_json().unwrap();
        assert!(rendered.contains(r#""input_format":"toon""#));
        assert!(rendered.contains(r#""estimate_kind":"bytes""#));
    }
}
