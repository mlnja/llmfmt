//! CLI-visible input and output format enums.

use clap::ValueEnum;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum InputFormat {
    Json,
    Yaml,
    Toml,
    Csv,
    Toon,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    Toon,
    Tsv,
    Yaml,
    JsonCompact,
}

impl OutputFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::Toon => "toon",
            Self::Tsv => "tsv",
            Self::Yaml => "yaml",
            Self::JsonCompact => "json",
        }
    }
}
