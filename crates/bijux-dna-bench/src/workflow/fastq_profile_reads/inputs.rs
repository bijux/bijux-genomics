//! Owner: bijux-dna-bench
//! Shared immutable input closure for real-corpus FASTQ observational campaigns.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkExecutionInput, BenchmarkExecutionInputKind, BenchmarkFastqLayout,
};

use super::FastqProfileReadsExpectationPublication;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::workflow) struct FastqCampaignSample {
    pub sample_id: String,
    pub input_layout: &'static str,
    pub mounted_fastq: Vec<String>,
    pub expected_read_count: u64,
}

pub(in crate::workflow) fn campaign_execution_inputs(
    publication: &FastqProfileReadsExpectationPublication,
) -> Result<(Vec<BenchmarkExecutionInput>, Vec<FastqCampaignSample>)> {
    let mut inputs = Vec::new();
    let mut ids = BTreeSet::new();
    let mut mounts = BTreeSet::new();
    let mut samples = Vec::new();
    if publication.admitted_corpus.samples.len() != publication.samples.len() {
        bail!("FASTQ admission and expectation sample counts differ");
    }
    for (admitted, expected) in publication.admitted_corpus.samples.iter().zip(&publication.samples)
    {
        if admitted.sample_id != expected.sample_id
            || admitted.files.len() as u64 != expected.file_count
        {
            bail!("FASTQ admission and expectation sample identities differ");
        }
        let mut mounted_fastq = Vec::new();
        for file in &admitted.files {
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
        samples.push(FastqCampaignSample {
            sample_id: admitted.sample_id.clone(),
            input_layout: match admitted.input_layout {
                BenchmarkFastqLayout::SingleEnd => "single_end",
                BenchmarkFastqLayout::PairedEnd => "paired_end",
            },
            mounted_fastq,
            expected_read_count: expected.read_count,
        });
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
    Ok((inputs, samples))
}

pub(super) fn profile_execution_inputs(
    publication: &FastqProfileReadsExpectationPublication,
) -> Result<(Vec<BenchmarkExecutionInput>, Vec<String>)> {
    let (inputs, samples) = campaign_execution_inputs(publication)?;
    let mounted_fastq =
        samples.into_iter().flat_map(|sample| sample.mounted_fastq).collect::<Vec<_>>();
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
        bail!("FASTQ campaign input identity or mount is duplicated");
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
        .context("FASTQ campaign input lacks a UTF-8 file name")
}

fn path_text(path: &Path) -> Result<&str> {
    path.to_str().context("FASTQ campaign evidence path is not UTF-8")
}
