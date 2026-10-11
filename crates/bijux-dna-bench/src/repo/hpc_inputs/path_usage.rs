//! Owner: command argument and shell path classification for HPC source discovery.
use anyhow::{anyhow, Context, Result};
use regex::Regex;
use std::collections::BTreeSet;
use std::{path::Path, sync::LazyLock};
const OUTPUT_FLAG_ARGS: [&str; 12] = [
    "-o",
    "-O",
    "--out",
    "--output",
    "--output-dir",
    "--output-root",
    "--output-prefix",
    "--prefix",
    "--stdout",
    "--stderr",
    "--report",
    "--summary",
];
const OUTPUT_ASSIGNMENT_KEYS: [&str; 7] =
    ["out", "output", "output_dir", "output_root", "output_prefix", "report", "summary"];
const SOURCE_ROOTS: [&str; 4] = ["artifacts/", "assets/", "benchmarks/", "runs/"];

static MATCHER: LazyLock<std::result::Result<Regex, regex::Error>> = LazyLock::new(|| {
    Regex::new(
        r#"'((?:artifacts|assets|benchmarks|runs)/[^']+)'|"((?:artifacts|assets|benchmarks|runs)/[^"]+)"|((?:artifacts|assets|benchmarks|runs)/[A-Za-z0-9_./@+-]+)"#,
    )
});
#[derive(Debug, Default)]
pub(super) struct StepPathUsage {
    pub(super) consumed: BTreeSet<String>,
    pub(super) produced: BTreeSet<String>,
}

/// Return the materialized stage declared by an argument vector.
#[must_use]
pub fn materialize_stage_id(argv: &[String]) -> Option<String> {
    if !argv.iter().any(|arg| arg == "materialize-stage") {
        return None;
    }
    argv.windows(2).find(|window| window[0] == "--stage-id").map(|window| window[1].clone())
}

fn shell_step_body(argv: &[String]) -> Option<&str> {
    let [shell, flag, body, ..] = argv else {
        return None;
    };
    let shell_name = Path::new(shell).file_name()?.to_str()?;
    if !matches!(shell_name, "sh" | "bash" | "zsh") {
        return None;
    }
    if !flag.starts_with('-') || !flag.contains('c') {
        return None;
    }

    Some(body)
}

pub(super) fn collect_step_path_usage(
    argv: &[String],
    generated_output_root: &str,
) -> Result<StepPathUsage> {
    if let Some(body) = shell_step_body(argv) {
        return collect_shell_step_path_usage(body, generated_output_root);
    }
    Ok(collect_tokenized_step_path_usage(argv, generated_output_root))
}

fn collect_tokenized_step_path_usage(
    argv: &[String],
    generated_output_root: &str,
) -> StepPathUsage {
    let mut usage = StepPathUsage::default();
    let mut previous_flag: Option<&str> = None;

    for arg in argv {
        if let Some((assignment_key, path)) = assignment_repo_path(arg) {
            if is_generated_benchmark_output_path(path, generated_output_root)
                || is_output_flag(assignment_key)
                || is_output_assignment_key(assignment_key)
            {
                usage.produced.insert(path.to_string());
            } else {
                usage.consumed.insert(path.to_string());
            }
        } else if let Some(path) = candidate_repo_path(arg) {
            if is_generated_benchmark_output_path(path, generated_output_root)
                || previous_flag.is_some_and(is_output_flag)
            {
                usage.produced.insert(path.to_string());
            } else {
                usage.consumed.insert(path.to_string());
            }
        }

        previous_flag = if arg.starts_with('-') { Some(arg.as_str()) } else { None };
    }

    usage
}

fn collect_shell_step_path_usage(
    shell_command: &str,
    generated_output_root: &str,
) -> Result<StepPathUsage> {
    let matcher = MATCHER
        .as_ref()
        .map_err(|error| anyhow!("compile local HPC input-discovery path matcher: {error}"))?;
    let mut usage = StepPathUsage::default();

    for captures in matcher.captures_iter(shell_command) {
        let full_match = captures.get(0).context("shell path capture must include full match")?;
        let source_path = captures
            .get(1)
            .or_else(|| captures.get(2))
            .or_else(|| captures.get(3))
            .context("shell path capture must include a repo-relative path")?
            .as_str()
            .trim_end_matches('.');
        if !is_repo_source_path(source_path) {
            continue;
        }
        if is_generated_benchmark_output_path(source_path, generated_output_root)
            || is_output_context(shell_command, full_match.start())
        {
            usage.produced.insert(source_path.to_string());
        } else {
            usage.consumed.insert(source_path.to_string());
        }
    }

    Ok(usage)
}

fn is_output_context(body: &str, start: usize) -> bool {
    let prefix = body.get(..start).unwrap_or(body).trim_end();
    if prefix.ends_with('>') || prefix.ends_with("1>") || prefix.ends_with("2>") {
        return true;
    }

    last_shell_token(prefix).is_some_and(|token| {
        is_output_flag(token) || token.strip_suffix('=').is_some_and(is_output_assignment_key)
    })
}

fn last_shell_token(prefix: &str) -> Option<&str> {
    prefix.rsplit(|ch: char| ch.is_whitespace()).find(|segment| !segment.is_empty())
}

fn is_output_flag(flag: &str) -> bool {
    OUTPUT_FLAG_ARGS.contains(&flag)
}

fn is_output_assignment_key(key: &str) -> bool {
    OUTPUT_ASSIGNMENT_KEYS.contains(&key)
}

fn candidate_repo_path(value: &str) -> Option<&str> {
    if is_repo_source_path(value) {
        Some(value)
    } else {
        None
    }
}

fn assignment_repo_path(value: &str) -> Option<(&str, &str)> {
    let (key, path) = value.split_once('=')?;
    candidate_repo_path(path).map(|path| (key, path))
}

fn is_repo_source_path(value: &str) -> bool {
    SOURCE_ROOTS.iter().any(|root| value.starts_with(root))
}

fn is_generated_benchmark_output_path(value: &str, generated_output_root: &str) -> bool {
    value == generated_output_root
        || value.strip_prefix(generated_output_root).is_some_and(|suffix| suffix.starts_with('/'))
}

#[cfg(test)]
mod tests {
    use super::collect_shell_step_path_usage;
    use anyhow::{Context, Result};

    #[test]
    fn shell_discovery_distinguishes_materialized_inputs_from_outputs() -> Result<()> {
        let usage = collect_shell_step_path_usage(
            "cat 'artifacts/benchmark-smoke/readiness/adapters/descent/ibdne/vcf.demography/artifacts/input/ibd_segments.tsv' | ibdne map='artifacts/benchmark-smoke/readiness/adapters/descent/ibdne/vcf.demography/artifacts/input/genetic.map' out='artifacts/benchmark-smoke/readiness/adapters/descent/ibdne/vcf.demography/demography_report' > 'artifacts/benchmark-smoke/readiness/adapters/descent/ibdne/vcf.demography/demography_report.log' 2>&1",
            "artifacts/benchmark-smoke/readiness/stage-tool-commands",
        )
        .context("discover IBDNe shell paths")?;

        assert_eq!(
            usage.consumed,
            [
                "artifacts/benchmark-smoke/readiness/adapters/descent/ibdne/vcf.demography/artifacts/input/genetic.map".to_string(),
                "artifacts/benchmark-smoke/readiness/adapters/descent/ibdne/vcf.demography/artifacts/input/ibd_segments.tsv".to_string(),
            ]
            .into_iter()
            .collect()
        );
        assert_eq!(
            usage.produced,
            [
                "artifacts/benchmark-smoke/readiness/adapters/descent/ibdne/vcf.demography/demography_report".to_string(),
                "artifacts/benchmark-smoke/readiness/adapters/descent/ibdne/vcf.demography/demography_report.log".to_string(),
            ]
            .into_iter()
            .collect()
        );
        Ok(())
    }
}
