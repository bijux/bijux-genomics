//! Owner: bijux-dna-bench
//! Input and mount closure for FASTQ read-profile attempts.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{BenchmarkExecutionInput, BenchmarkExecutionInputKind};

use crate::FastqProfileReadsExpectationPublication;

pub(super) fn execution_inputs(
    publication: &FastqProfileReadsExpectationPublication,
) -> Result<(Vec<BenchmarkExecutionInput>, Vec<String>)> {
    let mut inputs = Vec::new();
    let mut ids = BTreeSet::new();
    let mut mounts = BTreeSet::new();
    let mut mounted_fastq = Vec::new();
    for sample in &publication.admitted_corpus.samples {
        for file in &sample.files {
            let name = file_name(&file.path)?;
            let mount = format!("/input/{name}");
            push_input(
                &mut inputs,
                &mut ids,
                &mut mounts,
                normalized_id("fastq", &name),
                BenchmarkExecutionInputKind::Data,
                file.path.clone(),
                mount.clone(),
                file.sha256.clone(),
            )?;
            mounted_fastq.push(mount);
        }
    }
    for (id, path, mount, digest, kind) in [
        (
            "corpus_data_index",
            publication.admitted_corpus.data_index_path.as_str(),
            "/input/data-index.json",
            publication.admitted_corpus.data_index_sha256.as_str(),
            BenchmarkExecutionInputKind::Data,
        ),
        (
            "corpus_catalog",
            publication.admitted_corpus.catalog_path.as_str(),
            "/input/corpus-catalog.json",
            publication.admitted_corpus.catalog_sha256.as_str(),
            BenchmarkExecutionInputKind::Data,
        ),
        (
            "admission_profile",
            path_text(&publication.profile_path)?,
            "/input/expected-profile.tsv",
            "",
            BenchmarkExecutionInputKind::ExpectedEvidence,
        ),
        (
            "admission_provenance",
            path_text(&publication.provenance_path)?,
            "/input/expected-profile.json",
            "",
            BenchmarkExecutionInputKind::ExpectedEvidence,
        ),
    ] {
        let digest = if digest.is_empty() {
            bijux_dna_infra::hash_file_sha256(Path::new(path))?
        } else {
            digest.to_string()
        };
        push_input(
            &mut inputs,
            &mut ids,
            &mut mounts,
            id.to_string(),
            kind,
            path.to_string(),
            mount.to_string(),
            digest,
        )?;
    }
    Ok((inputs, mounted_fastq))
}

#[allow(clippy::too_many_arguments)]
fn push_input(
    inputs: &mut Vec<BenchmarkExecutionInput>,
    ids: &mut BTreeSet<String>,
    mounts: &mut BTreeSet<String>,
    id: String,
    kind: BenchmarkExecutionInputKind,
    path: String,
    mount_path: String,
    sha256: String,
) -> Result<()> {
    if !ids.insert(id.clone()) || !mounts.insert(mount_path.clone()) {
        bail!("FASTQ profile input identity or mount is duplicated");
    }
    inputs.push(BenchmarkExecutionInput { id, kind, path, mount_path, sha256 });
    Ok(())
}

fn normalized_id(prefix: &str, value: &str) -> String {
    format!("{prefix}_{value}")
        .to_ascii_lowercase()
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() { character } else { '_' })
        .collect()
}

fn file_name(path: &str) -> Result<String> {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_string)
        .context("FASTQ profile input lacks a UTF-8 file name")
}

fn path_text(path: &Path) -> Result<&str> {
    path.to_str().context("FASTQ profile evidence path is not UTF-8")
}
