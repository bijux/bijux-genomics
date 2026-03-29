use std::path::Path;

use anyhow::{anyhow, Result};
use clap::Parser;

use crate::commands::cli;

fn global_option_value_arity(flag: &str) -> Option<usize> {
    match flag {
        "--log-level" | "--profile" | "--platform" | "--telemetry-jsonl" => Some(1),
        _ => None,
    }
}

fn is_global_switch(flag: &str) -> bool {
    matches!(
        flag,
        "-v" | "--verbose"
            | "-q"
            | "--quiet"
            | "--print-effective-config"
            | "--dump-effective-config"
            | "--json"
    )
}

fn normalize_cli_argv(argv: &[String]) -> Vec<String> {
    let raw = match argv.first() {
        Some(first) => {
            let executable = Path::new(first)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or(first.as_str());
            match executable {
                "bijux" | "bijux-dna" => &argv[1..],
                _ => argv,
            }
        }
        None => argv,
    };

    let mut normalized = vec!["bijux-dna".to_string()];
    let mut index = 0usize;
    while index < raw.len() {
        let token = raw[index].as_str();
        if let Some(value_arity) = global_option_value_arity(token) {
            normalized.push(raw[index].clone());
            for offset in 1..=value_arity {
                if let Some(value) = raw.get(index + offset) {
                    normalized.push(value.clone());
                }
            }
            index = (index + 1 + value_arity).min(raw.len());
            continue;
        }
        if is_global_switch(token) {
            normalized.push(raw[index].clone());
            index += 1;
            continue;
        }
        break;
    }

    if raw.get(index).is_some_and(|token| token == "dna") {
        index += 1;
    }

    normalized.extend(raw[index..].iter().cloned());
    normalized
}

/// # Errors
/// Returns an error if the provided argv cannot be parsed into the direct DNA CLI surface.
pub fn parse_cli_from_argv(argv: &[String]) -> Result<cli::Cli> {
    let normalized = normalize_cli_argv(argv);
    cli::Cli::try_parse_from(normalized).map_err(|err| anyhow!("{err}"))
}

#[must_use]
pub fn parse_process_cli(argv: &[String]) -> cli::Cli {
    let normalized = normalize_cli_argv(argv);
    cli::Cli::parse_from(normalized)
}
