//! Owner: bijux-dna-bench
//! Input and mount closure for exact FASTQ read-length attempts.

use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkExecutionInput, BenchmarkExecutionInputKind, FastqReadLengthExpectationSample,
};

use crate::FastqReadLengthExpectationPublication;

#[derive(Debug, Clone)]
pub(super) struct CommandSample {
    pub sample_id: String,
    pub input_layout: &'static str,
    pub mounted_fastq: Vec<String>,
    pub expected_summary_mount: String,
    pub expected_histogram_mount: String,
    pub expected_exact_sha256: String,
}

pub(super) fn execution_inputs(
    publication: &FastqReadLengthExpectationPublication,
) -> Result<(Vec<BenchmarkExecutionInput>, Vec<CommandSample>)> {
    let mut inputs = Vec::new();
    let mut ids = BTreeSet::new();
    let mut mounts = BTreeSet::new();
    for sample in &publication.admitted_corpus.samples {
        for file in &sample.files {
            let name = file_name(&file.path)?;
            push_input(
                &mut inputs,
                &mut ids,
                &mut mounts,
                normalized_id("fastq", &name),
                BenchmarkExecutionInputKind::Data,
                file.path.clone(),
                format!("/input/{name}"),
                file.sha256.clone(),
            )?;
        }
    }
    push_input(
        &mut inputs,
        &mut ids,
        &mut mounts,
        "corpus_data_index".to_string(),
        BenchmarkExecutionInputKind::Data,
        publication.admitted_corpus.data_index_path.clone(),
        "/input/data-index.json".to_string(),
        publication.admitted_corpus.data_index_sha256.clone(),
    )?;
    push_input(
        &mut inputs,
        &mut ids,
        &mut mounts,
        "corpus_catalog".to_string(),
        BenchmarkExecutionInputKind::Data,
        publication.admitted_corpus.catalog_path.clone(),
        "/input/corpus-catalog.json".to_string(),
        publication.admitted_corpus.catalog_sha256.clone(),
    )?;
    push_input(
        &mut inputs,
        &mut ids,
        &mut mounts,
        "read_length_manifest".to_string(),
        BenchmarkExecutionInputKind::ExpectedEvidence,
        path_text(&publication.manifest_path)?.to_string(),
        "/input/read-length-manifest.json".to_string(),
        bijux_dna_infra::hash_file_sha256(&publication.manifest_path)?,
    )?;

    let mut command_samples = Vec::new();
    for (admitted, expected) in
        publication.admitted_corpus.samples.iter().zip(&publication.expectations.samples)
    {
        if admitted.sample_id != expected.sample_id {
            bail!("admitted corpus and expectation sample closures differ");
        }
        let summary_mount = format!("/input/{}.summary.tsv", expected.sample_id);
        let histogram_mount = format!("/input/{}.histogram.tsv", expected.sample_id);
        add_expectation_input(
            &mut inputs,
            &mut ids,
            &mut mounts,
            expected,
            &publication.root,
            true,
            &summary_mount,
        )?;
        add_expectation_input(
            &mut inputs,
            &mut ids,
            &mut mounts,
            expected,
            &publication.root,
            false,
            &histogram_mount,
        )?;
        command_samples.push(CommandSample {
            sample_id: admitted.sample_id.clone(),
            input_layout: match admitted.input_layout {
                bijux_dna_bench_model::BenchmarkFastqLayout::SingleEnd => "single_end",
                bijux_dna_bench_model::BenchmarkFastqLayout::PairedEnd => "paired_end",
            },
            mounted_fastq: admitted
                .files
                .iter()
                .map(|file| file_name(&file.path).map(|name| format!("/input/{name}")))
                .collect::<Result<Vec<_>>>()?,
            expected_summary_mount: summary_mount,
            expected_histogram_mount: histogram_mount,
            expected_exact_sha256: expected.exact_rows_sha256.clone(),
        });
    }
    Ok((inputs, command_samples))
}

fn add_expectation_input(
    inputs: &mut Vec<BenchmarkExecutionInput>,
    ids: &mut BTreeSet<String>,
    mounts: &mut BTreeSet<String>,
    expected: &FastqReadLengthExpectationSample,
    root: &Path,
    summary: bool,
    mount: &str,
) -> Result<()> {
    let (label, relative, digest) = if summary {
        ("summary", &expected.summary_path, &expected.summary_sha256)
    } else {
        ("histogram", &expected.histogram_path, &expected.histogram_sha256)
    };
    push_input(
        inputs,
        ids,
        mounts,
        normalized_id(&expected.sample_id, label),
        BenchmarkExecutionInputKind::ExpectedEvidence,
        path_text(&root.join(relative))?.to_string(),
        mount.to_string(),
        digest.clone(),
    )
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
        bail!("FASTQ execution input identity or mount is duplicated");
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
        .context("FASTQ execution input lacks a UTF-8 file name")
}

fn path_text(path: &Path) -> Result<&str> {
    path.to_str().context("FASTQ execution path is not UTF-8")
}
