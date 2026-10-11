//! Owner: bijux-dna-bench
//! Input and mount closure shared by BAM observational campaigns.

use std::collections::BTreeSet;

use anyhow::{bail, Result};
use bijux_dna_bench_model::{
    AdmittedBamCohort, AdmittedBamCorpus, AdmittedReferenceBundle, BenchmarkExecutionInput,
    BenchmarkExecutionInputKind,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::workflow) struct BamCohortCommandSample {
    pub(in crate::workflow) sample_id: String,
    pub(in crate::workflow) bam_mount: String,
    pub(in crate::workflow) bam_index_mount: String,
    pub(in crate::workflow) bam_input_bytes: u64,
    pub(in crate::workflow) bam_index_input_bytes: u64,
}

pub(in crate::workflow) fn execution_inputs(
    corpus: &AdmittedBamCorpus,
) -> Vec<BenchmarkExecutionInput> {
    vec![
        input("bam", &corpus.bam.path, "/input/ERR2112767.sorted.bam", &corpus.bam.sha256),
        input(
            "bam_index",
            &corpus.bam_index.path,
            "/input/ERR2112767.sorted.bam.bai",
            &corpus.bam_index.sha256,
        ),
        input(
            "corpus_data_index",
            &corpus.data_index_path,
            "/input/data-index.json",
            &corpus.data_index_sha256,
        ),
        input(
            "corpus_catalog",
            &corpus.catalog_path,
            "/input/corpus-catalog.json",
            &corpus.catalog_sha256,
        ),
    ]
}

pub(in crate::workflow) fn cohort_execution_inputs(
    cohort: &AdmittedBamCohort,
    reference: Option<&AdmittedReferenceBundle>,
) -> Result<(Vec<BenchmarkExecutionInput>, Vec<BamCohortCommandSample>)> {
    let sample_count = usize::try_from(cohort.sample_count)?;
    if sample_count == 0
        || sample_count != cohort.samples.len()
        || cohort.file_count != cohort.sample_count.saturating_mul(2)
    {
        bail!("BAM cohort identity counts differ from admitted samples");
    }
    if reference.is_some_and(|reference| {
        reference.data_index_path != cohort.data_index_path
            || reference.data_index_sha256 != cohort.data_index_sha256
    }) {
        bail!("BAM cohort and reference do not share one data-index authority");
    }
    let mut inputs = Vec::new();
    let mut ids = BTreeSet::new();
    let mut mounts = BTreeSet::new();
    let mut command_samples = Vec::with_capacity(sample_count);
    for sample in &cohort.samples {
        let normalized = normalized_id(&sample.sample_id);
        let bam_mount = format!("/input/cohort/{normalized}.bam");
        let bam_index_mount = format!("/input/cohort/{normalized}.bam.bai");
        push_input(
            &mut inputs,
            &mut ids,
            &mut mounts,
            format!("bam_{normalized}"),
            &sample.bam.path,
            &bam_mount,
            &sample.bam.sha256,
        )?;
        push_input(
            &mut inputs,
            &mut ids,
            &mut mounts,
            format!("bam_index_{normalized}"),
            &sample.bam_index.path,
            &bam_index_mount,
            &sample.bam_index.sha256,
        )?;
        command_samples.push(BamCohortCommandSample {
            sample_id: sample.sample_id.clone(),
            bam_mount,
            bam_index_mount,
            bam_input_bytes: sample.bam.size_bytes,
            bam_index_input_bytes: sample.bam_index.size_bytes,
        });
    }
    if command_samples.iter().map(|sample| sample.sample_id.as_str()).collect::<BTreeSet<_>>().len()
        != sample_count
    {
        bail!("BAM cohort contains duplicate sample or mount identities");
    }
    push_input(
        &mut inputs,
        &mut ids,
        &mut mounts,
        "corpus_data_index".to_string(),
        &cohort.data_index_path,
        "/input/data-index.json",
        &cohort.data_index_sha256,
    )?;
    push_input(
        &mut inputs,
        &mut ids,
        &mut mounts,
        "corpus_catalog".to_string(),
        &cohort.catalog_path,
        "/input/cohort-catalog.json",
        &cohort.catalog_sha256,
    )?;
    if let Some(reference) = reference {
        for (id, file, mount) in [
            ("reference_fasta", &reference.fasta, "/input/reference/reference.fasta"),
            (
                "reference_fasta_index",
                &reference.fasta_index,
                "/input/reference/reference.fasta.fai",
            ),
            (
                "reference_sequence_dictionary",
                &reference.sequence_dictionary,
                "/input/reference/reference.dict",
            ),
        ] {
            push_input(
                &mut inputs,
                &mut ids,
                &mut mounts,
                id.to_string(),
                &file.path,
                mount,
                &file.sha256,
            )?;
        }
        push_input(
            &mut inputs,
            &mut ids,
            &mut mounts,
            "reference_catalog".to_string(),
            &reference.catalog_path,
            "/input/reference-catalog.json",
            &reference.catalog_sha256,
        )?;
    }
    Ok((inputs, command_samples))
}

fn input(id: &str, path: &str, mount_path: &str, sha256: &str) -> BenchmarkExecutionInput {
    BenchmarkExecutionInput {
        id: id.to_string(),
        kind: BenchmarkExecutionInputKind::Data,
        path: path.to_string(),
        mount_path: mount_path.to_string(),
        sha256: sha256.to_string(),
    }
}

fn push_input(
    inputs: &mut Vec<BenchmarkExecutionInput>,
    ids: &mut BTreeSet<String>,
    mounts: &mut BTreeSet<String>,
    id: String,
    path: &str,
    mount_path: &str,
    sha256: &str,
) -> Result<()> {
    if !ids.insert(id.clone()) || !mounts.insert(mount_path.to_string()) {
        bail!("BAM cohort execution input identity or mount is duplicated");
    }
    inputs.push(input(&id, path, mount_path, sha256));
    Ok(())
}

fn normalized_id(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .map(|character| if character.is_ascii_alphanumeric() { character } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use bijux_dna_bench_model::{
        AdmittedBamCohort, AdmittedBamCohortSample, AdmittedBamCorpusFile, AdmittedReferenceBundle,
    };

    use super::cohort_execution_inputs;

    #[test]
    fn cohort_inputs_mount_every_pair_and_reference_authority_once() -> anyhow::Result<()> {
        let root = std::path::Path::new("/governed/study");
        let cohort = cohort(root);
        let reference = reference(root);
        let (inputs, samples) = cohort_execution_inputs(&cohort, Some(&reference))?;
        assert_eq!(samples.len(), 2);
        assert_eq!(inputs.len(), 10);
        assert_eq!(inputs.iter().filter(|input| input.id == "corpus_data_index").count(), 1);
        assert!(inputs.iter().any(|input| input.id == "reference_fasta"));
        assert_eq!(samples[0].sample_id, "sample-a");
        assert_eq!(samples[0].bam_mount, "/input/cohort/sample_a.bam");
        Ok(())
    }

    #[test]
    fn cohort_inputs_reject_a_different_reference_index() {
        let root = std::path::Path::new("/governed/study");
        let cohort = cohort(root);
        let mut reference = reference(root);
        reference.data_index_sha256 = "f".repeat(64);
        assert!(cohort_execution_inputs(&cohort, Some(&reference)).is_err());
    }

    fn cohort(root: &std::path::Path) -> AdmittedBamCohort {
        let sample = |sample_id: &str| AdmittedBamCohortSample {
            sample_id: sample_id.to_string(),
            bam: file(root, &format!("{sample_id}.bam"), 20),
            bam_index: file(root, &format!("{sample_id}.bam.bai"), 2),
            input_bytes: 22,
        };
        AdmittedBamCohort {
            corpus_id: "ancient-dna-bam-diversity".to_string(),
            data_index_path: root.join("data-index.json").display().to_string(),
            data_index_sha256: "a".repeat(64),
            catalog_path: root.join("cohort.json").display().to_string(),
            catalog_sha256: "b".repeat(64),
            sample_count: 2,
            file_count: 4,
            input_bytes: 44,
            samples: vec![sample("sample-a"), sample("sample-b")],
        }
    }

    fn reference(root: &std::path::Path) -> AdmittedReferenceBundle {
        AdmittedReferenceBundle {
            reference_id: "grch38-reference".to_string(),
            data_index_path: root.join("data-index.json").display().to_string(),
            data_index_sha256: "a".repeat(64),
            catalog_path: root.join("reference.json").display().to_string(),
            catalog_sha256: "c".repeat(64),
            fasta: file(root, "reference.fasta", 100),
            fasta_index: file(root, "reference.fasta.fai", 10),
            sequence_dictionary: file(root, "reference.dict", 5),
            input_bytes: 115,
        }
    }

    fn file(root: &std::path::Path, name: &str, size_bytes: u64) -> AdmittedBamCorpusFile {
        AdmittedBamCorpusFile {
            path: root.join(name).display().to_string(),
            sha256: "d".repeat(64),
            size_bytes,
        }
    }
}
