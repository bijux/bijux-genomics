use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::{StageId, ToolExecutionSpecV1, ToolId};
use bijux_dna_domain_bam::BamStage;
use serde::Deserialize;

use crate::selection::{allowed_tools_for_stage, load_bam_domain_tool_planning_spec};

const LOCAL_VALIDATE_CONFIG_PATH: &str = "configs/bench/local/bam-validate.toml";
const DEFAULT_LOCAL_VALIDATE_OUTPUT_DIR: &str = "target/local-smoke/bam.validate";
const LOCAL_QC_PRE_CONFIG_PATH: &str = "configs/bench/local/bam-qc-pre.toml";
const DEFAULT_LOCAL_QC_PRE_OUTPUT_DIR: &str = "target/local-smoke/bam.qc_pre";
const LOCAL_MAPPING_SUMMARY_CONFIG_PATH: &str = "configs/bench/local/bam-mapping-summary.toml";
const DEFAULT_LOCAL_MAPPING_SUMMARY_OUTPUT_DIR: &str = "target/local-smoke/bam.mapping_summary";

#[derive(Debug, Clone)]
pub struct LocalValidateSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub bam_index: Option<PathBuf>,
    pub reference_fasta: Option<PathBuf>,
    pub expect_pass: bool,
    pub required_refusal_codes: Vec<String>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

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

#[derive(Debug, Clone)]
pub struct LocalMappingSummarySmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub expected_total_reads: u64,
    pub expected_mapped_reads: u64,
    pub expected_mapping_fraction: f64,
    pub expected_reference_name: String,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Deserialize)]
struct LocalValidateSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalValidateSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalQcPreSmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalQcPreSmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalMappingSummarySmokeConfig {
    schema_version: String,
    tool_id: String,
    #[serde(default)]
    threads: Option<u32>,
    #[serde(default)]
    output_dir: Option<PathBuf>,
    cases: Vec<LocalMappingSummarySmokeCase>,
}

#[derive(Debug, Deserialize)]
struct LocalValidateSmokeCase {
    sample_id: String,
    bam: PathBuf,
    #[serde(default)]
    bam_index: Option<PathBuf>,
    #[serde(default)]
    reference_fasta: Option<PathBuf>,
    #[serde(default = "default_expect_pass")]
    expect_pass: bool,
    #[serde(default)]
    required_refusal_codes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LocalQcPreSmokeCase {
    sample_id: String,
    bam: PathBuf,
    expected_total_reads: u64,
    expected_mapped_reads: u64,
    expected_unmapped_reads: u64,
    expected_duplicate_flagged_reads: u64,
    expected_contigs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LocalMappingSummarySmokeCase {
    sample_id: String,
    bam: PathBuf,
    expected_total_reads: u64,
    expected_mapped_reads: u64,
    expected_mapping_fraction: f64,
    expected_reference_name: String,
}

const fn default_expect_pass() -> bool {
    true
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.validate` plans cannot be built.
pub fn local_validate_smoke_plans(repo_root: &Path) -> Result<Vec<LocalValidateSmokeCasePlan>> {
    let config = load_local_validate_smoke_config(repo_root)?;
    ensure_unique_sample_ids(&config.cases)?;

    let stage = BamStage::Validate;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.validate tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_VALIDATE_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_validate_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.qc_pre` plans cannot be built.
pub fn local_qc_pre_smoke_plans(repo_root: &Path) -> Result<Vec<LocalQcPreSmokeCasePlan>> {
    let config = load_local_qc_pre_smoke_config(repo_root)?;
    ensure_unique_qc_pre_sample_ids(&config.cases)?;

    let stage = BamStage::QcPre;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.qc_pre tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root =
        config.output_dir.unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_QC_PRE_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| build_local_qc_pre_smoke_case(repo_root, &tool_spec, &output_root, case))
        .collect()
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.mapping_summary` plans cannot be built.
pub fn local_mapping_summary_smoke_plans(
    repo_root: &Path,
) -> Result<Vec<LocalMappingSummarySmokeCasePlan>> {
    let config = load_local_mapping_summary_smoke_config(repo_root)?;
    ensure_unique_mapping_summary_sample_ids(&config.cases)?;

    let stage = BamStage::MappingSummary;
    let stage_id = StageId::new(stage.as_str().to_string());
    let tool_id = ToolId::try_from(config.tool_id.as_str())
        .map_err(|error| anyhow!("invalid local-smoke tool_id `{}`: {error}", config.tool_id))?;
    if !allowed_tools_for_stage(stage).iter().any(|candidate| candidate == &tool_id) {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary tool `{}` is not admitted by the BAM stage contract",
            tool_id.as_str()
        ));
    }

    let mut tool_spec = load_bam_domain_tool_planning_spec(repo_root, &stage_id, &tool_id)?;
    hydrate_smoke_threads(&mut tool_spec, config.threads);
    let output_root = config
        .output_dir
        .unwrap_or_else(|| PathBuf::from(DEFAULT_LOCAL_MAPPING_SUMMARY_OUTPUT_DIR));

    config
        .cases
        .into_iter()
        .map(|case| {
            build_local_mapping_summary_smoke_case(repo_root, &tool_spec, &output_root, case)
        })
        .collect()
}

fn build_local_validate_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalValidateSmokeCase,
) -> Result<LocalValidateSmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.validate BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }

    if let Some(bam_index) = case.bam_index.as_ref() {
        let bam_index_abs = repo_root.join(bam_index);
        if !bam_index_abs.is_file() {
            return Err(anyhow!(
                "local-smoke bam.validate BAM index fixture is missing: {}",
                bam_index_abs.display()
            ));
        }
    }

    if let Some(reference_fasta) = case.reference_fasta.as_ref() {
        let reference_abs = repo_root.join(reference_fasta);
        if !reference_abs.is_file() {
            return Err(anyhow!(
                "local-smoke bam.validate reference fixture is missing: {}",
                reference_abs.display()
            ));
        }
    }

    if case.expect_pass && !case.required_refusal_codes.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.validate passing case `{}` must not declare refusal expectations",
            case.sample_id
        ));
    }
    if !case.expect_pass && case.required_refusal_codes.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.validate refusal case `{}` must declare at least one expected refusal code",
            case.sample_id
        ));
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = crate::tool_adapters::bam::validate::plan(
        tool_spec,
        &case.bam,
        case.bam_index.as_deref(),
        case.reference_fasta.as_deref(),
        &out_dir,
    )?;

    Ok(LocalValidateSmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        bam_index: case.bam_index,
        reference_fasta: case.reference_fasta,
        expect_pass: case.expect_pass,
        required_refusal_codes: case.required_refusal_codes,
        plan,
    })
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

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
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

fn build_local_mapping_summary_smoke_case(
    repo_root: &Path,
    tool_spec: &ToolExecutionSpecV1,
    output_root: &Path,
    case: LocalMappingSummarySmokeCase,
) -> Result<LocalMappingSummarySmokeCasePlan> {
    let bam_abs = repo_root.join(&case.bam);
    if !bam_abs.is_file() {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary BAM fixture is missing: {}",
            bam_abs.display()
        ));
    }
    if case.expected_reference_name.trim().is_empty() {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary case `{}` must declare a non-empty expected reference name",
            case.sample_id
        ));
    }
    if case.expected_mapped_reads > case.expected_total_reads {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary case `{}` cannot declare mapped reads greater than total reads",
            case.sample_id
        ));
    }
    if !(0.0..=1.0).contains(&case.expected_mapping_fraction) {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary case `{}` must declare mapping fraction within [0, 1]",
            case.sample_id
        ));
    }
    let derived_fraction = if case.expected_total_reads == 0 {
        0.0
    } else {
        case.expected_mapped_reads as f64 / case.expected_total_reads as f64
    };
    if (derived_fraction - case.expected_mapping_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary case `{}` must keep expected mapping fraction aligned with mapped and total reads",
            case.sample_id
        ));
    }

    let out_dir = output_root.join(&case.sample_id).join(tool_spec.tool_id.as_str());
    let plan = crate::tool_adapters::bam::mapping_summary::plan(tool_spec, &case.bam, &out_dir)?;

    Ok(LocalMappingSummarySmokeCasePlan {
        sample_id: case.sample_id,
        bam: case.bam,
        expected_total_reads: case.expected_total_reads,
        expected_mapped_reads: case.expected_mapped_reads,
        expected_mapping_fraction: case.expected_mapping_fraction,
        expected_reference_name: case.expected_reference_name,
        plan,
    })
}

fn hydrate_smoke_threads(tool_spec: &mut ToolExecutionSpecV1, threads: Option<u32>) {
    if let Some(threads) = threads {
        tool_spec.resources.threads = threads.max(1);
    } else {
        tool_spec.resources.threads = tool_spec.resources.threads.max(1);
    }
}

fn ensure_unique_sample_ids(cases: &[LocalValidateSmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.validate sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.validate sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
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

fn ensure_unique_mapping_summary_sample_ids(cases: &[LocalMappingSummarySmokeCase]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for case in cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!("local-smoke bam.mapping_summary sample_id must not be empty"));
        }
        if !seen.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "duplicate local-smoke bam.mapping_summary sample_id `{}`",
                case.sample_id
            ));
        }
    }
    Ok(())
}

fn load_local_validate_smoke_config(repo_root: &Path) -> Result<LocalValidateSmokeConfig> {
    let path = repo_root.join(LOCAL_VALIDATE_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalValidateSmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_validate.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.validate schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!("local-smoke bam.validate must declare at least one governed case"));
    }
    Ok(config)
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

fn load_local_mapping_summary_smoke_config(
    repo_root: &Path,
) -> Result<LocalMappingSummarySmokeConfig> {
    let path = repo_root.join(LOCAL_MAPPING_SUMMARY_CONFIG_PATH);
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let config: LocalMappingSummarySmokeConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", path.display()))?;
    if config.schema_version != "bijux.bench.bam.local_mapping_summary.v1" {
        return Err(anyhow!(
            "unsupported local-smoke bam.mapping_summary schema_version `{}`",
            config.schema_version
        ));
    }
    if config.cases.is_empty() {
        return Err(anyhow!(
            "local-smoke bam.mapping_summary must declare at least one governed case"
        ));
    }
    Ok(config)
}
