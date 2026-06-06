use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use super::local_vcf_stage_catalog::{build_vcf_stage_catalog_rows, VcfStageCatalogRow};
use super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_SMOKE_ROOT_PATH: &str = "target/local-smoke/vcf/SMOKE_ROOT.json";
const DEFAULT_VCF_SMOKE_ROOT_DIR: &str = "target/local-smoke/vcf";
const LOCAL_VCF_SMOKE_ROOT_SCHEMA_VERSION: &str = "bijux.bench.local_vcf_smoke_root.v1";
const LOCAL_VCF_SMOKE_ROOT_COMMAND: &str = "bijux-dna bench local render-vcf-smoke-root";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfSmokeRootRow {
    pub(crate) stage_id: String,
    pub(crate) stage_name: String,
    pub(crate) support_status: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) asset_profile_id: String,
    pub(crate) adapter_id: String,
    pub(crate) parser_id: String,
    pub(crate) local_smoke_mode: String,
    pub(crate) expected_outputs: Vec<String>,
    pub(crate) pair_root: String,
    pub(crate) artifacts_root: String,
    pub(crate) result_manifest_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfSmokeRootReport {
    pub(crate) schema_version: &'static str,
    pub(crate) manifest_path: String,
    pub(crate) root_path: String,
    pub(crate) run_id: String,
    pub(crate) repo_revision: String,
    pub(crate) worktree_dirty: bool,
    pub(crate) corpus_id: String,
    pub(crate) created_at: String,
    pub(crate) command: &'static str,
    pub(crate) stage_count: usize,
    pub(crate) tool_pair_count: usize,
    pub(crate) rows: Vec<LocalVcfSmokeRootRow>,
}

pub(crate) fn run_render_vcf_smoke_root(
    args: &parse::BenchLocalRenderVcfSmokeRootArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_smoke_root(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_SMOKE_ROOT_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.manifest_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_smoke_root(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalVcfSmokeRootReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let smoke_root = repo_root.join(DEFAULT_VCF_SMOKE_ROOT_DIR);
    fs::create_dir_all(&smoke_root).with_context(|| format!("create {}", smoke_root.display()))?;
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let repo_revision = git_stdout(repo_root, &["rev-parse", "HEAD"])?;
    let created_at = git_stdout(repo_root, &["log", "-1", "--format=%cI", "HEAD"])?;
    let worktree_dirty =
        !git_stdout(repo_root, &["status", "--short", "--untracked-files=no"])?.trim().is_empty();
    let (corpus_id, rows) = build_vcf_smoke_root_rows(repo_root, &smoke_root)?;
    let run_id = build_run_id(&repo_revision, &corpus_id, &rows);

    let report = LocalVcfSmokeRootReport {
        schema_version: LOCAL_VCF_SMOKE_ROOT_SCHEMA_VERSION,
        manifest_path: path_relative_to_repo(repo_root, &output_path),
        root_path: path_relative_to_repo(repo_root, &smoke_root),
        run_id,
        repo_revision,
        worktree_dirty,
        corpus_id,
        created_at,
        command: LOCAL_VCF_SMOKE_ROOT_COMMAND,
        stage_count: rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len(),
        tool_pair_count: rows.len(),
        rows,
    };

    bijux_dna_infra::atomic_write_json(&output_path, &report)?;
    Ok(report)
}

fn build_vcf_smoke_root_rows(
    repo_root: &Path,
    smoke_root: &Path,
) -> Result<(String, Vec<LocalVcfSmokeRootRow>)> {
    let catalog_rows = build_vcf_stage_catalog_rows()?;
    let matrix_rows = build_vcf_stage_matrix_rows()?;
    let catalog_by_stage_id =
        catalog_rows.into_iter().map(|row| (row.stage_id.clone(), row)).collect::<BTreeMap<_, _>>();
    let corpus_ids = matrix_rows.iter().map(|row| row.corpus_id.clone()).collect::<BTreeSet<_>>();
    if corpus_ids.len() != 1 {
        bail!("VCF smoke root requires exactly one governed corpus id; found {corpus_ids:?}");
    }
    let corpus_id =
        corpus_ids.iter().next().cloned().ok_or_else(|| anyhow!("missing VCF corpus id"))?;

    let rows = matrix_rows
        .into_iter()
        .map(|matrix_row| {
            let catalog_row = catalog_by_stage_id.get(&matrix_row.stage_id).ok_or_else(|| {
                anyhow!(
                    "VCF smoke root is missing catalog metadata for stage `{}`",
                    matrix_row.stage_id
                )
            })?;
            build_vcf_smoke_root_row(repo_root, smoke_root, catalog_row, matrix_row)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((corpus_id, rows))
}

fn build_vcf_smoke_root_row(
    repo_root: &Path,
    smoke_root: &Path,
    catalog_row: &VcfStageCatalogRow,
    matrix_row: super::local_vcf_stage_matrix::VcfStageMatrixRow,
) -> Result<LocalVcfSmokeRootRow> {
    if catalog_row.local_smoke_mode != matrix_row.asset_profile_id {
        bail!(
            "VCF smoke root drifted for stage `{}`: catalog smoke mode `{}` != matrix asset profile `{}`",
            catalog_row.stage_id,
            catalog_row.local_smoke_mode,
            matrix_row.asset_profile_id
        );
    }

    let pair_root = smoke_root.join(&matrix_row.stage_id).join(&matrix_row.tool_id);
    let artifacts_root = pair_root.join("artifacts");
    let result_manifest_path = pair_root.join("stage-result.json");

    Ok(LocalVcfSmokeRootRow {
        stage_id: matrix_row.stage_id,
        stage_name: catalog_row.stage_name.clone(),
        support_status: catalog_row.support_status.clone(),
        tool_id: matrix_row.tool_id,
        corpus_id: matrix_row.corpus_id,
        asset_profile_id: matrix_row.asset_profile_id,
        adapter_id: matrix_row.adapter_id,
        parser_id: matrix_row.parser_id,
        local_smoke_mode: catalog_row.local_smoke_mode.clone(),
        expected_outputs: matrix_row.expected_outputs,
        pair_root: path_relative_to_repo(repo_root, &pair_root),
        artifacts_root: path_relative_to_repo(repo_root, &artifacts_root),
        result_manifest_path: path_relative_to_repo(repo_root, &result_manifest_path),
    })
}

fn build_run_id(repo_revision: &str, corpus_id: &str, rows: &[LocalVcfSmokeRootRow]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(LOCAL_VCF_SMOKE_ROOT_SCHEMA_VERSION.as_bytes());
    hasher.update(b"\n");
    hasher.update(repo_revision.as_bytes());
    hasher.update(b"\n");
    hasher.update(corpus_id.as_bytes());
    hasher.update(b"\n");
    for row in rows {
        hasher.update(row.stage_id.as_bytes());
        hasher.update(b"\n");
        hasher.update(row.tool_id.as_bytes());
        hasher.update(b"\n");
        hasher.update(row.pair_root.as_bytes());
        hasher.update(b"\n");
        hasher.update(row.result_manifest_path.as_bytes());
        hasher.update(b"\n");
    }
    let digest = sha256_hex(&hasher.finalize());
    format!("vcf-local-smoke-{}", &digest[..12])
}

fn git_stdout(repo_root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(args)
        .output()
        .with_context(|| format!("run git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!("git {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(String::from_utf8(output.stdout).context("decode git stdout")?.trim().to_string())
}

fn sha256_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
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
        render_vcf_smoke_root, DEFAULT_VCF_SMOKE_ROOT_PATH, LOCAL_VCF_SMOKE_ROOT_COMMAND,
        LOCAL_VCF_SMOKE_ROOT_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_smoke_root_tracks_governed_stage_tool_paths() {
        let repo_root = repo_root();
        let report = render_vcf_smoke_root(&repo_root, PathBuf::from(DEFAULT_VCF_SMOKE_ROOT_PATH))
            .expect("render VCF smoke root");

        assert_eq!(report.schema_version, LOCAL_VCF_SMOKE_ROOT_SCHEMA_VERSION);
        assert_eq!(report.manifest_path, DEFAULT_VCF_SMOKE_ROOT_PATH);
        assert_eq!(report.root_path, "target/local-smoke/vcf");
        assert_eq!(report.command, LOCAL_VCF_SMOKE_ROOT_COMMAND);
        assert_eq!(report.corpus_id, "vcf_production_regression");
        assert_eq!(report.stage_count, 20);
        assert_eq!(report.tool_pair_count, 20);
        assert!(report.run_id.starts_with("vcf-local-smoke-"));
        assert_eq!(report.repo_revision.len(), 40);
        assert!(report.created_at.ends_with('Z') || report.created_at.contains('+'));

        let prepare_reference_panel = report
            .rows
            .iter()
            .find(|row| row.stage_id == "vcf.prepare_reference_panel")
            .expect("prepare reference panel row");
        assert_eq!(prepare_reference_panel.tool_id, "bcftools");
        assert_eq!(prepare_reference_panel.asset_profile_id, "vcf_reference_panel");
        assert_eq!(
            prepare_reference_panel.pair_root,
            "target/local-smoke/vcf/vcf.prepare_reference_panel/bcftools"
        );
        assert_eq!(
            prepare_reference_panel.artifacts_root,
            "target/local-smoke/vcf/vcf.prepare_reference_panel/bcftools/artifacts"
        );
        assert_eq!(
            prepare_reference_panel.result_manifest_path,
            "target/local-smoke/vcf/vcf.prepare_reference_panel/bcftools/stage-result.json"
        );

        let phasing =
            report.rows.iter().find(|row| row.stage_id == "vcf.phasing").expect("phasing row");
        assert_eq!(phasing.tool_id, "shapeit5");
        assert_eq!(phasing.local_smoke_mode, "vcf_cohort_with_panel");
        assert_eq!(phasing.asset_profile_id, "vcf_cohort_with_panel");
        assert_eq!(phasing.expected_outputs, vec!["phased_vcf".to_string()]);
    }

    #[test]
    fn vcf_smoke_root_keeps_governed_root_when_manifest_path_is_redirected() {
        let repo_root = repo_root();
        let report = render_vcf_smoke_root(
            &repo_root,
            PathBuf::from("artifacts/test-output/local-vcf-smoke-root.json"),
        )
        .expect("render VCF smoke root with redirected manifest");

        assert_eq!(report.manifest_path, "artifacts/test-output/local-vcf-smoke-root.json");
        assert_eq!(report.root_path, "target/local-smoke/vcf");
        assert!(report.rows.iter().all(|row| row.pair_root.starts_with("target/local-smoke/vcf/")));
    }
}
