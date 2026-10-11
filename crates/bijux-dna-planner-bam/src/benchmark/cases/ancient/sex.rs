//! Governed BAM benchmark ancient sex.
use crate::benchmark::cases::admission::float_matches;
use crate::benchmark::cases::admission::local_smoke_case_output_dir;
use crate::benchmark::cases::hydrate_smoke_threads;
use crate::selection::allowed_tools_for_stage;
use crate::selection::load_bam_domain_tool_planning_spec;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_core::prelude::ArtifactId;
use bijux_dna_core::prelude::ArtifactRole;
use bijux_dna_core::prelude::StageId;
use bijux_dna_core::prelude::ToolExecutionSpecV1;
use bijux_dna_core::prelude::ToolId;
use bijux_dna_domain_bam::metrics::SexConfidenceClass;
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

const LOCAL_SEX_CONFIG_PATH: &str = "configs/bench/local/bam-sex.toml";

const DEFAULT_LOCAL_SEX_OUTPUT_DIR: &str = "artifacts/runs/bench/local-smoke/bam.sex";

#[derive(Debug, Clone)]
pub struct LocalSexSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub reference: PathBuf,
    pub chromosome_system: String,
    pub minimum_y_sites: u32,
    pub expected_method: String,
    pub expected_x_coverage: f64,
    pub expected_y_coverage: f64,
    pub expected_autosomal_coverage: f64,
    pub expected_call: SexConfidenceClass,
    pub expected_confidence: f64,
    pub expected_status: String,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalSexSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalSexSmokeCase>,
}

#[derive(Debug, Clone, Deserialize)]
struct LocalSexSmokeCase {
    sample_id: String,
    bam: PathBuf,
    reference: PathBuf,
    chromosome_system: String,
    minimum_y_sites: u32,
    expected_method: String,
    expected_x_coverage: f64,
    expected_y_coverage: f64,
    expected_autosomal_coverage: f64,
    expected_call: SexConfidenceClass,
    expected_confidence: f64,
    expected_status: String,
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.sex` plans cannot be built.
pub fn local_sex_smoke_plans(repo_root: &Path) -> Result<Vec<LocalSexSmokeCasePlan>> {
    let config = load_local_sex_smoke_config(repo_root)?;
    ensure_unique_sex_sample_ids(&config.cases)?;

    let stage = BamStage::Sex;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.sex tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_SEX_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_sex_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or one of
/// the admitted `bam.sex` tool plans cannot be built for output-contract proof coverage.
pub fn local_sex_output_contract_plans(repo_root: &Path) -> Result<Vec<LocalSexSmokeCasePlan>> {
    let config = load_local_sex_smoke_config(repo_root)?;
    ensure_unique_sex_sample_ids(&config.cases)?;

    let stage = BamStage::Sex;
    let stage_id = StageId::new(stage.as_str().to_string());
    let mut tool_ids = allowed_tools_for_stage(stage);
    tool_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_SEX_OUTPUT_DIR));
    let cases = config.cases;

    let mut plans = Vec::new();
    for tool_id in tool_ids {
        let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
        hydrate_smoke_threads(&mut tool_spec, config.threads);
        for case in &cases {
            let bam_abs = repo_root.join(&case.bam);
            let reference_abs = repo_root.join(&case.reference);
            let governed_summary = bijux_dna_domain_bam::summarize_tiny_bam_sex(
                &bam_abs,
                &reference_abs,
                tool_id.as_str(),
                Some(case.chromosome_system.as_str()),
                Some(case.minimum_y_sites),
            )?;
            let mut governed_case = case.clone();
            governed_case.expected_method = tool_id.as_str().to_string();
            governed_case.expected_x_coverage = governed_summary.x_coverage;
            governed_case.expected_y_coverage = governed_summary.y_coverage;
            governed_case.expected_autosomal_coverage = governed_summary.autosomal_coverage;
            governed_case.expected_call = governed_summary.call;
            governed_case.expected_confidence = governed_summary.confidence;
            governed_case.expected_status = governed_summary.status;
            plans.push(build_local_sex_smoke_case(
                repo_root,
                &tool_spec,
                &output_root,
                governed_case,
            )?);
        }
    }
    Ok(plans)
}

fn build_local_sex_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalSexSmokeCase,
) -> Result<LocalSexSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!("local-smoke bam.sex BAM fixture is missing: {}", bam_abs.display()));
    }
    let reference_abs = repo_root.join(&case.reference);
    if !reference_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.sex reference fixture is missing: {}",
            reference_abs.display()
        ));
    }
    if case.chromosome_system.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must declare a non-empty chromosome_system",
            case.sample_id
        ));
    }
    if case.minimum_y_sites == 0 {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must declare minimum_y_sites greater than zero",
            case.sample_id
        ));
    }
    if case.expected_method.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must declare a non-empty expected_method",
            case.sample_id
        ));
    }
    if case.expected_status.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must declare a non-empty expected_status",
            case.sample_id
        ));
    }
    if case.expected_x_coverage < 0.0
        || case.expected_y_coverage < 0.0
        || case.expected_autosomal_coverage < 0.0
    {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must keep expected coverage values non-negative",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_confidence) {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must keep expected_confidence within [0, 1]",
            case.sample_id
        ));
    }
    if case.expected_method != tool_spec.tool_id.as_str() {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must keep expected_method aligned with the governed sex tool",
            case.sample_id
        ));
    }

    let governed_summary = bijux_dna_domain_bam::summarize_tiny_bam_sex(
        &bam_abs,
        &reference_abs,
        case.expected_method.as_str(),
        Some(case.chromosome_system.as_str()),
        Some(case.minimum_y_sites),
    )?;
    if !float_matches(case.expected_x_coverage, governed_summary.x_coverage) {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must keep expected_x_coverage aligned with the governed sex summary",
            case.sample_id
        ));
    }
    if !float_matches(case.expected_y_coverage, governed_summary.y_coverage) {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must keep expected_y_coverage aligned with the governed sex summary",
            case.sample_id
        ));
    }
    if !float_matches(case.expected_autosomal_coverage, governed_summary.autosomal_coverage) {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must keep expected_autosomal_coverage aligned with the governed sex summary",
            case.sample_id
        ));
    }
    if case.expected_call != governed_summary.call {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must keep expected_call aligned with the governed sex summary",
            case.sample_id
        ));
    }
    if !float_matches(case.expected_confidence, governed_summary.confidence) {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must keep expected_confidence aligned with the governed sex summary",
            case.sample_id
        ));
    }
    if case.expected_status != governed_summary.status {
        return Err(anyhow!(
            "local-smoke bam.sex case `{}` must keep expected_status aligned with the governed sex summary",
            case.sample_id
        ));
    }

    let params = bijux_dna_domain_bam::params::SexEffectiveParams {
        expected_sex: None,
        method: case.expected_method.clone(),
        chromosome_system: Some(case.chromosome_system.clone()),
        minimum_y_sites: Some(case.minimum_y_sites),
        refuse_without_context: true,
    };
    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
    let mut plan = crate::tool_adapters::bam::sex::plan(tool_spec, &case.bam, &out_dir, &params)?;
    plan.io.inputs.push(bijux_dna_stage_contract::ArtifactRef::required(
        ArtifactId::from_static("reference"),
        case.reference.clone(),
        ArtifactRole::Reference,
    ));
    let plan_params = plan
        .params
        .as_object_mut()
        .ok_or_else(|| anyhow!("local-smoke bam.sex planner params must serialize as an object"))?;
    plan_params.insert("reference".to_string(), serde_json::json!(case.reference));

    Ok(LocalSexSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        reference: case.reference,
        chromosome_system: case.chromosome_system,
        minimum_y_sites: case.minimum_y_sites,
        expected_method: case.expected_method,
        expected_x_coverage: case.expected_x_coverage,
        expected_y_coverage: case.expected_y_coverage,
        expected_autosomal_coverage: case.expected_autosomal_coverage,
        expected_call: case.expected_call,
        expected_confidence: case.expected_confidence,
        expected_status: case.expected_status,
        plan,
    })
}

fn ensure_unique_sex_sample_ids(cases: &[LocalSexSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.sex sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!("duplicate local-smoke bam.sex sample_id `{}`", case.sample_id));
        }
    }
    Ok(())
}

fn load_local_sex_smoke_config(repo_root: &Path) -> Result<LocalSexSmokeConfig> {
    let path = repo_root.join(LOCAL_SEX_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalSexSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_sex.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.sex schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.sex must declare at least one governed case"));
    }
    Ok(config)
}
