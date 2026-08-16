//! Owner: bijux-dna-bench
//! Deterministic projection from verified source bytes into the governed data catalog.

use anyhow::{bail, Result};
use bijux_dna_bench_model::{
    CorpusAcquisitionPlan, CorpusAcquisitionSample, CorpusMaterializationDisposition,
    GovernedDataCatalog, GovernedDataCatalogFile, GovernedDataCatalogSample,
    GOVERNED_DATA_CATALOG_SCHEMA_V2,
};
use bijux_dna_db_ena::{VerifiedDownloadDisposition, VerifiedDownloadOutcome};

use super::fastq::FastqIdentity;
use super::MaterializationTarget;

pub(super) fn build_catalog(
    plan: &CorpusAcquisitionPlan,
    targets: &[MaterializationTarget],
    transfers: &[VerifiedDownloadOutcome],
    fastq: &[FastqIdentity],
    source_record: String,
    source_record_sha256: String,
    admitted_at_utc: String,
) -> GovernedDataCatalog {
    let files = targets
        .iter()
        .zip(transfers)
        .zip(fastq)
        .map(|((target, transfer), observed)| {
            let sample = &plan.samples[target.sample_index];
            let source = &sample.files[target.file_index];
            GovernedDataCatalogFile {
                path: target.path.display().to_string(),
                sha256: transfer.sha256.clone(),
                size_bytes: transfer.size_bytes,
                kind: "raw-fastq".to_string(),
                accession: sample.accession.clone(),
                study_accession: sample.study_accession.clone(),
                sample_accession: sample.sample_accession.clone(),
                experiment_accession: sample.experiment_accession.clone(),
                era: sample.era.clone(),
                layout: sample.layout,
                mate: source.mate,
                read_count: observed.read_count,
                base_count: observed.base_count,
                source_url: source.source_url.clone(),
                source_md5: source.source_md5.clone(),
                materialization_status: match transfer.disposition {
                    VerifiedDownloadDisposition::Downloaded => {
                        CorpusMaterializationDisposition::Downloaded
                    }
                    VerifiedDownloadDisposition::VerifiedExisting => {
                        CorpusMaterializationDisposition::VerifiedExisting
                    }
                },
            }
        })
        .collect();
    GovernedDataCatalog {
        schema_version: GOVERNED_DATA_CATALOG_SCHEMA_V2.to_string(),
        id: plan.corpus_id.clone(),
        purpose: plan.description.clone(),
        source: plan.source_authority.clone(),
        policy: plan.policy.clone(),
        evaluation: plan.evaluation.clone(),
        source_record,
        source_record_sha256,
        retrieved_at_utc: plan.retrieved_at_utc.clone(),
        admitted_at_utc,
        scientific_name: plan.species.clone(),
        samples: plan.samples.iter().map(catalog_sample).collect(),
        files,
    }
}

pub(super) fn validate_existing_catalog(
    catalog: &GovernedDataCatalog,
    plan: &CorpusAcquisitionPlan,
    targets: &[MaterializationTarget],
    fastq: &[FastqIdentity],
    transfers: &[VerifiedDownloadOutcome],
    source_record: &str,
    source_record_sha256: &str,
) -> Result<()> {
    let expected_samples = plan.samples.iter().map(catalog_sample).collect::<Vec<_>>();
    if catalog.id != plan.corpus_id
        || catalog.source != plan.source_authority
        || catalog.purpose != plan.description
        || catalog.policy != plan.policy
        || catalog.evaluation != plan.evaluation
        || catalog.source_record != source_record
        || catalog.source_record_sha256 != source_record_sha256
        || catalog.scientific_name != plan.species
        || catalog.samples != expected_samples
        || catalog.files.len() != targets.len()
        || fastq.len() != targets.len()
        || transfers.len() != targets.len()
    {
        bail!("existing governed catalog differs from acquisition plan identity");
    }
    for (((file, target), observed), transfer) in
        catalog.files.iter().zip(targets).zip(fastq).zip(transfers)
    {
        let sample = &plan.samples[target.sample_index];
        let source = &sample.files[target.file_index];
        if file.path != target.path.display().to_string()
            || file.sha256 != transfer.sha256
            || file.size_bytes != source.size_bytes
            || file.size_bytes != transfer.size_bytes
            || file.source_md5 != transfer.md5
            || file.kind != "raw-fastq"
            || file.accession != sample.accession
            || file.study_accession != sample.study_accession
            || file.sample_accession != sample.sample_accession
            || file.experiment_accession != sample.experiment_accession
            || file.era != sample.era
            || file.layout != sample.layout
            || file.mate != source.mate
            || file.read_count != observed.read_count
            || file.base_count != observed.base_count
            || file.source_url != source.source_url
            || file.source_md5 != source.source_md5
        {
            bail!(
                "existing governed catalog file differs from acquisition plan or FASTQ: {}",
                target.path.display()
            );
        }
    }
    Ok(())
}

fn catalog_sample(sample: &CorpusAcquisitionSample) -> GovernedDataCatalogSample {
    GovernedDataCatalogSample {
        accession: sample.accession.clone(),
        study_accession: sample.study_accession.clone(),
        sample_accession: sample.sample_accession.clone(),
        experiment_accession: sample.experiment_accession.clone(),
        era: sample.era.clone(),
        layout: sample.layout,
        size_band: sample.size_band.clone(),
        selection_reason: sample.selection_reason.clone(),
        scientific_name: sample.scientific_name.clone(),
        library_source: sample.library_source.clone(),
        library_strategy: sample.library_strategy.clone(),
        library_selection: sample.library_selection.clone(),
        instrument_platform: sample.instrument_platform.clone(),
        instrument_model: sample.instrument_model.clone(),
        archive_base_count: sample.archive_base_count,
        archive_read_count: sample.archive_read_count,
        first_public: sample.first_public.clone(),
        last_updated: sample.last_updated.clone(),
    }
}
