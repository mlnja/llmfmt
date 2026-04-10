use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;

use clap::Parser;
use llmfmt::cli::{Cli, StatsMode};
use llmfmt::error::Result;
use llmfmt::io::read_input;
use llmfmt::{ConvertOptions, convert};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let input = read_input(cli.input.as_deref())?;
    let result = convert(
        &input,
        cli.input.as_deref(),
        &ConvertOptions {
            input_format: cli.input_format,
            output_format: cli.output_format,
            profile: cli.profile.clone(),
            wrap: cli.wrap,
        },
    )?;

    match cli.stats {
        StatsMode::Text => eprintln!("{}", result.stats.render_text()),
        StatsMode::Json => eprintln!("{}", result.stats.render_json()?),
        StatsMode::Off => {}
    }

    if let Some(path) = cli.output.as_deref() {
        fs::write(path, result.payload.as_bytes())?;
    } else {
        let mut stdout = io::stdout().lock();
        stdout.write_all(result.payload.as_bytes())?;
        stdout.flush()?;
    }

    Ok(())
}
