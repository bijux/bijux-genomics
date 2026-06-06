use std::path::Path;

use anyhow::{anyhow, Result};

use crate::commands::benchmark::local_corpus_fixture::vcf::{
    validate_vcf_corpus_fixture_manifest_path, DEFAULT_VCF_MINI_MANIFEST_PATH,
};
use crate::commands::cli;

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
