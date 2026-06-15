use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

use super::vcf_bcftools_adapter::render_vcf_bcftools_adapter;
use super::vcf_eigensoft_adapter::render_vcf_eigensoft_adapter;
use super::vcf_imputation_family_adapter::render_vcf_imputation_family_adapter;
use super::vcf_phasing_family_adapter::render_vcf_phasing_family_adapter;
use super::vcf_plink_family_adapter::render_vcf_plink_family_adapter;
use super::vcf_readiness_inputs::load_governed_vcf_fixture_inputs;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH: &str =
    "benchmarks/readiness/vcf-adapter-missing-input-tests.json";
const VCF_ADAPTER_MISSING_INPUT_TESTS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_adapter_missing_input_tests.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfAdapterMissingInputTestRow {
    pub(crate) contract_surface: String,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) benchmark_status: String,
    pub(crate) missing_input_role: String,
    pub(crate) artifact_id: String,
    pub(crate) artifact_path: String,
    pub(crate) expected_error_fragment: String,
    pub(crate) observed_error: String,
    pub(crate) passed: bool,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfAdapterMissingInputTestsReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) passed_row_count: usize,
    pub(crate) failed_row_count: usize,
    pub(crate) adapter_row_count: usize,
    pub(crate) support_row_count: usize,
    pub(crate) rows: Vec<VcfAdapterMissingInputTestRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProbeInput {
    artifact_id: String,
    role: String,
    path: String,
}

pub(crate) fn run_render_vcf_adapter_missing_input_tests(
    args: &parse::BenchReadinessRenderVcfAdapterMissingInputTestsArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_adapter_missing_input_tests(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_adapter_missing_input_tests(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfAdapterMissingInputTestsReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_adapter_missing_input_test_rows(repo_root)?;
    ensure_required_role_coverage(&rows)?;
    let passed_row_count = rows.iter().filter(|row| row.passed).count();
    let failed_row_count = rows.len().saturating_sub(passed_row_count);
    let adapter_row_count =
        rows.iter().filter(|row| row.contract_surface == "adapter_contract").count();
    let support_row_count =
        rows.iter().filter(|row| row.contract_surface != "adapter_contract").count();
    if failed_row_count != 0 {
        let failed_roles = rows
            .iter()
            .filter(|row| !row.passed)
            .map(|row| format!("{}:{}:{}", row.stage_id, row.tool_id, row.missing_input_role))
            .collect::<Vec<_>>();
        return Err(anyhow!(
            "VCF missing-input tests must pass for every governed role, failed rows: {}",
            failed_roles.join(", ")
        ));
    }

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let report = VcfAdapterMissingInputTestsReport {
        schema_version: VCF_ADAPTER_MISSING_INPUT_TESTS_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        passed_row_count,
        failed_row_count,
        adapter_row_count,
        support_row_count,
        rows,
    };
    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

fn collect_vcf_adapter_missing_input_test_rows(
    repo_root: &Path,
) -> Result<Vec<VcfAdapterMissingInputTestRow>> {
    let temp_root = repo_root.join("artifacts/bench-readiness/vcf-missing-input-tests");
    fs::create_dir_all(&temp_root).with_context(|| format!("create {}", temp_root.display()))?;

    let bcftools_report =
        render_vcf_bcftools_adapter(repo_root, temp_root.join("bcftools.adapter.json"))?;
    let plink2_report = render_vcf_plink_family_adapter(
        repo_root,
        "plink2",
        temp_root.join("plink2.adapter.json"),
    )?;
    let phasing_report = render_vcf_phasing_family_adapter(
        repo_root,
        "shapeit5",
        temp_root.join("shapeit5.adapter.json"),
    )?;
    let _ = render_vcf_eigensoft_adapter(repo_root, temp_root.join("eigensoft.adapter.json"))?;
    let _ = render_vcf_imputation_family_adapter(
        repo_root,
        temp_root.join("imputation-family.adapter.json"),
    )?;

    let fixture_inputs = load_governed_vcf_fixture_inputs(repo_root)?;
    let bcftools_call = bcftools_report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.call")
        .ok_or_else(|| anyhow!("VCF bcftools adapter is missing `vcf.call`"))?;
    let shapeit5 = phasing_report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.phasing")
        .ok_or_else(|| anyhow!("VCF shapeit5 adapter is missing `vcf.phasing`"))?;
    let plink2_pca = plink2_report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.pca")
        .ok_or_else(|| anyhow!("VCF plink2 adapter is missing `vcf.pca`"))?;

    let rows = vec![
        build_adapter_probe_row(
            repo_root,
            "bam",
            "vcf.call",
            "bcftools",
            &bcftools_call.benchmark_status,
            "input_bam",
            &bcftools_call.reason,
            bcftools_call
                .required_inputs
                .iter()
                .map(|input| ProbeInput {
                    artifact_id: input.artifact_id.clone(),
                    role: input.role.clone(),
                    path: input.path.clone(),
                })
                .collect(),
        )?,
        build_adapter_probe_row(
            repo_root,
            "bai",
            "vcf.call",
            "bcftools",
            &bcftools_call.benchmark_status,
            "input_bam_index",
            &bcftools_call.reason,
            bcftools_call
                .required_inputs
                .iter()
                .map(|input| ProbeInput {
                    artifact_id: input.artifact_id.clone(),
                    role: input.role.clone(),
                    path: input.path.clone(),
                })
                .collect(),
        )?,
        build_adapter_probe_row(
            repo_root,
            "fasta",
            "vcf.call",
            "bcftools",
            &bcftools_call.benchmark_status,
            "reference_fasta",
            &bcftools_call.reason,
            bcftools_call
                .required_inputs
                .iter()
                .map(|input| ProbeInput {
                    artifact_id: input.artifact_id.clone(),
                    role: input.role.clone(),
                    path: input.path.clone(),
                })
                .collect(),
        )?,
        build_adapter_probe_row(
            repo_root,
            "fai",
            "vcf.call",
            "bcftools",
            &bcftools_call.benchmark_status,
            "reference_fai",
            &bcftools_call.reason,
            bcftools_call
                .required_inputs
                .iter()
                .map(|input| ProbeInput {
                    artifact_id: input.artifact_id.clone(),
                    role: input.role.clone(),
                    path: input.path.clone(),
                })
                .collect(),
        )?,
        build_adapter_probe_row(
            repo_root,
            "vcf",
            "vcf.phasing",
            "shapeit5",
            &shapeit5.benchmark_status,
            "vcf",
            &shapeit5.reason,
            shapeit5
                .required_inputs
                .iter()
                .map(|input| ProbeInput {
                    artifact_id: input.artifact_id.clone(),
                    role: input.role.clone(),
                    path: input.path.clone(),
                })
                .collect(),
        )?,
        build_adapter_probe_row(
            repo_root,
            "vcf_index",
            "vcf.phasing",
            "shapeit5",
            &shapeit5.benchmark_status,
            "vcf_index",
            &shapeit5.reason,
            shapeit5
                .required_inputs
                .iter()
                .map(|input| ProbeInput {
                    artifact_id: input.artifact_id.clone(),
                    role: input.role.clone(),
                    path: input.path.clone(),
                })
                .collect(),
        )?,
        build_adapter_probe_row(
            repo_root,
            "panel_vcf",
            "vcf.phasing",
            "shapeit5",
            &shapeit5.benchmark_status,
            "reference_panel_vcf",
            &shapeit5.reason,
            shapeit5
                .required_inputs
                .iter()
                .map(|input| ProbeInput {
                    artifact_id: input.artifact_id.clone(),
                    role: input.role.clone(),
                    path: input.path.clone(),
                })
                .collect(),
        )?,
        build_adapter_probe_row(
            repo_root,
            "map_file",
            "vcf.phasing",
            "shapeit5",
            &shapeit5.benchmark_status,
            "genetic_map_tsv",
            &shapeit5.reason,
            shapeit5
                .required_inputs
                .iter()
                .map(|input| ProbeInput {
                    artifact_id: input.artifact_id.clone(),
                    role: input.role.clone(),
                    path: input.path.clone(),
                })
                .collect(),
        )?,
        build_adapter_probe_row(
            repo_root,
            "sample_metadata",
            "vcf.pca",
            "plink2",
            &plink2_pca.benchmark_status,
            "sample_metadata_manifest",
            &plink2_pca.reason,
            plink2_pca
                .required_inputs
                .iter()
                .map(|input| ProbeInput {
                    artifact_id: input.artifact_id.clone(),
                    role: input.role.clone(),
                    path: input.path.clone(),
                })
                .collect(),
        )?,
        build_support_probe_row(
            repo_root,
            "sites_bed",
            "target_sites_bed",
            &fixture_inputs.target_sites_bed_path,
            "VCF target-sites BED is governed by the owned corpus fixture contract because no retained VCF adapter consumes it directly",
        ),
    ];
    Ok(rows)
}

fn build_adapter_probe_row(
    repo_root: &Path,
    missing_input_role: &str,
    stage_id: &str,
    tool_id: &str,
    benchmark_status: &str,
    artifact_id: &str,
    row_reason: &str,
    inputs: Vec<ProbeInput>,
) -> Result<VcfAdapterMissingInputTestRow> {
    let probe =
        inputs.iter().find(|input| input.artifact_id == artifact_id).cloned().ok_or_else(|| {
            anyhow!("VCF {stage_id} / {tool_id} row is missing required probe `{artifact_id}`")
        })?;
    let (expected_error_fragment, observed_error, passed) =
        run_missing_input_probe(repo_root, stage_id, &inputs, artifact_id);

    Ok(VcfAdapterMissingInputTestRow {
        contract_surface: "adapter_contract".to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        benchmark_status: benchmark_status.to_string(),
        missing_input_role: missing_input_role.to_string(),
        artifact_id: probe.artifact_id,
        artifact_path: probe.path,
        expected_error_fragment,
        observed_error,
        passed,
        reason: format!(
            "{row_reason}; Goal 243 probes `{artifact_id}` on this retained adapter surface before tool execution"
        ),
    })
}

fn build_support_probe_row(
    repo_root: &Path,
    missing_input_role: &str,
    artifact_id: &str,
    artifact_path: &str,
    reason: &str,
) -> VcfAdapterMissingInputTestRow {
    let inputs = vec![ProbeInput {
        artifact_id: artifact_id.to_string(),
        role: missing_input_role.to_string(),
        path: artifact_path.to_string(),
    }];
    let (expected_error_fragment, observed_error, passed) =
        run_missing_input_probe(repo_root, "vcf.corpus_fixture", &inputs, artifact_id);
    VcfAdapterMissingInputTestRow {
        contract_surface: "fixture_support".to_string(),
        stage_id: "vcf.corpus_fixture".to_string(),
        tool_id: "fixture_contract".to_string(),
        benchmark_status: "support_required".to_string(),
        missing_input_role: missing_input_role.to_string(),
        artifact_id: artifact_id.to_string(),
        artifact_path: artifact_path.to_string(),
        expected_error_fragment,
        observed_error,
        passed,
        reason: reason.to_string(),
    }
}

fn run_missing_input_probe(
    repo_root: &Path,
    stage_id: &str,
    inputs: &[ProbeInput],
    artifact_id: &str,
) -> (String, String, bool) {
    let mut mutated_inputs = inputs.to_vec();
    if let Some(probe) = mutated_inputs.iter_mut().find(|input| input.artifact_id == artifact_id) {
        probe.path = format!(
            "artifacts/bench-readiness/adapters/probes/{stage_id}/{}.missing",
            probe.artifact_id
        );
    }
    let expected_error_fragment = format!("required input `{artifact_id}`");
    let observed_error = match validate_required_inputs(repo_root, stage_id, &mutated_inputs) {
        Ok(()) => format!(
            "VCF missing-input probe unexpectedly accepted `{artifact_id}` for `{stage_id}`"
        ),
        Err(error) => error.to_string(),
    };
    let passed = observed_error.contains(&expected_error_fragment);
    (expected_error_fragment, observed_error, passed)
}

fn validate_required_inputs(repo_root: &Path, stage_id: &str, inputs: &[ProbeInput]) -> Result<()> {
    for input in inputs {
        let resolved = repo_relative_path(repo_root, Path::new(&input.path));
        if !resolved.is_file() {
            return Err(anyhow!(
                "required input `{}` for VCF stage `{}` does not exist at {}",
                input.artifact_id,
                stage_id,
                input.path
            ));
        }
    }
    Ok(())
}

fn ensure_required_role_coverage(rows: &[VcfAdapterMissingInputTestRow]) -> Result<()> {
    let expected_roles = [
        "bam",
        "bai",
        "fasta",
        "fai",
        "vcf",
        "vcf_index",
        "sites_bed",
        "panel_vcf",
        "map_file",
        "sample_metadata",
    ];
    let observed_roles =
        rows.iter().map(|row| row.missing_input_role.as_str()).collect::<BTreeSet<_>>();
    let expected_role_set = expected_roles.into_iter().collect::<BTreeSet<_>>();
    if observed_roles != expected_role_set {
        return Err(anyhow!(
            "VCF missing-input test role coverage drifted: expected {expected_role_set:?}, found {observed_roles:?}"
        ));
    }
    Ok(())
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_vcf_adapter_missing_input_tests, DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH,
        VCF_ADAPTER_MISSING_INPUT_TESTS_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_adapter_missing_input_tests_track_required_roles() {
        let repo_root = repo_root();
        let report = render_vcf_adapter_missing_input_tests(
            &repo_root,
            repo_root.join(DEFAULT_VCF_ADAPTER_MISSING_INPUT_TESTS_PATH),
        )
        .expect("render report");

        assert_eq!(report.schema_version, VCF_ADAPTER_MISSING_INPUT_TESTS_SCHEMA_VERSION);
        assert_eq!(report.row_count, 10);
        assert_eq!(report.passed_row_count, 10);
        assert_eq!(report.failed_row_count, 0);
        assert_eq!(report.adapter_row_count, 9);
        assert_eq!(report.support_row_count, 1);
        assert!(report.rows.iter().any(|row| {
            row.missing_input_role == "sites_bed"
                && row.contract_surface == "fixture_support"
                && row.passed
        }));
        assert!(report.rows.iter().any(|row| {
            row.missing_input_role == "vcf_index"
                && row.stage_id == "vcf.phasing"
                && row.tool_id == "shapeit5"
                && row.passed
        }));
    }
}
