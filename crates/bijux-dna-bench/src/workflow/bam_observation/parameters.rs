//! Owner: bijux-dna-bench
//! Immutable effective-parameter evidence for BAM observational campaigns.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    AdmittedBamCohort, BenchmarkEffectiveParameterValue, BenchmarkEffectiveParameters,
    BenchmarkExecutionParameterEvidence, BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
};

use super::super::parameter_authority::load_stage_parameter_authority;

pub(in crate::workflow) struct BamParameterPolicy<'a> {
    pub(in crate::workflow) stage_id: &'a str,
    pub(in crate::workflow) defaults_provider: &'a str,
}

pub(in crate::workflow) fn ensure_parameter_evidence(
    repository_root: &Path,
    attempt_directory: &Path,
    sample_id: &str,
    policy: BamParameterPolicy<'_>,
    resolved: BTreeMap<String, BenchmarkEffectiveParameterValue>,
) -> Result<BenchmarkExecutionParameterEvidence> {
    let resolved_by_sample = BTreeMap::from([(sample_id.to_string(), resolved)]);
    ensure_resolved_parameter_evidence(
        repository_root,
        attempt_directory,
        policy,
        resolved_by_sample,
    )
}

pub(in crate::workflow) fn ensure_cohort_parameter_evidence(
    repository_root: &Path,
    attempt_directory: &Path,
    cohort: &AdmittedBamCohort,
    policy: BamParameterPolicy<'_>,
    resolved: BTreeMap<String, BenchmarkEffectiveParameterValue>,
) -> Result<BenchmarkExecutionParameterEvidence> {
    let sample_count = usize::try_from(cohort.sample_count).context("BAM cohort sample count")?;
    if sample_count == 0 || sample_count != cohort.samples.len() {
        bail!("BAM cohort sample count differs from admitted samples");
    }
    let resolved_by_sample = cohort
        .samples
        .iter()
        .map(|sample| (sample.sample_id.clone(), resolved.clone()))
        .collect::<BTreeMap<_, _>>();
    if resolved_by_sample.len() != sample_count {
        bail!("BAM cohort contains duplicate sample identities");
    }
    ensure_resolved_parameter_evidence(
        repository_root,
        attempt_directory,
        policy,
        resolved_by_sample,
    )
}

fn ensure_resolved_parameter_evidence(
    repository_root: &Path,
    attempt_directory: &Path,
    policy: BamParameterPolicy<'_>,
    mut resolved_by_sample: BTreeMap<String, BTreeMap<String, BenchmarkEffectiveParameterValue>>,
) -> Result<BenchmarkExecutionParameterEvidence> {
    let authority = load_stage_parameter_authority(repository_root, policy.stage_id)?;
    let stage = authority.stage;
    if stage.defaults_provider != policy.defaults_provider || !stage.overrides.is_empty() {
        bail!("BAM parameter authority differs from executable policy");
    }
    for resolved in resolved_by_sample.values_mut() {
        resolved.insert(
            "schema_version".to_string(),
            BenchmarkEffectiveParameterValue::Text(stage.parameter_schema_version.clone()),
        );
    }

    let overrides_path = attempt_directory.join("config-pipeline-parameters.yml");
    ensure_bytes(&overrides_path, &authority.source_bytes)?;
    let effective = BenchmarkEffectiveParameters {
        schema_version: BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1.to_string(),
        stage_id: policy.stage_id.to_string(),
        parameter_schema_version: stage.parameter_schema_version.clone(),
        defaults_provider: policy.defaults_provider.to_string(),
        overrides: BTreeMap::new(),
        resolved_by_sample,
    };
    let mut effective_bytes = serde_json::to_vec_pretty(&effective)?;
    effective_bytes.push(b'\n');
    let effective_path = attempt_directory.join("effective-parameters.json");
    ensure_bytes(&effective_path, &effective_bytes)?;
    Ok(BenchmarkExecutionParameterEvidence {
        parameter_schema_version: stage.parameter_schema_version,
        defaults_provider: policy.defaults_provider.to_string(),
        overrides_path: "config-pipeline-parameters.yml".to_string(),
        overrides_sha256: bijux_dna_infra::hash_bytes_sha256(&authority.source_bytes),
        effective_path: "effective-parameters.json".to_string(),
        effective_sha256: bijux_dna_infra::hash_bytes_sha256(&effective_bytes),
    })
}

fn ensure_bytes(path: &Path, expected: &[u8]) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("parameter evidence is not a regular non-symlink file: {}", path.display());
            }
            if std::fs::read(path)? != expected {
                bail!("parameter evidence conflicts with immutable bytes: {}", path.display());
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(bijux_dna_infra::atomic_create_bytes(path, expected)?)
        }
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_bench_model::{
        AdmittedBamCohort, AdmittedBamCohortSample, AdmittedBamCorpusFile,
        BenchmarkEffectiveParameterValue as EffectiveValue, BenchmarkEffectiveParameters,
    };

    use super::{ensure_cohort_parameter_evidence, BamParameterPolicy};

    #[test]
    fn cohort_parameters_preserve_every_sample_identity() -> anyhow::Result<()> {
        let repository =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let temporary = bijux_dna_infra::temp_dir("bam-cohort-parameters")?;
        let cohort = admitted_cohort(temporary.path());
        let evidence = ensure_cohort_parameter_evidence(
            &repository,
            temporary.path(),
            &cohort,
            BamParameterPolicy {
                stage_id: "bam.insert_size",
                defaults_provider:
                    "bijux_dna_domain_bam::stage_specs::stage_spec(BamStage::InsertSize)",
            },
            BTreeMap::from([
                ("regions".to_string(), EffectiveValue::Null),
                (
                    "depth_thresholds".to_string(),
                    EffectiveValue::List(vec![EffectiveValue::Integer(1)]),
                ),
                (
                    "regime_mode".to_string(),
                    EffectiveValue::Text("advisory_and_enforced".to_string()),
                ),
            ]),
        )?;
        let effective: BenchmarkEffectiveParameters = serde_json::from_slice(&std::fs::read(
            temporary.path().join(&evidence.effective_path),
        )?)?;
        assert_eq!(effective.resolved_by_sample.len(), 2);
        assert!(effective.resolved_by_sample.contains_key("sample-a"));
        assert!(effective.resolved_by_sample.contains_key("sample-b"));
        assert!(ensure_cohort_parameter_evidence(
            &repository,
            temporary.path(),
            &cohort,
            BamParameterPolicy {
                stage_id: "bam.insert_size",
                defaults_provider:
                    "bijux_dna_domain_bam::stage_specs::stage_spec(BamStage::InsertSize)",
            },
            BTreeMap::new(),
        )
        .is_err());
        Ok(())
    }

    fn admitted_cohort(root: &std::path::Path) -> AdmittedBamCohort {
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
            catalog_path: root.join("catalog.json").display().to_string(),
            catalog_sha256: "b".repeat(64),
            sample_count: 2,
            file_count: 4,
            input_bytes: 44,
            samples: vec![sample("sample-a"), sample("sample-b")],
        }
    }

    fn file(root: &std::path::Path, name: &str, size_bytes: u64) -> AdmittedBamCorpusFile {
        AdmittedBamCorpusFile {
            path: root.join(name).display().to_string(),
            sha256: "c".repeat(64),
            size_bytes,
        }
    }
}
