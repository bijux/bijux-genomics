use crate::source::{
    validate_fastq_corpus_source_spec, FastqCorpusSourceSpec, FastqCorpusSourceSpec as CorpusSpec,
};
use anyhow::{anyhow, Context, Result};
use std::path::Path;
const MAX_CORPUS_SPEC_BYTES: usize = 4 * 1024 * 1024;
/// Load and validate a bounded UTF-8 source declaration.
///
/// # Errors
/// Rejects unreadable, malformed or semantically invalid source declarations.
pub fn load_fastq_corpus_source_spec(path: &Path) -> Result<FastqCorpusSourceSpec> {
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_CORPUS_SPEC_BYTES)
        .with_context(|| format!("read corpus specification {}", path.display()))?;
    let spec: FastqCorpusSourceSpec =
        toml::from_str(std::str::from_utf8(&bytes).context("corpus specification is not UTF-8")?)
            .with_context(|| format!("parse corpus specification {}", path.display()))?;
    validate_fastq_corpus_source_spec(&spec).context("validate FASTQ corpus source")?;
    Ok(spec)
}

/// Load an admitted source declaration for the requested corpus.
///
/// # Errors
/// Rejects source or corpus identity mismatch.
pub fn load_fastq_corpus_source_for_id(path: &Path, corpus_id: &str) -> Result<CorpusSpec> {
    let spec = load_fastq_corpus_source_spec(path)?;
    if spec.corpus_id != corpus_id {
        return Err(anyhow!(
            "FASTQ corpus source `{}` does not match requested corpus `{corpus_id}`",
            spec.corpus_id
        ));
    }
    Ok(spec)
}
