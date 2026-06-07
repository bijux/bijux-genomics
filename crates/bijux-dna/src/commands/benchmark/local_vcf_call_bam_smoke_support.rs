use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

use super::local_corpus_fixture::bam::{
    validate_bam_corpus_fixture_manifest_path, DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH,
};
use super::local_stage_result_manifest::{
    path_relative_to_repo, BenchStageResultCommandV1, BenchStageResultManifestV1,
    BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::local_vcf_stage_matrix::{build_vcf_stage_matrix_rows, VcfStageMatrixRow};

pub(crate) const GOVERNED_VCF_CALL_TOOL_ID: &str = "bcftools";
pub(crate) const GOVERNED_BAM_SAMPLE_ID: &str = "human_like_validation";
pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfCallBamSmokeContract {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) input_bam: String,
    pub(crate) input_bam_index: String,
    pub(crate) reference: String,
    pub(crate) sample_id: String,
    pub(crate) sample_name: String,
}

pub(crate) fn resolve_governed_vcf_call_bam_smoke_contract(
    repo_root: &Path,
    requested_stage_id: &str,
    requested_tool_id: &str,
    expected_output_id: &str,
) -> Result<GovernedVcfCallBamSmokeContract> {
    let matrix_row =
        resolve_vcf_call_matrix_row(requested_stage_id, requested_tool_id, expected_output_id)?;
    let fixture_report = validate_bam_corpus_fixture_manifest_path(
        repo_root,
        &repo_root.join(DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH),
    )?;
    let sample = fixture_report
        .samples
        .iter()
        .find(|sample| sample.sample_id == GOVERNED_BAM_SAMPLE_ID)
        .ok_or_else(|| {
            anyhow!(
                "governed BAM fixture is missing required VCF call smoke sample `{GOVERNED_BAM_SAMPLE_ID}`"
            )
        })?;
    let sample_name = sample.observed_header_sample_ids.first().cloned().ok_or_else(|| {
        anyhow!("governed BAM fixture sample `{}` has no observed sample name", sample.sample_id)
    })?;
    if !sample.alignment_path.ends_with(".bam") {
        bail!(
            "governed VCF BAM smoke requires a real BAM input, found `{}`",
            sample.alignment_path
        );
    }
    if !sample.index_path.ends_with(".bam.bai") {
        bail!("governed VCF BAM smoke requires a BAM index sidecar, found `{}`", sample.index_path);
    }

    Ok(GovernedVcfCallBamSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        input_bam: sample.alignment_path.clone(),
        input_bam_index: sample.index_path.clone(),
        reference: fixture_report.reference_fasta,
        sample_id: sample.sample_id.clone(),
        sample_name,
    })
}

pub(crate) fn resolve_vcf_call_matrix_row(
    requested_stage_id: &str,
    requested_tool_id: &str,
    expected_output_id: &str,
) -> Result<VcfStageMatrixRow> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == requested_stage_id)
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{requested_stage_id}`"))?;
    if matrix_row.tool_id != GOVERNED_VCF_CALL_TOOL_ID {
        bail!(
            "VCF BAM smoke requires retained tool `{GOVERNED_VCF_CALL_TOOL_ID}`, found `{}` in the governed matrix",
            matrix_row.tool_id
        );
    }
    if requested_tool_id != matrix_row.tool_id {
        bail!(
            "VCF BAM smoke only retains tool `{}` for `{}`; requested `{requested_tool_id}`",
            matrix_row.tool_id,
            matrix_row.stage_id
        );
    }
    if matrix_row.corpus_id != "vcf_production_regression" {
        bail!(
            "VCF BAM smoke requires corpus `vcf_production_regression`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != "bam_bundle" {
        bail!(
            "VCF BAM smoke requires asset profile `bam_bundle`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec![expected_output_id.to_string()] {
        bail!(
            "VCF BAM smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }
    Ok(matrix_row)
}

pub(crate) fn parse_output_sample_count(vcf_path: &Path) -> Result<u64> {
    let raw = bijux_dna_stages_vcf::vcf_io::read_vcf_text(vcf_path)?;
    let sample_header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("VCF output is missing the #CHROM header"))?;
    let sample_count = sample_header.split('\t').skip(9).count();
    u64::try_from(sample_count).map_err(|_| anyhow!("VCF output sample count overflowed u64"))
}

pub(crate) fn materialize_reference_fasta(
    source_reference: &Path,
    artifacts_root: &Path,
) -> Result<PathBuf> {
    let reference_root = artifacts_root.join("reference");
    fs::create_dir_all(&reference_root)
        .with_context(|| format!("create {}", reference_root.display()))?;
    let file_name = source_reference.file_name().ok_or_else(|| {
        anyhow!("reference FASTA has no file name: {}", source_reference.display())
    })?;
    let materialized_reference = reference_root.join(file_name);
    fs::copy(source_reference, &materialized_reference).with_context(|| {
        format!(
            "copy governed reference {} to {}",
            source_reference.display(),
            materialized_reference.display()
        )
    })?;
    Ok(materialized_reference)
}

pub(crate) fn build_stage_result_manifest(
    repo_root: &Path,
    contract: &GovernedVcfCallBamSmokeContract,
    command: &str,
    output_entries: &[(&str, String, &Path, &str)],
    started_at: &str,
    finished_at: &str,
    elapsed_seconds: f64,
) -> BenchStageResultManifestV1 {
    BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: contract.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: contract.tool_id.clone() },
        command: BenchStageResultCommandV1 { rendered: command.to_string() },
        runtime: BenchStageResultRuntimeV1 {
            mode: "local_smoke".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: started_at.to_string(),
            finished_at: finished_at.to_string(),
            elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs: output_entries
            .iter()
            .map(|(artifact_id, declared_path, realized_path, role)| BenchStageResultOutputV1 {
                artifact_id: (*artifact_id).to_string(),
                declared_path: declared_path.clone(),
                realized_path: path_relative_to_repo(repo_root, realized_path),
                role: (*role).to_string(),
                optional: false,
                exists: true,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        parse_output_sample_count, resolve_governed_vcf_call_bam_smoke_contract,
        GOVERNED_VCF_CALL_TOOL_ID,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn governed_vcf_bam_smoke_contract_uses_matrix_and_bam_fixture() {
        let repo_root = repo_root();
        let contract = resolve_governed_vcf_call_bam_smoke_contract(
            &repo_root,
            "vcf.call",
            GOVERNED_VCF_CALL_TOOL_ID,
            "called_vcf",
        )
        .expect("resolve governed vcf bam smoke contract");

        assert_eq!(contract.stage_id, "vcf.call");
        assert_eq!(contract.tool_id, "bcftools");
        assert_eq!(contract.corpus_id, "vcf_production_regression");
        assert_eq!(contract.sample_id, "human_like_validation");
        assert_eq!(contract.sample_name, "core-v1-pass");
        assert_eq!(
            contract.input_bam,
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam"
        );
        assert_eq!(
            contract.reference,
            "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta"
        );
    }

    #[test]
    fn parse_output_sample_count_reads_governed_fixture_vcf() {
        let repo_root = repo_root();
        let fixture_vcf =
            repo_root.join("benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf");
        let sample_count = parse_output_sample_count(&fixture_vcf).expect("parse sample count");
        assert_eq!(sample_count, 4);
    }
}
