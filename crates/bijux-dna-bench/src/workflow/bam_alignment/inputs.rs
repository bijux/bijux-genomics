//! Owner: bijux-dna-bench
//! Exact mount closure for controlled alignment truth campaigns.

use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    AdmittedAlignmentTruthSet, AdmittedFastqCorpus, AdmittedReferenceBundle,
    BenchmarkExecutionInput, BenchmarkExecutionInputKind, BenchmarkFastqLayout,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AlignmentCommandSample {
    pub(super) sample_id: String,
    pub(super) layout: BenchmarkFastqLayout,
    pub(super) fastq_mounts: Vec<String>,
    pub(super) fastq_input_bytes: u64,
    pub(super) truth_mount: String,
    pub(super) truth_input_bytes: u64,
}

pub(super) fn alignment_execution_inputs(
    corpus: &AdmittedFastqCorpus,
    reference: &AdmittedReferenceBundle,
    truth: &AdmittedAlignmentTruthSet,
) -> Result<(Vec<BenchmarkExecutionInput>, Vec<AlignmentCommandSample>)> {
    if corpus.data_index_sha256 != reference.data_index_sha256
        || corpus.data_index_sha256 != truth.data_index_sha256
        || corpus.data_index_path != reference.data_index_path
        || corpus.data_index_path != truth.data_index_path
        || corpus.catalog_path != truth.catalog_path
        || corpus.catalog_sha256 != truth.catalog_sha256
    {
        bail!("alignment corpus, reference, and truth do not share one admitted authority");
    }
    let truth_by_sample = truth
        .samples
        .iter()
        .map(|sample| (sample.sample_id.as_str(), &sample.truth))
        .collect::<BTreeMap<_, _>>();
    let mut inputs = Vec::new();
    let mut ids = BTreeSet::new();
    let mut mounts = BTreeSet::new();
    let mut command_samples = Vec::new();
    for sample in &corpus.samples {
        let normalized = normalized_id(&sample.sample_id);
        let truth_file = truth_by_sample
            .get(sample.sample_id.as_str())
            .with_context(|| format!("alignment truth omits sample {}", sample.sample_id))?;
        let mut fastq_mounts = Vec::new();
        let mut fastq_input_bytes = 0_u64;
        for (index, file) in sample.files.iter().enumerate() {
            let suffix = file.mate.map_or_else(|| "reads".to_string(), |mate| format!("R{mate}"));
            let mount = format!("/input/panel/{normalized}.{suffix}.fastq.gz");
            push(
                (&mut inputs, &mut ids, &mut mounts),
                format!("fastq_{normalized}_{index}"),
                BenchmarkExecutionInputKind::Data,
                &file.path,
                &mount,
                &file.sha256,
            )?;
            fastq_mounts.push(mount);
            fastq_input_bytes = fastq_input_bytes
                .checked_add(file.size_bytes)
                .context("alignment FASTQ size overflow")?;
        }
        let truth_mount = format!("/input/truth/{normalized}.tsv");
        push(
            (&mut inputs, &mut ids, &mut mounts),
            format!("truth_{normalized}"),
            BenchmarkExecutionInputKind::ExpectedEvidence,
            &truth_file.path,
            &truth_mount,
            &truth_file.sha256,
        )?;
        command_samples.push(AlignmentCommandSample {
            sample_id: sample.sample_id.clone(),
            layout: sample.input_layout,
            fastq_mounts,
            fastq_input_bytes,
            truth_mount,
            truth_input_bytes: truth_file.size_bytes,
        });
    }
    if command_samples.len() != usize::try_from(corpus.sample_count)?
        || command_samples.len() != usize::try_from(truth.sample_count)?
        || command_samples.iter().map(|sample| sample.sample_id.as_str()).collect::<BTreeSet<_>>()
            != truth_by_sample.keys().copied().collect()
    {
        bail!("alignment sample closure differs between FASTQ and truth");
    }
    push(
        (&mut inputs, &mut ids, &mut mounts),
        "corpus_data_index".to_string(),
        BenchmarkExecutionInputKind::Data,
        &corpus.data_index_path,
        "/input/data-index.json",
        &corpus.data_index_sha256,
    )?;
    for (id, kind, path, mount, sha256) in [
        (
            "corpus_catalog",
            BenchmarkExecutionInputKind::Data,
            corpus.catalog_path.as_str(),
            "/input/corpus-catalog.json",
            corpus.catalog_sha256.as_str(),
        ),
        (
            "reference_fasta",
            BenchmarkExecutionInputKind::Reference,
            reference.fasta.path.as_str(),
            "/input/reference/reference.fasta",
            reference.fasta.sha256.as_str(),
        ),
        (
            "reference_fasta_index",
            BenchmarkExecutionInputKind::Reference,
            reference.fasta_index.path.as_str(),
            "/input/reference/reference.fasta.fai",
            reference.fasta_index.sha256.as_str(),
        ),
        (
            "reference_sequence_dictionary",
            BenchmarkExecutionInputKind::Reference,
            reference.sequence_dictionary.path.as_str(),
            "/input/reference/reference.dict",
            reference.sequence_dictionary.sha256.as_str(),
        ),
        (
            "reference_catalog",
            BenchmarkExecutionInputKind::Reference,
            reference.catalog_path.as_str(),
            "/input/reference-catalog.json",
            reference.catalog_sha256.as_str(),
        ),
    ] {
        push((&mut inputs, &mut ids, &mut mounts), id.to_string(), kind, path, mount, sha256)?;
    }
    Ok((inputs, command_samples))
}

fn push(
    state: (&mut Vec<BenchmarkExecutionInput>, &mut BTreeSet<String>, &mut BTreeSet<String>),
    id: String,
    kind: BenchmarkExecutionInputKind,
    path: &str,
    mount_path: &str,
    sha256: &str,
) -> Result<()> {
    let (inputs, ids, mounts) = state;
    if !ids.insert(id.clone()) || !mounts.insert(mount_path.to_string()) {
        bail!("alignment input identity or mount is duplicated");
    }
    inputs.push(BenchmarkExecutionInput {
        id,
        kind,
        path: path.to_string(),
        mount_path: mount_path.to_string(),
        sha256: sha256.to_string(),
    });
    Ok(())
}

pub(super) fn normalized_id(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() { character } else { '_' })
        .collect()
}
