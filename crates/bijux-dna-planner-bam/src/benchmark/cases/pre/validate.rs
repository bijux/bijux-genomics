//! Governed BAM benchmark pre validate.
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

const LOCAL_VALIDATE_CONFIG_PATH: &str = "configs/bench/local/bam-validate.toml";

const DEFAULT_LOCAL_VALIDATE_OUTPUT_DIR: &str = "artifacts/runs/bench/local-smoke/bam.validate";

#[derive(Debug, Clone)]
pub struct LocalValidateSmokeCasePlan {
    pub sample_id: String,
    pub bam: PathBuf,
    pub alignment_fixture_encoding: LocalValidateAlignmentFixtureEncoding,
    pub bam_index: Option<PathBuf>,
    pub reference_fasta: Option<PathBuf>,
    pub expect_pass: bool,
    pub required_refusal_codes: Vec<String>,
    pub plan: bijux_dna_stage_contract::StagePlanV1,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalValidateAlignmentFixtureEncoding {
    BinaryBam,
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
struct LocalValidateSmokeCase {
    sample_id: String,
    bam: PathBuf,
    alignment_fixture_encoding: LocalValidateAlignmentFixtureEncoding,
    #[serde(default)]
    bam_index: Option<PathBuf>,
    #[serde(default)]
    reference_fasta: Option<PathBuf>,
    #[serde(default = "default_expect_pass")]
    expect_pass: bool,
    #[serde(default)]
    required_refusal_codes: Vec<String>,
}

const fn default_expect_pass() -> bool {
    true
}

/// # Errors
/// Returns an error if the governed local-smoke config is invalid, fixtures are missing, or the
/// governed `bam.validate` plans cannot be built.
pub fn local_validate_smoke_plans(repo_root: &Path) -> Result<Vec<LocalValidateSmokeCasePlan>> {
    let config = load_local_validate_smoke_config(repo_root)?;
    ensure_unique_validate_sample_ids(&config.cases)?;

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

    let out_dir = local_smoke_case_output_dir(
        repo_root,
        output_root,
        &case.sample_id,
        tool_spec.tool_id.as_str(),
    )?;
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
        alignment_fixture_encoding: case.alignment_fixture_encoding,
        bam_index: case.bam_index,
        reference_fasta: case.reference_fasta,
        expect_pass: case.expect_pass,
        required_refusal_codes: case.required_refusal_codes,
        plan,
    })
}

fn ensure_unique_validate_sample_ids(cases: &[LocalValidateSmokeCase]) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::LocalValidateSmokeConfig;
    #[test]
    fn local_validate_smoke_config_rejects_sam_text_proxy_fixtures() {
        let config = r#"
schema_version = "bijux.bench.bam.local_validate.v1"
tool_id = "samtools"

[[cases]]
sample_id = "proxy-refusal"
bam = "assets/toy/core-v1/bam/validation_malformed.bam"
alignment_fixture_encoding = "sam_text_proxy"
expect_pass = false
required_refusal_codes = ["malformed_alignment_record"]
"#;

        let error = toml::from_str::<LocalValidateSmokeConfig>(config)
            .expect_err("sam_text_proxy should not deserialize for bam.validate local smoke");
        assert!(
            error.to_string().contains("unknown variant `sam_text_proxy`"),
            "unexpected deserialize error: {error}"
        );
    }
}
