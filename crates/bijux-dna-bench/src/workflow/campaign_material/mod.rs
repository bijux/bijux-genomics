//! Owner: bijux-dna-bench
//! Evidence admission and executable material generation for governed benchmark campaigns.

mod report;

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Result};
use bijux_dna_bench_model::{
    AdmittedAlignmentTruthSet, AdmittedBamCohort, AdmittedBamCorpus, AdmittedFastqCorpus,
    AdmittedIndexedReference, AdmittedReferenceBundle, BenchmarkAttemptIdentity,
    BenchmarkCorpusSnapshotV1,
};

use super::{
    prepare_bam_alignment_material, prepare_bam_coverage_material_from_corpus,
    prepare_bam_damage_material_from_evidence,
    prepare_bam_duplication_metrics_material_from_corpus,
    prepare_bam_endogenous_content_material_from_corpus,
    prepare_bam_genotyping_material_from_corpus, prepare_bam_mapping_summary_material_from_corpus,
    prepare_bam_picard_cohort_material, prepare_fastq_detect_adapters_material_from_expectations,
    prepare_fastq_profile_read_lengths_material_from_expectations,
    prepare_fastq_profile_reads_material_from_expectations,
    prepare_fastq_validate_reads_material_from_expectations, BenchmarkExecutionMaterial,
    FastqProfileReadsExpectationPublication, FastqReadLengthExpectationPublication,
};
use crate::repo::{
    admit_alignment_truth_set, admit_bam_benchmark_cohort, admit_bam_benchmark_corpus,
    admit_bam_genotyping_corpus, admit_benchmark_indexed_reference, admit_benchmark_reference,
    admit_fastq_benchmark_corpus, AdmittedBamGenotypingCorpus,
};

pub use report::{
    publish_benchmark_campaign_report,
    publish_benchmark_campaign_report_with_presentation_authority,
    publish_case_benchmark_campaign_report,
    publish_case_benchmark_campaign_report_with_presentation_authority,
    BenchmarkCampaignReportPublication,
};

const BAM_CORPUS_ID: &str = "ena-prjeb22390-err2112767-bam";
const FASTQ_CORPUS_ID: &str = "human-short-read-diversity";
const FASTQ_EXPECTED_SAMPLE_COUNT: usize = 20;
const FASTQ_EXPECTED_FILE_COUNT: usize = 30;

const EXECUTABLE_CAMPAIGN_RUN_IDS: &[&str] = &[
    "run-2001", "run-2002", "run-2003", "run-2004", "run-2005", "run-2011", "run-2013", "run-2015",
    "run-2010", "run-2030", "run-2042", "run-2043", "run-2051",
];

/// Admitted evidence closure required to prepare one governed campaign.
pub enum BenchmarkCampaignEvidence {
    BamAlignment {
        corpus: AdmittedFastqCorpus,
        reference: AdmittedReferenceBundle,
        truth: AdmittedAlignmentTruthSet,
    },
    BamGenotyping(Box<AdmittedBamGenotypingCorpus>),
    BamDamage {
        corpus: AdmittedBamCorpus,
        reference: AdmittedIndexedReference,
    },
    BamPicardCohort {
        cohort: AdmittedBamCohort,
        reference: Option<AdmittedReferenceBundle>,
    },
    BamObservation(AdmittedBamCorpus),
    FastqReadLengths(FastqReadLengthExpectationPublication),
    FastqProfileReads(FastqProfileReadsExpectationPublication),
}

impl BenchmarkCampaignEvidence {
    /// Return the exact primary corpus snapshot admitted for campaign execution.
    #[must_use]
    pub fn primary_corpus_snapshot(&self) -> BenchmarkCorpusSnapshotV1 {
        match self {
            Self::BamAlignment { corpus, .. } => fastq_corpus_snapshot(corpus),
            Self::BamGenotyping(corpus) => bam_corpus_snapshot(&corpus.bam),
            Self::BamDamage { corpus, .. } => bam_corpus_snapshot(corpus),
            Self::BamPicardCohort { cohort, .. } => BenchmarkCorpusSnapshotV1 {
                corpus_id: cohort.corpus_id.clone(),
                snapshot_sha256: cohort.catalog_sha256.clone(),
                sample_ids: sorted_sample_ids(
                    cohort.samples.iter().map(|sample| sample.sample_id.as_str()),
                ),
            },
            Self::BamObservation(corpus) => bam_corpus_snapshot(corpus),
            Self::FastqReadLengths(publication) => {
                fastq_corpus_snapshot(&publication.admitted_corpus)
            }
            Self::FastqProfileReads(publication) => {
                fastq_corpus_snapshot(&publication.admitted_corpus)
            }
        }
    }
}

fn fastq_corpus_snapshot(corpus: &AdmittedFastqCorpus) -> BenchmarkCorpusSnapshotV1 {
    BenchmarkCorpusSnapshotV1 {
        corpus_id: corpus.corpus_id.clone(),
        snapshot_sha256: corpus.catalog_sha256.clone(),
        sample_ids: sorted_sample_ids(
            corpus.samples.iter().map(|sample| sample.sample_id.as_str()),
        ),
    }
}

fn bam_corpus_snapshot(corpus: &AdmittedBamCorpus) -> BenchmarkCorpusSnapshotV1 {
    BenchmarkCorpusSnapshotV1 {
        corpus_id: corpus.corpus_id.clone(),
        snapshot_sha256: corpus.catalog_sha256.clone(),
        sample_ids: vec![corpus.sample_id.clone()],
    }
}

fn sorted_sample_ids<'a>(samples: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut samples = samples.map(str::to_owned).collect::<Vec<_>>();
    samples.sort();
    samples
}

/// Immutable operator and source authorities shared by one campaign attempt.
pub struct BenchmarkCampaignMaterialRequest<'a> {
    pub repository_root: &'a Path,
    pub benchmark_root: &'a Path,
    pub identity: &'a BenchmarkAttemptIdentity,
    pub source_contract_sha256: &'a BTreeMap<String, String>,
    pub operator: &'a str,
    pub prepared_at_utc: &'a str,
    pub account: &'a str,
    pub partition: &'a str,
}

#[must_use]
pub(crate) fn benchmark_campaign_has_executable_material(run_id: &str) -> bool {
    EXECUTABLE_CAMPAIGN_RUN_IDS.contains(&run_id)
}

/// Admit the exact corpus or expectation evidence required by a campaign.
///
/// # Errors
/// Returns an error when the campaign is unsupported or its data authorities cannot be admitted.
pub fn admit_benchmark_campaign_evidence(
    repository_root: &Path,
    study_root: &Path,
    data_index: &Path,
    run_id: &str,
) -> Result<BenchmarkCampaignEvidence> {
    match run_id {
        "run-2001" => Ok(BenchmarkCampaignEvidence::BamGenotyping(Box::new(
            admit_bam_genotyping_corpus(study_root, data_index)?,
        ))),
        "run-2005" => Ok(BenchmarkCampaignEvidence::BamAlignment {
            corpus: admit_fastq_benchmark_corpus(
                study_root,
                data_index,
                "alignment-truth-panel",
                10,
                20,
            )?,
            reference: admit_benchmark_reference(study_root, data_index, "grch38-reference")?,
            truth: admit_alignment_truth_set(study_root, data_index, "alignment-truth-panel", 10)?,
        }),
        "run-2010" => Ok(BenchmarkCampaignEvidence::BamDamage {
            corpus: admit_bam_benchmark_corpus(
                study_root,
                data_index,
                "ena-prjeb22390-err2112767-ngsbriggs-input",
            )?,
            reference: admit_benchmark_indexed_reference(
                study_root,
                data_index,
                "equcab3-gcf-002863925.1",
            )?,
        }),
        "run-2002" | "run-2003" | "run-2004" | "run-2011" => {
            Ok(BenchmarkCampaignEvidence::BamObservation(admit_bam_benchmark_corpus(
                study_root,
                data_index,
                BAM_CORPUS_ID,
            )?))
        }
        "run-2013" | "run-2015" => {
            let cohort = admit_bam_benchmark_cohort(
                study_root,
                data_index,
                "ancient-dna-bam-diversity",
                10,
            )?;
            let reference = if run_id == "run-2013" {
                Some(admit_benchmark_reference(study_root, data_index, "grch38-reference")?)
            } else {
                None
            };
            Ok(BenchmarkCampaignEvidence::BamPicardCohort { cohort, reference })
        }
        "run-2030" | "run-2042" | "run-2043" | "run-2051" => {
            let read_lengths = super::ensure_fastq_read_length_expectations(
                repository_root,
                study_root,
                data_index,
                FASTQ_CORPUS_ID,
                FASTQ_EXPECTED_SAMPLE_COUNT,
                FASTQ_EXPECTED_FILE_COUNT,
            )?;
            if run_id == "run-2042" {
                Ok(BenchmarkCampaignEvidence::FastqReadLengths(read_lengths))
            } else {
                Ok(BenchmarkCampaignEvidence::FastqProfileReads(
                    super::ensure_fastq_profile_reads_expectations(study_root, &read_lengths)?,
                ))
            }
        }
        _ => bail!("benchmark campaign has no crate-owned executable material: {run_id}"),
    }
}

/// Build executable material through the campaign implementation owned by this crate.
///
/// # Errors
/// Returns an error when evidence kind, campaign identity, lifecycle, or admitted inputs differ
/// from the governed campaign contract.
pub fn prepare_benchmark_campaign_material(
    request: &BenchmarkCampaignMaterialRequest<'_>,
    evidence: &BenchmarkCampaignEvidence,
) -> Result<BenchmarkExecutionMaterial> {
    match evidence {
        BenchmarkCampaignEvidence::BamAlignment { corpus, reference, truth } => {
            Ok(prepare_bam_alignment_material(
                request.repository_root,
                request.benchmark_root,
                request.identity,
                request.source_contract_sha256,
                request.operator,
                request.prepared_at_utc,
                request.account,
                request.partition,
                corpus,
                reference,
                truth,
            )?
            .execution)
        }
        BenchmarkCampaignEvidence::BamGenotyping(corpus) => {
            Ok(prepare_bam_genotyping_material_from_corpus(
                request.repository_root,
                request.benchmark_root,
                request.identity,
                request.source_contract_sha256,
                request.operator,
                request.prepared_at_utc,
                request.account,
                request.partition,
                corpus,
            )?
            .execution)
        }
        BenchmarkCampaignEvidence::BamDamage { corpus, reference } => {
            Ok(prepare_bam_damage_material_from_evidence(
                request.repository_root,
                request.benchmark_root,
                request.identity,
                request.source_contract_sha256,
                request.operator,
                request.prepared_at_utc,
                request.account,
                request.partition,
                corpus,
                reference,
            )?
            .execution)
        }
        BenchmarkCampaignEvidence::BamObservation(corpus) => {
            prepare_bam_observation_material(request, corpus)
        }
        BenchmarkCampaignEvidence::BamPicardCohort { cohort, reference } => {
            Ok(prepare_bam_picard_cohort_material(
                request.repository_root,
                request.benchmark_root,
                request.identity,
                request.source_contract_sha256,
                request.operator,
                request.prepared_at_utc,
                request.account,
                request.partition,
                cohort,
                reference.as_ref(),
            )?
            .execution)
        }
        BenchmarkCampaignEvidence::FastqReadLengths(expectations) => {
            Ok(prepare_fastq_profile_read_lengths_material_from_expectations(
                request.repository_root,
                request.benchmark_root,
                request.identity,
                request.source_contract_sha256,
                request.operator,
                request.prepared_at_utc,
                request.account,
                request.partition,
                expectations,
            )?
            .execution)
        }
        BenchmarkCampaignEvidence::FastqProfileReads(expectations) => {
            if request.identity.run.run_id == "run-2030" {
                Ok(prepare_fastq_detect_adapters_material_from_expectations(
                    request.repository_root,
                    request.benchmark_root,
                    request.identity,
                    request.source_contract_sha256,
                    request.operator,
                    request.prepared_at_utc,
                    request.account,
                    request.partition,
                    expectations,
                )?
                .execution)
            } else if request.identity.run.run_id == "run-2051" {
                Ok(prepare_fastq_validate_reads_material_from_expectations(
                    request.repository_root,
                    request.benchmark_root,
                    request.identity,
                    request.source_contract_sha256,
                    request.operator,
                    request.prepared_at_utc,
                    request.account,
                    request.partition,
                    expectations,
                )?
                .execution)
            } else {
                Ok(prepare_fastq_profile_reads_material_from_expectations(
                    request.repository_root,
                    request.benchmark_root,
                    request.identity,
                    request.source_contract_sha256,
                    request.operator,
                    request.prepared_at_utc,
                    request.account,
                    request.partition,
                    expectations,
                )?
                .execution)
            }
        }
    }
}

fn prepare_bam_observation_material(
    request: &BenchmarkCampaignMaterialRequest<'_>,
    corpus: &AdmittedBamCorpus,
) -> Result<BenchmarkExecutionMaterial> {
    let material = match request.identity.run.run_id.as_str() {
        "run-2002" => {
            prepare_bam_coverage_material_from_corpus(
                request.repository_root,
                request.benchmark_root,
                request.identity,
                request.source_contract_sha256,
                request.operator,
                request.prepared_at_utc,
                request.account,
                request.partition,
                corpus,
            )?
            .execution
        }
        "run-2003" => {
            prepare_bam_endogenous_content_material_from_corpus(
                request.repository_root,
                request.benchmark_root,
                request.identity,
                request.source_contract_sha256,
                request.operator,
                request.prepared_at_utc,
                request.account,
                request.partition,
                corpus,
            )?
            .execution
        }
        "run-2004" => {
            prepare_bam_mapping_summary_material_from_corpus(
                request.repository_root,
                request.benchmark_root,
                request.identity,
                request.source_contract_sha256,
                request.operator,
                request.prepared_at_utc,
                request.account,
                request.partition,
                corpus,
            )?
            .execution
        }
        "run-2011" => {
            prepare_bam_duplication_metrics_material_from_corpus(
                request.repository_root,
                request.benchmark_root,
                request.identity,
                request.source_contract_sha256,
                request.operator,
                request.prepared_at_utc,
                request.account,
                request.partition,
                corpus,
            )?
            .execution
        }
        run_id => bail!("BAM observation evidence is not valid for campaign {run_id}"),
    };
    Ok(material)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use bijux_dna_bench_model::{AdmittedBamCorpus, AdmittedBamCorpusFile};

    use super::{
        benchmark_campaign_has_executable_material, BenchmarkCampaignEvidence,
        EXECUTABLE_CAMPAIGN_RUN_IDS,
    };

    #[test]
    fn executable_campaign_material_covers_the_declared_registry() -> anyhow::Result<()> {
        let (registry, _, _) = super::super::load_checked_in_benchmark_planning_contracts()?;
        let declared = registry
            .campaigns
            .iter()
            .map(|campaign| campaign.run_id.as_str())
            .collect::<BTreeSet<_>>();
        let executable = EXECUTABLE_CAMPAIGN_RUN_IDS.iter().copied().collect::<BTreeSet<_>>();
        assert_eq!(executable, declared);
        assert!(declared.iter().all(|run_id| benchmark_campaign_has_executable_material(run_id)));
        assert!(!benchmark_campaign_has_executable_material("run-9999"));
        Ok(())
    }

    #[test]
    fn campaign_evidence_exposes_its_exact_primary_corpus_snapshot() {
        let file = AdmittedBamCorpusFile {
            path: "/study/data/sample.bam".to_string(),
            sha256: "a".repeat(64),
            size_bytes: 42,
        };
        let evidence = BenchmarkCampaignEvidence::BamObservation(AdmittedBamCorpus {
            corpus_id: "real-bam-corpus".to_string(),
            sample_id: "sample-a".to_string(),
            data_index_path: "/study/data-index.json".to_string(),
            data_index_sha256: "b".repeat(64),
            catalog_path: "/study/catalog.json".to_string(),
            catalog_sha256: "c".repeat(64),
            bam: file.clone(),
            bam_index: file,
            input_bytes: 84,
        });

        let snapshot = evidence.primary_corpus_snapshot();
        assert_eq!(snapshot.corpus_id, "real-bam-corpus");
        assert_eq!(snapshot.snapshot_sha256, "c".repeat(64));
        assert_eq!(snapshot.sample_ids, ["sample-a"]);
    }
}
