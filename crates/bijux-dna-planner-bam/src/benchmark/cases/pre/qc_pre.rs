//! Governed BAM benchmark pre qc pre.
use crate::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::benchmark::cases::hydrate_smoke_threads;
use crate::selection::allowed_tools_for_stage;
use crate::selection::load_bam_domain_tool_planning_spec;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_QC_PRE_CONFIG_PATH: &str = "configs/bench/local/bam-qc-pre.toml";

const DEFAULT_LOCAL_QC_PRE_OUTPUT_DIR: &str = "artifacts/runs/bench/local-smoke/bam.qc_pre";

#[derive(Debug, Clone)]
pub struct LocalQcPreSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_total_reads: u64,
    pub expected_mapped_reads: u64,
    pub expected_unmapped_reads: u64,
    pub expected_duplicate_flagged_reads: u64,
    pub expected_contigs: Vec<String>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalQcPreSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    supplemental_tool_ids: Vec<String>,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalQcPreSmokeCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalQcPreSmokeCase {
    sample_id: String,
    bam: PathBuf,
    expected_total_reads: u64,
    expected_mapped_reads: u64,
    expected_unmapped_reads: u64,
    expected_duplicate_flagged_reads: u64,
    expected_contigs: Vec<String>,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.qc_pre` plans cannot be built.
pub fn local_qc_pre_smoke_plans(repo_root: &Path) -> Result<Vec<LocalQcPreSmokeCasePlan>> {
    let config = load_local_qc_pre_smoke_config(repo_root)?;
    ensure_unique_qc_pre_sample_ids(&config.cases)?;

    let stage = BamStage::QcPre;
    let stage_id = StageId::new(stage.as_str().to_string());
    let requested_tool_ids = std::iter::once(config.tool_id.clone())
        .chain(config.supplemental_tool_ids.iter().cloned())
        .collect::<Vec<_>>();
    let mut tool_specs = Vec::new();
    for raw_tool_id in requested_tool_ids {
        let tool_id = ToolId::try_from(raw_tool_id.as_str())
            .map_err(|error| anyhow!("invalid local-smoke tool_id `{raw_tool_id}`: {error}"))?;
        if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
            return Err(anyhow!(
                "local-smoke bam.qc_pre tool `{}` is not admitted by the BAM stage contract",
                tool_id.as_str()
            ));
        }
        if tool_specs.iter().any(|tool_spec: &ToolExecutionSpecV1| tool_spec.tool_id == tool_id) {
            continue;
        }
        let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
        hydrate_smoke_threads(&mut tool_spec, config.threads);
        tool_specs.push(tool_spec);
    }
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_QC_PRE_OUTPUT_DIR));
    let cases = config.cases;
    let mut plans = Vec::new();
    for tool_spec in &tool_specs {
        for case in &cases {
            plans.push(build_local_qc_pre_smoke_case(
                repo_root,
                tool_spec,
                &output_root,
                case.clone(),
            )?);
        }
    }
    Ok(plans)
}

fn build_local_qc_pre_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalQcPreSmokeCase,
) -> Result<LocalQcPreSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.qc_pre BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_contigs.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.qc_pre case `{}` must declare at least one expected contig",
            case.sample_id
        ));
    }
    if case.expected_mapped_reads + case.expected_unmapped_reads != case.expected_total_reads {
        return Err(anyhow!(
            "local-smoke bam.qc_pre case `{}` must satisfy mapped + unmapped == total",
            case.sample_id
        ));
    }

    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let plan = crate::tool_adapters::bam::qc_pre::plan(tool_spec, &case.bam, &out_dir)?;

    Ok(LocalQcPreSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_total_reads: case.expected_total_reads,
        expected_mapped_reads: case.expected_mapped_reads,
        expected_unmapped_reads: case.expected_unmapped_reads,
        expected_duplicate_flagged_reads: case.expected_duplicate_flagged_reads,
        expected_contigs: case.expected_contigs,
        plan,
    })
}

fn ensure_unique_qc_pre_sample_ids(cases: &[LocalQcPreSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.qc_pre sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!("duplicate local-smoke bam.qc_pre sample_id `{}`", case.sample_id));
        }
    }
    Ok(())
}

fn load_local_qc_pre_smoke_config(repo_root: &Path) -> Result<LocalQcPreSmokeConfig> {
    let path = repo_root.join(LOCAL_QC_PRE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalQcPreSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_qc_pre.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.qc_pre schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.qc_pre must declare at least one governed case"));
    }
    Ok(config)
}
