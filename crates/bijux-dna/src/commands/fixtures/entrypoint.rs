use std::path::Path;

use anyhow::{anyhow, Result};

use crate::commands::benchmark::local_corpus_fixture::vcf::{
    validate_vcf_corpus_fixture_manifest_path, DEFAULT_VCF_MINI_MANIFEST_PATH,
};
use crate::commands::fixtures::build::vcf::{
    build_vcf_mini_fixture, DEFAULT_VCF_MINI_REGENERATION_ROOT,
};
use crate::commands::fixtures::expected::vcf::validate_vcf_expected_truth;
use crate::commands::cli;

/// Build a governed local fixture corpus by corpus id.
///
/// # Errors
/// Returns an error if the requested corpus id is unsupported or if generation,
/// validation, or count matching fails.
pub(crate) fn build_fixture(cwd: &Path, args: &cli::FixturesBuildArgs) -> Result<()> {
    match args.corpus.as_str() {
        "vcf-mini" => {
            let output_root = args
                .out
                .as_ref()
                .map_or_else(|| cwd.join(DEFAULT_VCF_MINI_REGENERATION_ROOT), |path| {
                    if path.is_absolute() {
                        path.clone()
                    } else {
                        cwd.join(path)
                    }
                });
            let report = build_vcf_mini_fixture(cwd, &output_root)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.report_path);
            }
            Ok(())
        }
        _ => Err(anyhow!(
            "unsupported governed fixture corpus `{}`",
            args.corpus
        )),
    }
}

/// Validate a governed local fixture corpus by corpus id.
///
/// # Errors
/// Returns an error if the requested corpus id is unsupported or its governed
/// fixture contract fails validation.
pub(crate) fn validate_fixture(cwd: &Path, args: &cli::FixturesValidateArgs) -> Result<()> {
    match args.corpus.as_str() {
        "vcf-mini" => {
            let manifest_path = cwd.join(DEFAULT_VCF_MINI_MANIFEST_PATH);
            let report = validate_vcf_corpus_fixture_manifest_path(cwd, &manifest_path)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.manifest_path);
            }
            Ok(())
        }
        _ => Err(anyhow!(
            "unsupported governed fixture corpus `{}`",
            args.corpus
        )),
    }
}

/// Validate a governed expected-truth bundle by corpus id.
///
/// # Errors
/// Returns an error if the requested corpus id is unsupported or its governed
/// expected-truth contract fails validation.
pub(crate) fn validate_expected_fixture(
    cwd: &Path,
    args: &cli::FixturesValidateExpectedArgs,
) -> Result<()> {
    match args.corpus.as_str() {
        "vcf-mini" => {
            let report = validate_vcf_expected_truth(cwd)?;
            if args.json {
                cli::render::json::print_pretty(&report)?;
            } else {
                println!("{}", report.expected_dir);
            }
            Ok(())
        }
        _ => Err(anyhow!(
            "unsupported governed expected-truth corpus `{}`",
            args.corpus
        )),
    }
}
