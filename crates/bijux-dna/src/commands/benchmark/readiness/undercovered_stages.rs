use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use super::catalog::{load_stage_admissions, ReadinessDomain};
use super::tool_serving_map::{
    render_bam_tool_serving_map, render_fastq_tool_serving_map, DEFAULT_BAM_TOOL_SERVING_MAP_PATH,
    DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH,
};
use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_UNDERCOVERED_STAGES_PATH: &str =
    "target/bench-readiness/undercovered-stages.tsv";
const UNDERCOVERED_STAGES_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.undercovered_stages.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct UndercoveredStageRow {
    pub(crate) domain: String,
    pub(crate) stage_id: String,
    pub(crate) valid_tool_count: usize,
    pub(crate) registered_tool_count: usize,
    pub(crate) valid_tool_ids: Vec<String>,
    pub(crate) registered_tool_ids: Vec<String>,
    pub(crate) missing_tool_ids: Vec<String>,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct UndercoveredStagesReport {
    pub(crate) schema_version: &'static str,
    pub(crate) output_path: String,
    pub(crate) stage_count: usize,
    pub(crate) undercovered_stage_count: usize,
    pub(crate) domain_counts: BTreeMap<String, usize>,
    pub(crate) ok: bool,
    pub(crate) rows: Vec<UndercoveredStageRow>,
}

pub(crate) fn run_render_undercovered_stages(
    args: &parse::BenchReadinessRenderUndercoveredStagesArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_undercovered_stages(
        &repo_root,
        args.output
            .clone()
            .unwrap_or_else(|| PathBuf::from(DEFAULT_UNDERCOVERED_STAGES_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.output_path);
    }
    Ok(())
}

pub(crate) fn render_undercovered_stages(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<UndercoveredStagesReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let fastq_map =
        render_fastq_tool_serving_map(repo_root, PathBuf::from(DEFAULT_FASTQ_TOOL_SERVING_MAP_PATH))?;
    let bam_map =
        render_bam_tool_serving_map(repo_root, PathBuf::from(DEFAULT_BAM_TOOL_SERVING_MAP_PATH))?;

    let registered_tool_ids = BTreeMap::from([
        (
            ReadinessDomain::Fastq,
            stage_to_registered_tool_ids(&fastq_map.rows),
        ),
        (ReadinessDomain::Bam, stage_to_registered_tool_ids(&bam_map.rows)),
    ]);

    let mut rows = Vec::new();
    for domain in [ReadinessDomain::Fastq, ReadinessDomain::Bam] {
        let stage_admissions = load_stage_admissions(repo_root, domain)?;
        let registered_by_stage = registered_tool_ids.get(&domain).expect("registered stage map");
        for (stage_id, admissions) in stage_admissions {
            let valid_tool_ids = admissions
                .iter()
                .map(|admission| admission.tool_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let registered_tool_ids = registered_by_stage
                .get(stage_id.as_str())
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();
            if valid_tool_ids.len() <= 1 || registered_tool_ids.len() != 1 {
                continue;
            }
            let missing_tool_ids = valid_tool_ids
                .iter()
                .filter(|tool_id| !registered_tool_ids.contains(tool_id))
                .cloned()
                .collect::<Vec<_>>();
            rows.push(UndercoveredStageRow {
                domain: domain.as_str().to_string(),
                stage_id: stage_id.clone(),
                valid_tool_count: valid_tool_ids.len(),
                registered_tool_count: registered_tool_ids.len(),
                valid_tool_ids: valid_tool_ids.clone(),
                registered_tool_ids: registered_tool_ids.clone(),
                missing_tool_ids: missing_tool_ids.clone(),
                reason: format!(
                    "stage `{stage_id}` admits {} governed tool options ({}) but only registers {}; add {} to avoid a single-backend benchmark slice",
                    valid_tool_ids.len(),
                    valid_tool_ids.join(", "),
                    registered_tool_ids.join(", "),
                    missing_tool_ids.join(", "),
                ),
            });
        }
    }
    rows.sort_by(|left, right| left.domain.cmp(&right.domain).then_with(|| left.stage_id.cmp(&right.stage_id)));

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(&output_path, render_undercovered_stages_tsv(&rows))
        .with_context(|| format!("write {}", output_path.display()))?;

    let mut domain_counts = BTreeMap::<String, usize>::new();
    for row in &rows {
        *domain_counts.entry(row.domain.clone()).or_default() += 1;
    }

    Ok(UndercoveredStagesReport {
        schema_version: UNDERCOVERED_STAGES_SCHEMA_VERSION,
        output_path: path_relative_to_repo(repo_root, &output_path),
        stage_count: fastq_map.stage_count + bam_map.stage_count,
        undercovered_stage_count: rows.len(),
        domain_counts,
        ok: rows.is_empty(),
        rows,
    })
}

fn stage_to_registered_tool_ids(
    rows: &[super::tool_serving_map::ToolServingMapRow],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut registered = BTreeMap::<String, BTreeSet<String>>::new();
    for row in rows {
        registered
            .entry(row.stage_id.clone())
            .or_default()
            .insert(row.tool_id.clone());
    }
    registered
}

fn render_undercovered_stages_tsv(rows: &[UndercoveredStageRow]) -> String {
    let mut rendered = String::from(
        "domain\tstage_id\tvalid_tool_count\tregistered_tool_count\tvalid_tool_ids\tregistered_tool_ids\tmissing_tool_ids\treason\n",
    );
    for row in rows {
        rendered.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            sanitize_tsv(&row.domain),
            sanitize_tsv(&row.stage_id),
            row.valid_tool_count,
            row.registered_tool_count,
            sanitize_tsv(&row.valid_tool_ids.join(",")),
            sanitize_tsv(&row.registered_tool_ids.join(",")),
            sanitize_tsv(&row.missing_tool_ids.join(",")),
            sanitize_tsv(&row.reason),
        ));
    }
    rendered
}

fn sanitize_tsv(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
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
        render_undercovered_stages, DEFAULT_UNDERCOVERED_STAGES_PATH,
        UNDERCOVERED_STAGES_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn undercovered_stages_report_retains_stage_level_findings() {
        let root = repo_root();
        let report = render_undercovered_stages(&root, PathBuf::from(DEFAULT_UNDERCOVERED_STAGES_PATH))
            .expect("render undercovered stages");

        assert_eq!(report.schema_version, UNDERCOVERED_STAGES_SCHEMA_VERSION);
        assert_eq!(report.stage_count, 51);
        assert!(
            report.rows.iter().all(|row| row.valid_tool_count > 1 && row.registered_tool_count == 1),
            "undercovered rows must only contain stage slices with multiple admitted tools but one registered tool"
        );
    }
}
