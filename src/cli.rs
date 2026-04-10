use std::path::PathBuf;

use clap::Parser;

use crate::format::{InputFormat, OutputFormat};

#[derive(Clone, Copy, Debug, Eq, PartialEq, clap::ValueEnum)]
pub enum StatsMode {
    Text,
    Json,
    Off,
}

#[derive(Debug, Parser)]
#[command(
    name = "llmfmt",
    version,
    about = "Deterministic prompt-ready formatter for structured data"
)]
pub struct Cli {
    #[arg(value_name = "INPUT")]
    pub input: Option<PathBuf>,

    #[arg(long, value_enum)]
    pub input_format: Option<InputFormat>,

    #[arg(long, value_enum)]
    pub output_format: Option<OutputFormat>,

    #[arg(long, default_value = "latest")]
    pub profile: Option<String>,

    #[arg(long)]
    pub wrap: bool,

    #[arg(long, value_enum, default_value = "text")]
    pub stats: StatsMode,

    #[arg(short = 'o', value_name = "FILE")]
    pub output: Option<PathBuf>,
}
