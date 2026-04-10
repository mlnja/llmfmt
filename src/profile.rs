//! Frozen routing profiles.

use crate::analysis::DataAnalysis;
use crate::error::{LlmFmtError, Result};
use crate::format::OutputFormat;

/// A versioned decision function from analyzed data shape to output format.
pub trait Profile {
    fn id(&self) -> &'static str;
    fn select_format(&self, analysis: &DataAnalysis) -> OutputFormat;
}

pub struct Profile20260410;
pub struct Profile20260411;

impl Profile for Profile20260410 {
    fn id(&self) -> &'static str {
        "20260410"
    }

    fn select_format(&self, analysis: &DataAnalysis) -> OutputFormat {
        if analysis.is_flat_object_array && analysis.row_count > 0 {
            if analysis.field_count <= 3 && analysis.sparsity <= 0.05 {
                OutputFormat::Tsv
            } else {
                OutputFormat::Toon
            }
        } else if analysis.is_uniform_object_array
            && analysis.uniformity >= 0.9
            && analysis.sparsity <= 0.35
        {
            OutputFormat::Toon
        } else if !analysis.is_deeply_nested && analysis.depth <= 4 {
            OutputFormat::Yaml
        } else {
            OutputFormat::JsonCompact
        }
    }
}

static PROFILE_20260410: Profile20260410 = Profile20260410;
static PROFILE_20260411: Profile20260411 = Profile20260411;

impl Profile for Profile20260411 {
    fn id(&self) -> &'static str {
        "20260411"
    }

    fn select_format(&self, analysis: &DataAnalysis) -> OutputFormat {
        if analysis.is_flat_object_array && analysis.row_count > 0 {
            if analysis.field_count <= 3 && analysis.sparsity <= 0.05 {
                OutputFormat::Tsv
            } else {
                OutputFormat::Toon
            }
        } else if analysis.is_uniform_object_array
            && analysis.uniformity >= 0.9
            && analysis.sparsity <= 0.35
            && analysis.nested_value_field_count <= analysis.scalar_field_count
        {
            OutputFormat::Toon
        } else if !analysis.is_deeply_nested && analysis.depth <= 4 {
            OutputFormat::Yaml
        } else {
            OutputFormat::JsonCompact
        }
    }
}

pub fn resolve_profile(id: Option<&str>) -> Result<&'static dyn Profile> {
    match id.unwrap_or("latest") {
        "latest" | "20260411" => Ok(&PROFILE_20260411),
        "20260410" => Ok(&PROFILE_20260410),
        other => Err(LlmFmtError::Message(format!(
            "unknown profile `{other}`; supported values: latest, 20260410, 20260411"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use crate::analysis::analyze;
    use serde_json::json;

    use super::{Profile, Profile20260410, Profile20260411};
    use crate::format::OutputFormat;

    #[test]
    fn routes_dense_small_table_to_tsv() {
        let profile = Profile20260410;
        let analysis = analyze(&json!([{"id": 1, "name": "a"}, {"id": 2, "name": "b"}]));
        assert_eq!(profile.select_format(&analysis), OutputFormat::Tsv);
    }

    #[test]
    fn routes_wider_table_to_toon() {
        let profile = Profile20260410;
        let analysis = analyze(&json!([
            {"id": 1, "name": "a", "role": "x", "team": "red"},
            {"id": 2, "name": "b", "role": "y", "team": "blue"}
        ]));
        assert_eq!(profile.select_format(&analysis), OutputFormat::Toon);
    }

    #[test]
    fn routes_nested_object_to_yaml() {
        let profile = Profile20260410;
        let analysis = analyze(&json!({"user": {"id": 1, "name": "alice"}}));
        assert_eq!(profile.select_format(&analysis), OutputFormat::Yaml);
    }

    #[test]
    fn routes_deep_structure_to_compact_json() {
        let profile = Profile20260410;
        let analysis = analyze(&json!({"a":{"b":{"c":{"d":{"e":1}}}}}));
        assert_eq!(profile.select_format(&analysis), OutputFormat::JsonCompact);
    }

    #[test]
    fn new_profile_routes_nested_uniform_rows_away_from_toon() {
        let profile = Profile20260411;
        let analysis = analyze(&json!([
            {
                "id": 1,
                "name": "Alice",
                "address": {"city": "Gwenborough", "geo": {"lat": "1", "lng": "2"}},
                "company": {"name": "Romaguera-Crona", "bs": "harness real-time e-markets"}
            },
            {
                "id": 2,
                "name": "Bob",
                "address": {"city": "Wisokyburgh", "geo": {"lat": "3", "lng": "4"}},
                "company": {"name": "Deckow-Crist", "bs": "synergize scalable supply-chains"}
            }
        ]));
        assert_eq!(profile.select_format(&analysis), OutputFormat::Yaml);
    }

    #[test]
    fn old_profile_still_routes_nested_uniform_rows_to_toon() {
        let profile = Profile20260410;
        let analysis = analyze(&json!([
            {
                "id": 1,
                "name": "Alice",
                "address": {"city": "Gwenborough"},
                "company": {"name": "Romaguera-Crona"}
            },
            {
                "id": 2,
                "name": "Bob",
                "address": {"city": "Wisokyburgh"},
                "company": {"name": "Deckow-Crist"}
            }
        ]));
        assert_eq!(profile.select_format(&analysis), OutputFormat::Toon);
    }
}
