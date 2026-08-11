//! Owner: bijux-dna-bench
//! Immutable common coverage denominator derived from an admitted BAM header.

use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    AdmittedBamCorpus, BenchmarkExecutionInput, BenchmarkExecutionInputKind,
};

const MOUNT_PATH: &str = "/input/reference-span.tsv";

pub(super) fn ensure_reference_span_input(
    attempt_directory: &Path,
    corpus: &AdmittedBamCorpus,
) -> Result<BenchmarkExecutionInput> {
    let reference_bases =
        bijux_dna_domain_bam::reference::bam_header_reference_span(Path::new(&corpus.bam.path))?;
    let bytes = format!(
        "schema_version\tbijux.benchmark.bam.reference-span.v1\nsource_bam_sha256\t{}\nreference_bases\t{reference_bases}\n",
        corpus.bam.sha256
    )
    .into_bytes();
    let path = attempt_directory.join("reference-span.tsv");
    ensure_bytes(&path, &bytes)?;
    Ok(BenchmarkExecutionInput {
        id: "bam_header_reference_span".to_string(),
        kind: BenchmarkExecutionInputKind::ExpectedEvidence,
        path: path.display().to_string(),
        mount_path: MOUNT_PATH.to_string(),
        sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
    })
}

fn ensure_bytes(path: &Path, expected: &[u8]) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("BAM reference-span evidence is not a regular file: {}", path.display());
            }
            if std::fs::read(path)? != expected {
                bail!(
                    "BAM reference-span evidence conflicts with immutable bytes: {}",
                    path.display()
                );
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, expected)?;
            Ok(())
        }
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}
