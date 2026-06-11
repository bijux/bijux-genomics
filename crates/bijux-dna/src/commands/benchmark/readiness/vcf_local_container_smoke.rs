use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use super::vcf_active_stage_tool_matrix::{
    collect_vcf_active_stage_tool_matrix_rows, VcfActiveStageToolMatrixRow,
};
use crate::commands::benchmark::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH: &str =
    "benchmarks/readiness/vcf/vcf-local-container-smoke.tsv";
const VCF_LOCAL_CONTAINER_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_local_container_smoke.v1";

const VCF_REGISTRY_PATHS: &[&str] = &[
    "configs/ci/registry/tool_registry_vcf.toml",
    "configs/ci/registry/tool_registry_vcf_downstream.toml",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VcfLocalContainerSmokeRow {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) registered_binary: String,
    pub(crate) tool_status: String,
    pub(crate) stage_support_status: String,
    pub(crate) scope_state: String,
    pub(crate) scope_detail: String,
    pub(crate) smoke_path_kind: String,
    pub(crate) smoke_runtime: String,
    pub(crate) smoke_tool_id: String,
    pub(crate) smoke_command: String,
    pub(crate) smoke_support_path: String,
    pub(crate) smoke_minimal_cmd: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfLocalContainerSmokeReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) row_count: usize,
    pub(crate) stage_count: usize,
    pub(crate) tool_count: usize,
    pub(crate) host_stage_smoke_row_count: usize,
    pub(crate) container_smoke_row_count: usize,
    pub(crate) runtime_counts: BTreeMap<String, usize>,
    pub(crate) rows: Vec<VcfLocalContainerSmokeRow>,
}

#[derive(Debug, Clone, Default)]
struct VcfRegistrySmokeRecord {
    tool_id: String,
    registered_binary: String,
    dockerfile: String,
    apptainer_def: String,
    smoke_minimal_cmd: String,
    smoke_minimal_rationale: String,
}

pub(crate) fn run_render_vcf_local_container_smoke(
    args: &parse::BenchReadinessRenderVcfLocalContainerSmokeArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_local_container_smoke(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_local_container_smoke(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<VcfLocalContainerSmokeReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = collect_vcf_local_container_smoke_rows(repo_root)?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_vcf_local_container_smoke_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut runtime_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *runtime_counts.entry(row.smoke_runtime.clone()).or_default() += 1;
    }
    let host_stage_smoke_row_count =
        rows.iter().filter(|row| row.smoke_path_kind == "host_stage_smoke").count();
    let container_smoke_row_count = rows.len().saturating_sub(host_stage_smoke_row_count);

    Ok(VcfLocalContainerSmokeReport {
        schema_version: VCF_LOCAL_CONTAINER_SMOKE_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        row_count: rows.len(),
        stage_count: rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>().len(),
        tool_count: rows.iter().map(|row| row.tool_id.as_str()).collect::<BTreeSet<_>>().len(),
        host_stage_smoke_row_count,
        container_smoke_row_count,
        runtime_counts,
        rows,
    })
}

pub(crate) fn collect_vcf_local_container_smoke_rows(
    repo_root: &Path,
) -> Result<Vec<VcfLocalContainerSmokeRow>> {
    let retained_rows = collect_vcf_active_stage_tool_matrix_rows(repo_root)?;
    let stage_matrix_by_stage = build_vcf_stage_matrix_rows()?
        .into_iter()
        .map(|row| (row.stage_id.clone(), row.tool_id))
        .collect::<BTreeMap<_, _>>();
    let registry_by_tool = load_vcf_registry_smoke_records(repo_root)?;

    let mut rows = Vec::with_capacity(retained_rows.len());
    for retained_row in retained_rows {
        rows.push(build_vcf_local_container_smoke_row(
            repo_root,
            &retained_row,
            &stage_matrix_by_stage,
            &registry_by_tool,
        )?);
    }

    rows.sort_by(|left, right| {
        left.stage_id.cmp(&right.stage_id).then_with(|| left.tool_id.cmp(&right.tool_id))
    });
    ensure_vcf_local_container_smoke_contract(repo_root, &rows)?;
    Ok(rows)
}

fn build_vcf_local_container_smoke_row(
    repo_root: &Path,
    retained_row: &VcfActiveStageToolMatrixRow,
    stage_matrix_by_stage: &BTreeMap<String, String>,
    registry_by_tool: &BTreeMap<String, VcfRegistrySmokeRecord>,
) -> Result<VcfLocalContainerSmokeRow> {
    let registry_row = registry_by_tool.get(retained_row.tool_id.as_str()).ok_or_else(|| {
        anyhow!(
            "VCF local-container smoke is missing registry coverage for retained tool `{}`",
            retained_row.tool_id
        )
    })?;

    let is_governed_default = stage_matrix_by_stage
        .get(retained_row.stage_id.as_str())
        .is_some_and(|tool_id| tool_id == &retained_row.tool_id);

    if is_governed_default {
        if let Some(smoke_support_path) = host_smoke_source_path(repo_root, &retained_row.stage_id)?
        {
            let smoke_command = format!(
                "bijux-dna bench local {} --tool-id {}",
                local_smoke_command_name(&retained_row.stage_id)?,
                retained_row.tool_id
            );
            return Ok(VcfLocalContainerSmokeRow {
                stage_id: retained_row.stage_id.clone(),
                tool_id: retained_row.tool_id.clone(),
                registered_binary: registry_row.registered_binary.clone(),
                tool_status: retained_row.tool_status.clone(),
                stage_support_status: retained_row.stage_support_status.clone(),
                scope_state: retained_row.scope_state.clone(),
                scope_detail: retained_row.scope_detail.clone(),
                smoke_path_kind: "host_stage_smoke".to_string(),
                smoke_runtime: "host".to_string(),
                smoke_tool_id: retained_row.tool_id.clone(),
                smoke_command,
                smoke_support_path,
                smoke_minimal_cmd: String::new(),
                reason: format!(
                    "binding `{}` / `{}` matches the governed VCF stage-matrix default tool, so the exact tiny-fixture stage smoke wrapper is available on host",
                    retained_row.stage_id, retained_row.tool_id
                ),
            });
        }
    }

    let (smoke_path_kind, smoke_runtime, smoke_command, smoke_support_path) =
        resolve_container_smoke_wrapper(repo_root, registry_row)?;
    let alias_clause = if is_governed_default {
        format!(
            "binding `{}` / `{}` matches the governed VCF stage-matrix default tool, but no exact tiny-fixture stage smoke wrapper is checked in",
            retained_row.stage_id, retained_row.tool_id
        )
    } else if registry_row.registered_binary != retained_row.tool_id {
        format!(
            "retained tool `{}` resolves through registered binary `{}`",
            retained_row.tool_id, registry_row.registered_binary
        )
    } else {
        format!(
            "retained tool `{}` has no exact tiny-fixture stage smoke wrapper",
            retained_row.tool_id
        )
    };
    let rationale_clause = if registry_row.smoke_minimal_rationale.trim().is_empty() {
        String::new()
    } else {
        format!("; {}", registry_row.smoke_minimal_rationale.trim())
    };

    Ok(VcfLocalContainerSmokeRow {
        stage_id: retained_row.stage_id.clone(),
        tool_id: retained_row.tool_id.clone(),
        registered_binary: registry_row.registered_binary.clone(),
        tool_status: retained_row.tool_status.clone(),
        stage_support_status: retained_row.stage_support_status.clone(),
        scope_state: retained_row.scope_state.clone(),
        scope_detail: retained_row.scope_detail.clone(),
        smoke_path_kind,
        smoke_runtime,
        smoke_tool_id: registry_row.registered_binary.clone(),
        smoke_command,
        smoke_support_path,
        smoke_minimal_cmd: registry_row.smoke_minimal_cmd.clone(),
        reason: format!(
            "{alias_clause}, so the governed container smoke wrapper is the available local exercise path for `{}` / `{}`{rationale_clause}",
            retained_row.stage_id, retained_row.tool_id
        ),
    })
}

fn local_smoke_command_name(stage_id: &str) -> Result<String> {
    let Some(suffix) = stage_id.strip_prefix("vcf.") else {
        bail!("VCF local-container smoke expected a `vcf.*` stage id, found `{stage_id}`");
    };
    Ok(format!("run-vcf-{}-smoke", suffix.replace('_', "-")))
}

fn host_smoke_source_path(repo_root: &Path, stage_id: &str) -> Result<Option<String>> {
    let suffix = stage_id
        .strip_prefix("vcf.")
        .ok_or_else(|| anyhow!("VCF local-container smoke expected a `vcf.*` stage id"))?;
    let relative_path = PathBuf::from(format!(
        "crates/bijux-dna/src/commands/benchmark/local_vcf_{suffix}_smoke.rs"
    ));
    let absolute_path = repo_root.join(&relative_path);
    if !absolute_path.is_file() {
        return Ok(None);
    }
    Ok(Some(path_relative_to_repo(repo_root, &absolute_path)))
}

fn resolve_container_smoke_wrapper(
    repo_root: &Path,
    registry_row: &VcfRegistrySmokeRecord,
) -> Result<(String, String, String, String)> {
    let dockerfile = repo_root.join(&registry_row.dockerfile);
    if !registry_row.dockerfile.trim().is_empty() && dockerfile.is_file() {
        return Ok((
            "docker_container_smoke".to_string(),
            "docker-arm64".to_string(),
            format!("bijux-dna env smoke docker-arm64 {}", registry_row.registered_binary),
            path_relative_to_repo(repo_root, &dockerfile),
        ));
    }
    let apptainer_def = repo_root.join(&registry_row.apptainer_def);
    if !registry_row.apptainer_def.trim().is_empty() && apptainer_def.is_file() {
        return Ok((
            "apptainer_container_smoke".to_string(),
            "apptainer".to_string(),
            format!("bijux-dna env smoke apptainer {}", registry_row.registered_binary),
            path_relative_to_repo(repo_root, &apptainer_def),
        ));
    }

    bail!(
        "VCF local-container smoke could not resolve a checked-in container wrapper for retained tool `{}` (dockerfile=`{}`, apptainer_def=`{}`)",
        registry_row.tool_id,
        registry_row.dockerfile,
        registry_row.apptainer_def
    );
}

fn load_vcf_registry_smoke_records(
    repo_root: &Path,
) -> Result<BTreeMap<String, VcfRegistrySmokeRecord>> {
    let mut records = BTreeMap::<String, VcfRegistrySmokeRecord>::new();

    for relative_path in VCF_REGISTRY_PATHS {
        let absolute_path = repo_root.join(relative_path);
        let raw = fs::read_to_string(&absolute_path)
            .with_context(|| format!("read {}", absolute_path.display()))?;
        let parsed: toml::Value =
            toml::from_str(&raw).with_context(|| format!("parse {}", absolute_path.display()))?;
        let entries = parsed
            .get("tools")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing tools in {}", absolute_path.display()))?;

        for entry in entries {
            let tool_id = entry
                .get("id")
                .and_then(toml::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| anyhow!("tool entry in {} is missing id", absolute_path.display()))?
                .to_string();
            let record = records.entry(tool_id.clone()).or_insert_with(|| VcfRegistrySmokeRecord {
                tool_id: tool_id.clone(),
                ..VcfRegistrySmokeRecord::default()
            });
            merge_registry_string(
                &mut record.registered_binary,
                string_field(entry, "expected_bin"),
                "expected_bin",
                &tool_id,
            )?;
            merge_registry_string(
                &mut record.dockerfile,
                string_field(entry, "dockerfile"),
                "dockerfile",
                &tool_id,
            )?;
            merge_registry_string(
                &mut record.apptainer_def,
                string_field(entry, "apptainer_def"),
                "apptainer_def",
                &tool_id,
            )?;
            merge_registry_string(
                &mut record.smoke_minimal_cmd,
                string_field(entry, "smoke_minimal_cmd"),
                "smoke_minimal_cmd",
                &tool_id,
            )?;
            merge_registry_string(
                &mut record.smoke_minimal_rationale,
                string_field(entry, "smoke_minimal_rationale"),
                "smoke_minimal_rationale",
                &tool_id,
            )?;
        }
    }

    for (tool_id, record) in &records {
        if record.registered_binary.trim().is_empty() {
            bail!("VCF local-container smoke registry row `{tool_id}` is missing expected_bin");
        }
    }
    Ok(records)
}

fn merge_registry_string(
    target: &mut String,
    incoming: String,
    field: &str,
    tool_id: &str,
) -> Result<()> {
    if incoming.trim().is_empty() {
        return Ok(());
    }
    if target.trim().is_empty() {
        *target = incoming;
        return Ok(());
    }
    if target != &incoming {
        return Err(anyhow!(
            "VCF local-container smoke registry field `{field}` drifted for `{tool_id}` (`{target}` vs `{incoming}`)"
        ));
    }
    Ok(())
}

fn string_field(value: &toml::Value, key: &str) -> String {
    value.get(key).and_then(toml::Value::as_str).unwrap_or_default().trim().to_string()
}

fn ensure_vcf_local_container_smoke_contract(
    repo_root: &Path,
    rows: &[VcfLocalContainerSmokeRow],
) -> Result<()> {
    let host_stage_matrix_row_count = build_vcf_stage_matrix_rows()?
        .into_iter()
        .filter(|row| {
            host_smoke_source_path(repo_root, &row.stage_id).is_ok_and(|path| path.is_some())
        })
        .count();
    if rows.len() != 44 {
        return Err(anyhow!(
            "VCF local-container smoke report drifted from the governed retained surface (expected 44 rows, found {})",
            rows.len()
        ));
    }
    let host_stage_smoke_row_count =
        rows.iter().filter(|row| row.smoke_path_kind == "host_stage_smoke").count();
    if host_stage_smoke_row_count != host_stage_matrix_row_count {
        return Err(anyhow!(
            "VCF local-container smoke host wrapper count drifted from the governed VCF stage matrix (expected {}, found {})",
            host_stage_matrix_row_count,
            host_stage_smoke_row_count
        ));
    }

    let expected_rows = [
        (
            "vcf.call",
            "bcftools",
            "bcftools",
            "host_stage_smoke",
            "host",
            "bcftools",
            "bijux-dna bench local run-vcf-call-smoke --tool-id bcftools",
            "crates/bijux-dna/src/commands/benchmark/local_vcf_call_smoke.rs",
            "",
        ),
        (
            "vcf.ibd",
            "germline",
            "germline",
            "host_stage_smoke",
            "host",
            "germline",
            "bijux-dna bench local run-vcf-ibd-smoke --tool-id germline",
            "crates/bijux-dna/src/commands/benchmark/local_vcf_ibd_smoke.rs",
            "",
        ),
        (
            "vcf.imputation_metrics",
            "beagle-imputation",
            "beagle",
            "docker_container_smoke",
            "docker-arm64",
            "beagle",
            "bijux-dna env smoke docker-arm64 beagle",
            "containers/docker/arm64/Dockerfile.beagle",
            "beagle --help",
        ),
        (
            "vcf.impute",
            "glimpse",
            "glimpse",
            "docker_container_smoke",
            "docker-arm64",
            "glimpse",
            "bijux-dna env smoke docker-arm64 glimpse",
            "containers/docker/arm64/Dockerfile.glimpse",
            "glimpse --help",
        ),
        (
            "vcf.postprocess",
            "bcftools",
            "bcftools",
            "docker_container_smoke",
            "docker-arm64",
            "bcftools",
            "bijux-dna env smoke docker-arm64 bcftools",
            "containers/docker/arm64/Dockerfile.bcftools",
            "",
        ),
        (
            "vcf.phasing",
            "shapeit",
            "shapeit",
            "apptainer_container_smoke",
            "apptainer",
            "shapeit",
            "bijux-dna env smoke apptainer shapeit",
            "containers/apptainer/shared/shapeit.def",
            "shapeit --help",
        ),
        (
            "vcf.ibd",
            "ibdseq",
            "ibdseq",
            "apptainer_container_smoke",
            "apptainer",
            "ibdseq",
            "bijux-dna env smoke apptainer ibdseq",
            "containers/apptainer/shared/ibdseq.def",
            "ibdseq --help",
        ),
    ];

    for (
        stage_id,
        tool_id,
        registered_binary,
        smoke_path_kind,
        smoke_runtime,
        smoke_tool_id,
        smoke_command,
        smoke_support_path,
        smoke_minimal_cmd,
    ) in expected_rows
    {
        let row = rows
            .iter()
            .find(|row| row.stage_id == stage_id && row.tool_id == tool_id)
            .ok_or_else(|| {
                anyhow!("VCF local-container smoke report is missing `{stage_id}` / `{tool_id}`")
            })?;
        if row.registered_binary != registered_binary
            || row.smoke_path_kind != smoke_path_kind
            || row.smoke_runtime != smoke_runtime
            || row.smoke_tool_id != smoke_tool_id
            || row.smoke_command != smoke_command
            || row.smoke_support_path != smoke_support_path
            || row.smoke_minimal_cmd != smoke_minimal_cmd
        {
            return Err(anyhow!(
                "VCF local-container smoke row `{stage_id}` / `{tool_id}` drifted from the governed wrapper contract"
            ));
        }
        if !repo_root.join(&row.smoke_support_path).exists() {
            return Err(anyhow!(
                "VCF local-container smoke support path `{}` no longer exists",
                row.smoke_support_path
            ));
        }
    }

    if rows.iter().any(|row| row.smoke_command.trim().is_empty()) {
        return Err(anyhow!("VCF local-container smoke report must not emit empty smoke commands"));
    }

    Ok(())
}

fn render_vcf_local_container_smoke_tsv(rows: &[VcfLocalContainerSmokeRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\tregistered_binary\ttool_status\tstage_support_status\tscope_state\tscope_detail\tsmoke_path_kind\tsmoke_runtime\tsmoke_tool_id\tsmoke_command\tsmoke_support_path\tsmoke_minimal_cmd\treason\n",
    );
    for row in rows {
        rendered.push_str(&row.stage_id);
        rendered.push('\t');
        rendered.push_str(&row.tool_id);
        rendered.push('\t');
        rendered.push_str(&row.registered_binary);
        rendered.push('\t');
        rendered.push_str(&row.tool_status);
        rendered.push('\t');
        rendered.push_str(&row.stage_support_status);
        rendered.push('\t');
        rendered.push_str(&row.scope_state);
        rendered.push('\t');
        rendered.push_str(&row.scope_detail);
        rendered.push('\t');
        rendered.push_str(&row.smoke_path_kind);
        rendered.push('\t');
        rendered.push_str(&row.smoke_runtime);
        rendered.push('\t');
        rendered.push_str(&row.smoke_tool_id);
        rendered.push('\t');
        rendered.push_str(&row.smoke_command);
        rendered.push('\t');
        rendered.push_str(&row.smoke_support_path);
        rendered.push('\t');
        rendered.push_str(&row.smoke_minimal_cmd);
        rendered.push('\t');
        rendered.push_str(&row.reason);
        rendered.push('\n');
    }
    rendered
}

fn repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        render_vcf_local_container_smoke, DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH,
        VCF_LOCAL_CONTAINER_SMOKE_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_local_container_smoke_tracks_retained_bindings() {
        let root = repo_root();
        let report = render_vcf_local_container_smoke(
            &root,
            PathBuf::from(DEFAULT_VCF_LOCAL_CONTAINER_SMOKE_PATH),
        )
        .expect("render VCF local-container smoke");

        assert_eq!(report.schema_version, VCF_LOCAL_CONTAINER_SMOKE_SCHEMA_VERSION);
        assert_eq!(report.output_path, "benchmarks/readiness/vcf/vcf-local-container-smoke.tsv");
        assert_eq!(report.row_count, 44);
        assert_eq!(report.stage_count, 20);
        assert_eq!(report.tool_count, 17);
        assert_eq!(report.host_stage_smoke_row_count, 19);
        assert_eq!(report.container_smoke_row_count, 25);
        assert_eq!(report.runtime_counts.get("host").copied(), Some(19));
        assert_eq!(report.runtime_counts.get("docker-arm64").copied(), Some(23));
        assert_eq!(report.runtime_counts.get("apptainer").copied(), Some(2));

        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.call"
                && row.tool_id == "bcftools"
                && row.smoke_path_kind == "host_stage_smoke"
                && row.smoke_command
                    == "bijux-dna bench local run-vcf-call-smoke --tool-id bcftools"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.impute"
                && row.tool_id == "glimpse"
                && row.smoke_path_kind == "docker_container_smoke"
                && row.smoke_command == "bijux-dna env smoke docker-arm64 glimpse"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.postprocess"
                && row.tool_id == "bcftools"
                && row.smoke_path_kind == "docker_container_smoke"
                && row.smoke_command == "bijux-dna env smoke docker-arm64 bcftools"
        }));
        assert!(report.rows.iter().any(|row| {
            row.stage_id == "vcf.phasing"
                && row.tool_id == "shapeit"
                && row.smoke_path_kind == "apptainer_container_smoke"
                && row.smoke_command == "bijux-dna env smoke apptainer shapeit"
        }));
    }
}
