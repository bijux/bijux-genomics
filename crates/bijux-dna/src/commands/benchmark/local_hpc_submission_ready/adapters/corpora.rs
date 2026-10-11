use super::LocalReadinessAdapter;
use anyhow::Result;
use bijux_dna_api::v1::api::bench::{
    BenchmarkLocalCorpusSkipReport, BenchmarkLocalCorpusStageCompatibilityReport,
};
use bijux_dna_bench_hpc::readiness::ports::{
    AmpliconFixtureObservation, CorpusEvidence, CorpusSamplesObservation, DamageFixtureObservation,
    EdnaFixtureObservation, TaxonomyFixtureObservation,
};
use std::path::Path;
impl CorpusEvidence for LocalReadinessAdapter {
    fn validate_fastq_corpus_fixture_manifest_path(
        &self,
        repo_root: &Path,
        manifest_path: &Path,
    ) -> Result<CorpusSamplesObservation> {
        let report = crate::commands::benchmark::local_corpus_fixture::fastq::validate_fastq_corpus_fixture_manifest_path(repo_root, manifest_path)?;
        Ok(CorpusSamplesObservation { sample_count: report.sample_count, valid: report.valid })
    }
    fn validate_bam_corpus_fixture_manifest_path(
        &self,
        repo_root: &Path,
        manifest_path: &Path,
    ) -> Result<CorpusSamplesObservation> {
        let report = crate::commands::benchmark::local_corpus_fixture::bam::validate_bam_corpus_fixture_manifest_path(repo_root, manifest_path)?;
        Ok(CorpusSamplesObservation { sample_count: report.sample_count, valid: report.valid })
    }
    fn validate_bam_damage_fixture_manifest_path(
        &self,
        repo_root: &Path,
        manifest_path: &Path,
    ) -> Result<DamageFixtureObservation> {
        let report = crate::commands::benchmark::local_corpus_fixture::damage::validate_bam_damage_fixture_manifest_path(repo_root, manifest_path)?;
        Ok(DamageFixtureObservation {
            expected_damage_path: report.expected_damage_path,
            valid: report.valid,
        })
    }
    fn validate_edna_corpus_fixture_manifest_path(
        &self,
        repo_root: &Path,
        manifest_path: &Path,
    ) -> Result<EdnaFixtureObservation> {
        let report = crate::commands::benchmark::local_corpus_fixture::edna::validate_edna_corpus_fixture_manifest_path(repo_root, manifest_path)?;
        Ok(EdnaFixtureObservation {
            manifest_path: report.manifest_path,
            sample_count: report.sample_count,
            expected_taxa_path: report.expected_taxa_path,
            expected_taxa_count: report.expected_taxa_count,
            expected_taxa_output_row_count: report.expected_taxa_output_row_count,
            valid: report.valid,
        })
    }
    fn validate_amplicon_corpus_fixture_manifest_path(
        &self,
        repo_root: &Path,
        manifest_path: &Path,
    ) -> Result<AmpliconFixtureObservation> {
        let report = crate::commands::benchmark::local_corpus_fixture::amplicon::validate_amplicon_corpus_fixture_manifest_path(repo_root, manifest_path)?;
        Ok(AmpliconFixtureObservation {
            manifest_path: report.manifest_path,
            sample_count: report.sample_count,
            control_count: report.control_count,
            primer_table_row_count: report.primer_table_row_count,
            primers_tsv_path: report.primers_tsv_path,
            expected_asv_row_count: report.expected_asv_row_count,
            expected_asvs_path: report.expected_asvs_path,
            chimera_expectation_row_count: report.chimera_expectation_row_count,
            chimera_controls_fasta_path: report.chimera_controls_fasta_path,
            chimera_expectations_path: report.chimera_expectations_path,
            valid: report.valid,
        })
    }
    fn validate_taxonomy_database_fixture_manifest_path(
        &self,
        repo_root: &Path,
        manifest_path: &Path,
    ) -> Result<TaxonomyFixtureObservation> {
        let report = crate::commands::benchmark::local_taxonomy_database_fixture::validate_taxonomy_database_fixture_manifest_path(repo_root, manifest_path)?;
        Ok(TaxonomyFixtureObservation {
            valid: report.valid,
            taxa_count: report.taxa_count,
            source_record_count: report.source_record_count,
        })
    }
    fn validate_corpus_stage_compatibility_path(
        &self,
        repo_root: &Path,
        matrix_path: &Path,
    ) -> Result<BenchmarkLocalCorpusStageCompatibilityReport> {
        crate::commands::benchmark::local_corpus_stage_compatibility::validate_corpus_stage_compatibility_path(repo_root, matrix_path)
    }
    fn render_corpus_skip_report_path(
        &self,
        repo_root: &Path,
        matrix_path: &Path,
        output_path: &Path,
    ) -> Result<BenchmarkLocalCorpusSkipReport> {
        crate::commands::benchmark::local_corpus_skip_report::render_corpus_skip_report_path(
            repo_root,
            matrix_path,
            output_path,
        )
    }
}
