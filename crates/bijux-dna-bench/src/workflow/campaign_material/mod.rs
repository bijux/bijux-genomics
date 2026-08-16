//! Owner: bijux-dna-bench
//! Evidence admission and executable material generation for governed benchmark campaigns.

mod report;

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    AdmittedAlignmentTruthSet, AdmittedBamCohort, AdmittedBamCorpus, AdmittedFastqCorpus,
    AdmittedIndexedReference, AdmittedReferenceBundle, BenchmarkAttemptIdentity,
    BenchmarkCampaignMaterialDriver, BenchmarkCorpusSnapshotV1,
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
    BamCoverage(AdmittedBamCorpus),
    BamDuplicationMetrics(AdmittedBamCorpus),
    BamEndogenousContent(AdmittedBamCorpus),
    BamGcBias {
        cohort: AdmittedBamCohort,
        reference: AdmittedReferenceBundle,
    },
    BamInsertSize(AdmittedBamCohort),
    BamMappingSummary(AdmittedBamCorpus),
    FastqDetectAdapters(FastqProfileReadsExpectationPublication),
    FastqReadLengths(FastqReadLengthExpectationPublication),
    FastqProfileReads(FastqProfileReadsExpectationPublication),
    FastqValidateReads(FastqProfileReadsExpectationPublication),
}

impl BenchmarkCampaignEvidence {
    /// Return the exact primary corpus snapshot admitted for campaign execution.
    #[must_use]
    pub fn primary_corpus_snapshot(&self) -> BenchmarkCorpusSnapshotV1 {
        match self {
            Self::BamAlignment { corpus, .. } => fastq_corpus_snapshot(corpus),
            Self::BamGenotyping(corpus) => bam_corpus_snapshot(&corpus.bam),
            Self::BamDamage { corpus, .. } => bam_corpus_snapshot(corpus),
            Self::BamGcBias { cohort, .. } | Self::BamInsertSize(cohort) => {
                BenchmarkCorpusSnapshotV1 {
                    corpus_id: cohort.corpus_id.clone(),
                    snapshot_sha256: cohort.catalog_sha256.clone(),
                    sample_ids: sorted_sample_ids(
                        cohort.samples.iter().map(|sample| sample.sample_id.as_str()),
                    ),
                }
            }
            Self::BamCoverage(corpus)
            | Self::BamDuplicationMetrics(corpus)
            | Self::BamEndogenousContent(corpus)
            | Self::BamMappingSummary(corpus) => bam_corpus_snapshot(corpus),
            Self::FastqDetectAdapters(publication)
            | Self::FastqProfileReads(publication)
            | Self::FastqValidateReads(publication) => {
                fastq_corpus_snapshot(&publication.admitted_corpus)
            }
            Self::FastqReadLengths(publication) => {
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
    let campaigns = super::load_stage_campaign_registry(
        &repository_root.join("benchmarks/contracts/stage-campaigns.toml"),
    )?;
    let campaign = campaigns
        .campaigns
        .iter()
        .find(|campaign| campaign.run_id == run_id)
        .with_context(|| format!("benchmark campaign is not declared: {run_id}"))?;
    let material_driver = campaign
        .material_driver
        .context("validated benchmark campaign omits its material driver")?;
    match material_driver {
        BenchmarkCampaignMaterialDriver::BamGenotyping => {
            Ok(BenchmarkCampaignEvidence::BamGenotyping(Box::new(admit_bam_genotyping_corpus(
                study_root, data_index,
            )?)))
        }
        BenchmarkCampaignMaterialDriver::BamAlignment => {
            Ok(BenchmarkCampaignEvidence::BamAlignment {
                corpus: admit_fastq_benchmark_corpus(
                    study_root,
                    data_index,
                    "alignment-truth-panel",
                    10,
                    20,
                )?,
                reference: admit_benchmark_reference(study_root, data_index, "grch38-reference")?,
                truth: admit_alignment_truth_set(
                    study_root,
                    data_index,
                    "alignment-truth-panel",
                    10,
                )?,
            })
        }
        BenchmarkCampaignMaterialDriver::BamDamage => Ok(BenchmarkCampaignEvidence::BamDamage {
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
        BenchmarkCampaignMaterialDriver::BamCoverage => Ok(BenchmarkCampaignEvidence::BamCoverage(
            admit_bam_benchmark_corpus(study_root, data_index, BAM_CORPUS_ID)?,
        )),
        BenchmarkCampaignMaterialDriver::BamDuplicationMetrics => {
            Ok(BenchmarkCampaignEvidence::BamDuplicationMetrics(admit_bam_benchmark_corpus(
                study_root,
                data_index,
                BAM_CORPUS_ID,
            )?))
        }
        BenchmarkCampaignMaterialDriver::BamEndogenousContent => {
            Ok(BenchmarkCampaignEvidence::BamEndogenousContent(admit_bam_benchmark_corpus(
                study_root,
                data_index,
                BAM_CORPUS_ID,
            )?))
        }
        BenchmarkCampaignMaterialDriver::BamMappingSummary => {
            Ok(BenchmarkCampaignEvidence::BamMappingSummary(admit_bam_benchmark_corpus(
                study_root,
                data_index,
                BAM_CORPUS_ID,
            )?))
        }
        BenchmarkCampaignMaterialDriver::BamGcBias => Ok(BenchmarkCampaignEvidence::BamGcBias {
            cohort: admit_bam_benchmark_cohort(
                study_root,
                data_index,
                "ancient-dna-bam-diversity",
                10,
            )?,
            reference: admit_benchmark_reference(study_root, data_index, "grch38-reference")?,
        }),
        BenchmarkCampaignMaterialDriver::BamInsertSize => {
            Ok(BenchmarkCampaignEvidence::BamInsertSize(admit_bam_benchmark_cohort(
                study_root,
                data_index,
                "ancient-dna-bam-diversity",
                10,
            )?))
        }
        BenchmarkCampaignMaterialDriver::FastqDetectAdapters
        | BenchmarkCampaignMaterialDriver::FastqProfileReadLengths
        | BenchmarkCampaignMaterialDriver::FastqProfileReads
        | BenchmarkCampaignMaterialDriver::FastqValidateReads => {
            let read_lengths = super::ensure_fastq_read_length_expectations(
                repository_root,
                study_root,
                data_index,
                FASTQ_CORPUS_ID,
                FASTQ_EXPECTED_SAMPLE_COUNT,
                FASTQ_EXPECTED_FILE_COUNT,
            )?;
            if material_driver == BenchmarkCampaignMaterialDriver::FastqProfileReadLengths {
                return Ok(BenchmarkCampaignEvidence::FastqReadLengths(read_lengths));
            }
            let expectations =
                super::ensure_fastq_profile_reads_expectations(study_root, &read_lengths)?;
            match material_driver {
                BenchmarkCampaignMaterialDriver::FastqDetectAdapters => {
                    Ok(BenchmarkCampaignEvidence::FastqDetectAdapters(expectations))
                }
                BenchmarkCampaignMaterialDriver::FastqProfileReads => {
                    Ok(BenchmarkCampaignEvidence::FastqProfileReads(expectations))
                }
                BenchmarkCampaignMaterialDriver::FastqValidateReads => {
                    Ok(BenchmarkCampaignEvidence::FastqValidateReads(expectations))
                }
                _ => unreachable!("read-length material driver handled before profile admission"),
            }
        }
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
        BenchmarkCampaignEvidence::BamCoverage(corpus) => {
            Ok(prepare_bam_coverage_material_from_corpus(
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
        BenchmarkCampaignEvidence::BamDuplicationMetrics(corpus) => {
            Ok(prepare_bam_duplication_metrics_material_from_corpus(
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
        BenchmarkCampaignEvidence::BamEndogenousContent(corpus) => {
            Ok(prepare_bam_endogenous_content_material_from_corpus(
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
        BenchmarkCampaignEvidence::BamGcBias { cohort, reference } => {
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
                Some(reference),
            )?
            .execution)
        }
        BenchmarkCampaignEvidence::BamInsertSize(cohort) => Ok(prepare_bam_picard_cohort_material(
            request.repository_root,
            request.benchmark_root,
            request.identity,
            request.source_contract_sha256,
            request.operator,
            request.prepared_at_utc,
            request.account,
            request.partition,
            cohort,
            None,
        )?
        .execution),
        BenchmarkCampaignEvidence::BamMappingSummary(corpus) => {
            Ok(prepare_bam_mapping_summary_material_from_corpus(
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
        BenchmarkCampaignEvidence::FastqDetectAdapters(expectations) => {
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
        BenchmarkCampaignEvidence::FastqValidateReads(expectations) => {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use bijux_dna_bench_model::{AdmittedBamCorpus, AdmittedBamCorpusFile};

    use super::BenchmarkCampaignEvidence;

    #[test]
    fn executable_campaigns_use_reviewed_drivers_or_generic_bindings() -> anyhow::Result<()> {
        let (registry, _, _) = super::super::load_checked_in_benchmark_planning_contracts()?;
        let drivers = registry
            .campaigns
            .iter()
            .filter_map(|campaign| campaign.material_driver)
            .collect::<BTreeSet<_>>();
        assert_eq!(drivers.len(), 13);
        assert!(registry
            .campaigns
            .iter()
            .filter(|campaign| campaign.material_driver.is_some())
            .all(|campaign| campaign
                .material_driver
                .is_some_and(|driver| driver.stage_id() == campaign.stage_id)));
        assert_eq!(
            registry.campaigns.iter().filter(|campaign| campaign.material_driver.is_none()).count(),
            58
        );
        Ok(())
    }

    #[test]
    fn campaign_evidence_exposes_its_exact_primary_corpus_snapshot() {
        let file = AdmittedBamCorpusFile {
            path: "/study/data/sample.bam".to_string(),
            sha256: "a".repeat(64),
            size_bytes: 42,
        };
        let evidence = BenchmarkCampaignEvidence::BamCoverage(AdmittedBamCorpus {
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
