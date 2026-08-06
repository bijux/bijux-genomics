//! Owner: bijux-dna-bench
//! Deterministic projection from verified source bytes into the governed data catalog.

use bijux_dna_bench_model::{
    CorpusAcquisitionPlan, CorpusAcquisitionSample, CorpusMaterializationDisposition,
    GovernedDataCatalog, GovernedDataCatalogFile, GovernedDataCatalogSample,
    GOVERNED_DATA_CATALOG_SCHEMA_V1,
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
        schema_version: GOVERNED_DATA_CATALOG_SCHEMA_V1.to_string(),
        id: plan.portfolio_id.clone(),
        purpose: "real human short-read diversity corpus for comparative FASTQ campaigns"
            .to_string(),
        source: plan.source_authority.clone(),
        source_record,
        source_record_sha256,
        retrieved_at_utc: plan.retrieved_at_utc.clone(),
        admitted_at_utc,
        access_classification: "public".to_string(),
        redistribution_classification: "accession-referenced".to_string(),
        access_terms: "ENA public data access terms apply; publication identifies source accessions."
            .to_string(),
        scientific_name: plan.species.clone(),
        samples: plan.samples.iter().map(catalog_sample).collect(),
        files,
        known_limitations: vec![
            "The cohort is curated for workload diversity and is not population-representative."
                .to_string(),
            "Public archive metadata and sequencing context differ across studies and eras."
                .to_string(),
            "No independent sequence truth is available; this corpus supports observational and robustness comparisons only."
                .to_string(),
        ],
    }
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
