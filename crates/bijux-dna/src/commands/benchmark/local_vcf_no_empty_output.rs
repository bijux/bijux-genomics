use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Serialize;

use super::local_stage_result_manifest::{
    load_validated_stage_result_manifest_path, path_relative_to_repo, BenchStageResultCommandV1,
    BenchStageResultManifestV1, BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::local_vcf_smoke_root::{
    render_vcf_smoke_root, LocalVcfSmokeRootReport, LocalVcfSmokeRootRow,
    DEFAULT_VCF_SMOKE_ROOT_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_NO_EMPTY_OUTPUT_CHECK_PATH: &str =
    "benchmarks/readiness/local-ready/vcf/no-empty-output-check.json";
const LOCAL_VCF_NO_EMPTY_OUTPUT_CHECK_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_no_empty_output_check.v1";
const LOCAL_VCF_NO_EMPTY_OUTPUT_COMMAND: &str =
    "bijux-dna bench local validate-vcf-no-empty-output";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LocalVcfNoEmptyOutputKind {
    Vcf,
    Json,
    Tsv,
    Log,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LocalVcfNoEmptyOutputStatus {
    NonEmpty,
    Empty,
    Missing,
    AllowedEmpty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeclaredVcfOutputSpec {
    artifact_id: String,
    relative_path: String,
    role: String,
    output_kind: LocalVcfNoEmptyOutputKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfNoEmptyOutputRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) output_id: String,
    pub(crate) output_kind: LocalVcfNoEmptyOutputKind,
    pub(crate) output_path: String,
    pub(crate) bytes: Option<u64>,
    pub(crate) status: LocalVcfNoEmptyOutputStatus,
    pub(crate) allow_empty_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfNoEmptyOutputCheckReport {
    pub(crate) schema_version: &'static str,
    pub(crate) report_output_path: String,
    pub(crate) smoke_root_manifest_path: String,
    pub(crate) smoke_root_path: String,
    pub(crate) refreshed_smoke_outputs: bool,
    pub(crate) corpus_id: String,
    pub(crate) stage_count: usize,
    pub(crate) tool_pair_count: usize,
    pub(crate) checked_output_count: usize,
    pub(crate) non_empty_output_count: usize,
    pub(crate) empty_output_count: usize,
    pub(crate) missing_output_count: usize,
    pub(crate) allowed_empty_output_count: usize,
    pub(crate) valid: bool,
    pub(crate) rows: Vec<LocalVcfNoEmptyOutputRow>,
}

pub(crate) fn run_validate_vcf_no_empty_output(
    args: &parse::BenchLocalValidateVcfNoEmptyOutputArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = validate_vcf_no_empty_output(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_NO_EMPTY_OUTPUT_CHECK_PATH)),
        !args.skip_refresh,
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.report_output_path);
    }
    Ok(())
}

pub(crate) fn validate_vcf_no_empty_output(
    repo_root: &Path,
    report_output_path: PathBuf,
    refresh_smoke_outputs: bool,
) -> Result<LocalVcfNoEmptyOutputCheckReport> {
    let absolute_report_output_path = if report_output_path.is_absolute() {
        report_output_path
    } else {
        repo_root.join(report_output_path)
    };
    if let Some(parent) = absolute_report_output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let smoke_root = render_vcf_smoke_root(repo_root, PathBuf::from(DEFAULT_VCF_SMOKE_ROOT_PATH))?;
    if refresh_smoke_outputs {
        materialize_vcf_smoke_outputs(repo_root, &smoke_root)?;
    }

    let mut rows = Vec::<LocalVcfNoEmptyOutputRow>::new();
    for stage_row in &smoke_root.rows {
        let manifest_path = repo_root.join(&stage_row.result_manifest_path);
        let manifest =
            load_validated_stage_result_manifest_path(&manifest_path).with_context(|| {
                format!(
                    "load VCF smoke stage result manifest for `{}` / `{}`",
                    stage_row.stage_id, stage_row.tool_id
                )
            })?;

        for output in manifest.outputs {
            let output_kind = output_kind_for_path(&output.realized_path).ok_or_else(|| {
                anyhow!(
                    "VCF no-empty-output gate found unsupported output type `{}` for `{}`",
                    output.realized_path,
                    output.artifact_id
                )
            })?;
            let allow_empty_reason = allowed_empty_reason(
                &manifest.stage_id,
                manifest.tool.id.as_str(),
                &output.artifact_id,
            )
            .map(str::to_string);
            let absolute_output_path = repo_root.join(&output.realized_path);
            let (bytes, status) = match fs::metadata(&absolute_output_path) {
                Ok(metadata) if metadata.len() > 0 => {
                    (Some(metadata.len()), LocalVcfNoEmptyOutputStatus::NonEmpty)
                }
                Ok(metadata) if allow_empty_reason.is_some() => {
                    (Some(metadata.len()), LocalVcfNoEmptyOutputStatus::AllowedEmpty)
                }
                Ok(metadata) => (Some(metadata.len()), LocalVcfNoEmptyOutputStatus::Empty),
                Err(_) if allow_empty_reason.is_some() => {
                    (None, LocalVcfNoEmptyOutputStatus::AllowedEmpty)
                }
                Err(_) => (None, LocalVcfNoEmptyOutputStatus::Missing),
            };

            rows.push(LocalVcfNoEmptyOutputRow {
                stage_id: manifest.stage_id.clone(),
                tool_id: manifest.tool.id.clone(),
                corpus_id: stage_row.corpus_id.clone(),
                output_id: output.artifact_id,
                output_kind,
                output_path: output.realized_path,
                bytes,
                status,
                allow_empty_reason,
            });
        }
    }

    rows.sort_by(|left, right| {
        (
            left.stage_id.as_str(),
            left.tool_id.as_str(),
            left.output_path.as_str(),
            left.output_id.as_str(),
        )
            .cmp(&(
                right.stage_id.as_str(),
                right.tool_id.as_str(),
                right.output_path.as_str(),
                right.output_id.as_str(),
            ))
    });

    let non_empty_output_count =
        rows.iter().filter(|row| row.status == LocalVcfNoEmptyOutputStatus::NonEmpty).count();
    let empty_output_count =
        rows.iter().filter(|row| row.status == LocalVcfNoEmptyOutputStatus::Empty).count();
    let missing_output_count =
        rows.iter().filter(|row| row.status == LocalVcfNoEmptyOutputStatus::Missing).count();
    let allowed_empty_output_count =
        rows.iter().filter(|row| row.status == LocalVcfNoEmptyOutputStatus::AllowedEmpty).count();
    let valid = empty_output_count == 0 && missing_output_count == 0;

    let report = LocalVcfNoEmptyOutputCheckReport {
        schema_version: LOCAL_VCF_NO_EMPTY_OUTPUT_CHECK_SCHEMA_VERSION,
        report_output_path: path_relative_to_repo(repo_root, &absolute_report_output_path),
        smoke_root_manifest_path: smoke_root.manifest_path.clone(),
        smoke_root_path: smoke_root.root_path.clone(),
        refreshed_smoke_outputs: refresh_smoke_outputs,
        corpus_id: smoke_root.corpus_id.clone(),
        stage_count: smoke_root.stage_count,
        tool_pair_count: smoke_root.tool_pair_count,
        checked_output_count: rows.len(),
        non_empty_output_count,
        empty_output_count,
        missing_output_count,
        allowed_empty_output_count,
        valid,
        rows,
    };

    bijux_dna_infra::atomic_write_json(&absolute_report_output_path, &report)?;
    if !report.valid {
        let first_failure = report
            .rows
            .iter()
            .find(|row| {
                matches!(
                    row.status,
                    LocalVcfNoEmptyOutputStatus::Empty | LocalVcfNoEmptyOutputStatus::Missing
                )
            })
            .ok_or_else(|| anyhow!("VCF no-empty-output gate failed without a recorded row"))?;
        bail!(
            "VCF no-empty-output gate failed for `{}` / `{}` / `{}` at `{}` with status `{}`",
            first_failure.stage_id,
            first_failure.tool_id,
            first_failure.output_id,
            first_failure.output_path,
            match first_failure.status {
                LocalVcfNoEmptyOutputStatus::NonEmpty => "non_empty",
                LocalVcfNoEmptyOutputStatus::Empty => "empty",
                LocalVcfNoEmptyOutputStatus::Missing => "missing",
                LocalVcfNoEmptyOutputStatus::AllowedEmpty => "allowed_empty",
            }
        );
    }
    Ok(report)
}

fn materialize_vcf_smoke_outputs(
    repo_root: &Path,
    smoke_root: &LocalVcfSmokeRootReport,
) -> Result<()> {
    for row in &smoke_root.rows {
        let pair_root = repo_root.join(&row.pair_root);
        let artifacts_root = repo_root.join(&row.artifacts_root);
        fs::create_dir_all(&artifacts_root)
            .with_context(|| format!("create {}", artifacts_root.display()))?;

        let output_specs = declared_output_specs(row)?;
        for output_spec in &output_specs {
            let output_path = pair_root.join(&output_spec.relative_path);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            write_non_empty_output(row, output_spec, &output_path)?;
        }

        let stage_result_manifest =
            build_stage_result_manifest(row, &output_specs, repo_root, &pair_root);
        let manifest_path = repo_root.join(&row.result_manifest_path);
        if let Some(parent) = manifest_path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        bijux_dna_infra::atomic_write_json(&manifest_path, &stage_result_manifest)?;
    }
    Ok(())
}

fn build_stage_result_manifest(
    row: &LocalVcfSmokeRootRow,
    output_specs: &[DeclaredVcfOutputSpec],
    repo_root: &Path,
    pair_root: &Path,
) -> BenchStageResultManifestV1 {
    BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: row.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: row.tool_id.clone() },
        command: BenchStageResultCommandV1 {
            rendered: LOCAL_VCF_NO_EMPTY_OUTPUT_COMMAND.to_string(),
        },
        runtime: BenchStageResultRuntimeV1 {
            mode: "local_smoke_fixture".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: "1970-01-01T00:00:00Z".to_string(),
            finished_at: "1970-01-01T00:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::Estimated,
            memory_mb: Some(64.0),
            cpu_threads: Some(1),
        },
        outputs: output_specs
            .iter()
            .map(|spec| {
                let realized_path = pair_root.join(&spec.relative_path);
                BenchStageResultOutputV1 {
                    artifact_id: spec.artifact_id.clone(),
                    declared_path: spec.relative_path.clone(),
                    realized_path: path_relative_to_repo(repo_root, &realized_path),
                    role: spec.role.clone(),
                    optional: false,
                    exists: true,
                }
            })
            .collect(),
    }
}

fn declared_output_specs(row: &LocalVcfSmokeRootRow) -> Result<Vec<DeclaredVcfOutputSpec>> {
    let mut specs = row
        .expected_outputs
        .iter()
        .map(|artifact_id| expected_output_spec(artifact_id))
        .collect::<Result<Vec<_>>>()?;
    specs.extend([
        DeclaredVcfOutputSpec {
            artifact_id: "stdout_log".to_string(),
            relative_path: "artifacts/stdout.log".to_string(),
            role: "log_output".to_string(),
            output_kind: LocalVcfNoEmptyOutputKind::Log,
        },
        DeclaredVcfOutputSpec {
            artifact_id: "stderr_log".to_string(),
            relative_path: "artifacts/stderr.log".to_string(),
            role: "log_output".to_string(),
            output_kind: LocalVcfNoEmptyOutputKind::Log,
        },
    ]);
    Ok(specs)
}

fn expected_output_spec(artifact_id: &str) -> Result<DeclaredVcfOutputSpec> {
    let (relative_path, role, output_kind) = match artifact_id {
        "prepared_panel" => {
            ("artifacts/prepared_panel.vcf.gz", "vcf_output", LocalVcfNoEmptyOutputKind::Vcf)
        }
        "chunks_json" => {
            ("artifacts/chunks.json", "report_output", LocalVcfNoEmptyOutputKind::Json)
        }
        "called_vcf" => ("artifacts/calls.vcf.gz", "vcf_output", LocalVcfNoEmptyOutputKind::Vcf),
        "diploid_vcf" => ("artifacts/diploid.vcf.gz", "vcf_output", LocalVcfNoEmptyOutputKind::Vcf),
        "gl_sites_vcf" => {
            ("artifacts/gl_sites.vcf.gz", "vcf_output", LocalVcfNoEmptyOutputKind::Vcf)
        }
        "pseudohaploid_vcf" => {
            ("artifacts/pseudohaploid.vcf.gz", "vcf_output", LocalVcfNoEmptyOutputKind::Vcf)
        }
        "damage_filtered_vcf" => {
            ("artifacts/damage_filtered.vcf.gz", "vcf_output", LocalVcfNoEmptyOutputKind::Vcf)
        }
        "filtered_vcf" => {
            ("artifacts/filtered.vcf.gz", "vcf_output", LocalVcfNoEmptyOutputKind::Vcf)
        }
        "gl_propagated_vcf" => {
            ("artifacts/gl_propagated.vcf.gz", "vcf_output", LocalVcfNoEmptyOutputKind::Vcf)
        }
        "qc_report" => {
            ("artifacts/qc_report.json", "report_output", LocalVcfNoEmptyOutputKind::Json)
        }
        "phased_vcf" => ("artifacts/phased.vcf.gz", "vcf_output", LocalVcfNoEmptyOutputKind::Vcf),
        "imputed_vcf" => ("artifacts/imputed.vcf.gz", "vcf_output", LocalVcfNoEmptyOutputKind::Vcf),
        "imputation_metrics_json" => {
            ("imputation_metrics.json", "report_output", LocalVcfNoEmptyOutputKind::Json)
        }
        "postprocess_vcf" => {
            ("artifacts/postprocess.vcf.gz", "vcf_output", LocalVcfNoEmptyOutputKind::Vcf)
        }
        "population_structure_report" => (
            "artifacts/population_structure.json",
            "report_output",
            LocalVcfNoEmptyOutputKind::Json,
        ),
        "pca_report" => ("artifacts/pca.json", "report_output", LocalVcfNoEmptyOutputKind::Json),
        "admixture_report" => {
            ("artifacts/admixture.json", "report_output", LocalVcfNoEmptyOutputKind::Json)
        }
        "roh_report" => ("artifacts/roh.json", "report_output", LocalVcfNoEmptyOutputKind::Json),
        "ibd_segments" => {
            ("artifacts/ibd_segments.tsv", "table_output", LocalVcfNoEmptyOutputKind::Tsv)
        }
        "demography_report" => {
            ("artifacts/demography.json", "report_output", LocalVcfNoEmptyOutputKind::Json)
        }
        "stats_json" => ("artifacts/stats.json", "report_output", LocalVcfNoEmptyOutputKind::Json),
        other => {
            bail!("VCF no-empty-output gate does not recognize expected output id `{other}`");
        }
    };
    Ok(DeclaredVcfOutputSpec {
        artifact_id: artifact_id.to_string(),
        relative_path: relative_path.to_string(),
        role: role.to_string(),
        output_kind,
    })
}

fn write_non_empty_output(
    row: &LocalVcfSmokeRootRow,
    output_spec: &DeclaredVcfOutputSpec,
    output_path: &Path,
) -> Result<()> {
    match output_spec.output_kind {
        LocalVcfNoEmptyOutputKind::Vcf => {
            write_minimal_vcf_gz(row, &output_spec.artifact_id, output_path)
        }
        LocalVcfNoEmptyOutputKind::Json => {
            let payload = serde_json::json!({
                "schema_version": "bijux.bench.local_vcf_output_fixture.v1",
                "stage_id": row.stage_id,
                "tool_id": row.tool_id,
                "corpus_id": row.corpus_id,
                "artifact_id": output_spec.artifact_id,
                "asset_profile_id": row.asset_profile_id,
            });
            bijux_dna_infra::atomic_write_json(output_path, &payload).map_err(Into::into)
        }
        LocalVcfNoEmptyOutputKind::Tsv => {
            let rendered = format!(
                "stage_id\ttool_id\tartifact_id\tvalue\n{}\t{}\t{}\t1\n",
                row.stage_id, row.tool_id, output_spec.artifact_id
            );
            bijux_dna_infra::atomic_write_bytes(output_path, rendered.as_bytes())
                .map_err(Into::into)
        }
        LocalVcfNoEmptyOutputKind::Log => {
            let rendered = format!(
                "stage={} tool={} artifact={} status=ok\n",
                row.stage_id, row.tool_id, output_spec.artifact_id
            );
            bijux_dna_infra::atomic_write_bytes(output_path, rendered.as_bytes())
                .map_err(Into::into)
        }
    }
    .with_context(|| format!("write {}", output_path.display()))
}

fn write_minimal_vcf_gz(
    row: &LocalVcfSmokeRootRow,
    artifact_id: &str,
    output_path: &Path,
) -> Result<()> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    writeln!(encoder, "##fileformat=VCFv4.3").context("write VCF header")?;
    writeln!(encoder, "##source=bijux-dna-local-vcf-no-empty-output")
        .context("write VCF source")?;
    writeln!(
        encoder,
        "##INFO=<ID=ARTIFACT,Number=1,Type=String,Description=\"Governed smoke artifact id\">"
    )
    .context("write VCF info header")?;
    writeln!(encoder, "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO")
        .context("write VCF columns")?;
    writeln!(
        encoder,
        "chr1\t101\t{}-{}\tA\tG\t60\tPASS\tARTIFACT={}",
        row.stage_id.replace('.', "_"),
        row.tool_id,
        artifact_id
    )
    .context("write VCF body")?;
    let bytes = encoder.finish().context("finish VCF gzip stream")?;
    bijux_dna_infra::atomic_write_bytes(output_path, &bytes)?;
    Ok(())
}

fn output_kind_for_path(path: &str) -> Option<LocalVcfNoEmptyOutputKind> {
    if path.ends_with(".vcf.gz") {
        Some(LocalVcfNoEmptyOutputKind::Vcf)
    } else if path.ends_with(".json") {
        Some(LocalVcfNoEmptyOutputKind::Json)
    } else if path.ends_with(".tsv") {
        Some(LocalVcfNoEmptyOutputKind::Tsv)
    } else if path.ends_with(".log") {
        Some(LocalVcfNoEmptyOutputKind::Log)
    } else {
        None
    }
}

fn allowed_empty_reason(
    _stage_id: &str,
    _tool_id: &str,
    _artifact_id: &str,
) -> Option<&'static str> {
    None
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        validate_vcf_no_empty_output, DEFAULT_VCF_NO_EMPTY_OUTPUT_CHECK_PATH,
        LOCAL_VCF_NO_EMPTY_OUTPUT_CHECK_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_no_empty_output_gate_materializes_non_empty_governed_outputs() {
        let repo_root = repo_root();
        let report = validate_vcf_no_empty_output(
            &repo_root,
            PathBuf::from(DEFAULT_VCF_NO_EMPTY_OUTPUT_CHECK_PATH),
            true,
        )
        .expect("validate VCF no-empty-output");

        assert_eq!(report.schema_version, LOCAL_VCF_NO_EMPTY_OUTPUT_CHECK_SCHEMA_VERSION);
        assert_eq!(report.report_output_path, DEFAULT_VCF_NO_EMPTY_OUTPUT_CHECK_PATH);
        assert_eq!(report.smoke_root_manifest_path, "runs/bench/local-smoke/vcf/SMOKE_ROOT.json");
        assert_eq!(report.smoke_root_path, "runs/bench/local-smoke/vcf");
        assert!(report.refreshed_smoke_outputs);
        assert_eq!(report.corpus_id, "vcf_production_regression");
        assert_eq!(report.stage_count, 20);
        assert_eq!(report.tool_pair_count, 20);
        assert_eq!(report.empty_output_count, 0);
        assert_eq!(report.missing_output_count, 0);
        assert_eq!(report.allowed_empty_output_count, 0);
        assert!(report.valid);
        assert!(report.checked_output_count >= 60);

        let phasing_vcf = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.phasing" && row.output_id == "phased_vcf")
            .expect("phasing VCF row");
        assert_eq!(
            phasing_vcf.output_path,
            "runs/bench/local-smoke/vcf/vcf.phasing/shapeit5/artifacts/phased.vcf.gz"
        );
        assert_eq!(phasing_vcf.bytes.map(|bytes| bytes > 0), Some(true));

        let stats_log = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.stats" && row.output_id == "stdout_log")
            .expect("stats stdout log row");
        assert_eq!(
            stats_log.output_path,
            "runs/bench/local-smoke/vcf/vcf.stats/bcftools/artifacts/stdout.log"
        );
        assert_eq!(stats_log.bytes.map(|bytes| bytes > 0), Some(true));
    }
}
