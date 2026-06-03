use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::prelude::{StageId, ToolId};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BamDomainToolStageOutputContract {
    pub tool_id: ToolId,
    pub stage_id: StageId,
    pub declared_output_ids: Vec<String>,
    pub execution_expected_output_ids: Vec<String>,
    pub stage_expected_artifact_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DomainToolYaml {
    tool_id: String,
    #[serde(default)]
    outputs: Vec<DomainToolOutput>,
    #[serde(default)]
    execution_contract: Option<DomainToolExecutionContract>,
    #[serde(default)]
    stage_contracts: BTreeMap<String, DomainToolStageContract>,
    #[serde(default)]
    expected_artifacts: Vec<String>,
    #[serde(default)]
    stage_id: Option<String>,
    #[serde(default)]
    stage_ids: Vec<String>,
    #[serde(default)]
    planned_stage_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DomainToolOutput {
    name: String,
}

#[derive(Debug, Deserialize)]
struct DomainToolExecutionContract {
    #[serde(default)]
    expected_outputs: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct DomainToolStageContract {
    #[serde(default)]
    expected_artifacts: Vec<String>,
}

/// # Errors
/// Returns an error if the governed BAM tool YAML cannot be read, the tool does not admit the
/// requested stage, or a multi-stage tool omits stage-specific expected artifact declarations.
pub fn load_bam_domain_tool_stage_output_contract(
    repo_root: &Path,
    stage_id: &StageId,
    tool_id: &ToolId,
) -> Result<BamDomainToolStageOutputContract> {
    let parsed = load_domain_tool_yaml(repo_root, tool_id)?;
    let yaml_path =
        repo_root.join("domain").join("bam").join("tools").join(format!("{tool_id}.yaml"));
    let admitted_stage_ids = admitted_stage_ids(&parsed);
    if !admitted_stage_ids.iter().any(|candidate| candidate == stage_id.as_str()) {
        return Err(anyhow!(
            "governed tool yaml {} does not admit stage {}",
            yaml_path.display(),
            stage_id.as_str()
        ));
    }

    let stage_expected_artifact_ids =
        if let Some(contract) = parsed.stage_contracts.get(stage_id.as_str()) {
            contract.expected_artifacts.clone()
        } else if admitted_stage_ids.len() == 1 && !parsed.expected_artifacts.is_empty() {
            parsed.expected_artifacts.clone()
        } else {
            return Err(anyhow!(
                "governed tool yaml {} is missing a stage_contract for {}",
                yaml_path.display(),
                stage_id.as_str()
            ));
        };

    Ok(BamDomainToolStageOutputContract {
        tool_id: tool_id.clone(),
        stage_id: stage_id.clone(),
        declared_output_ids: parsed.outputs.into_iter().map(|output| output.name).collect(),
        execution_expected_output_ids: parsed
            .execution_contract
            .map(|contract| contract.expected_outputs)
            .unwrap_or_default(),
        stage_expected_artifact_ids,
    })
}

fn admitted_stage_ids(parsed: &DomainToolYaml) -> Vec<String> {
    let mut admitted_stage_ids = parsed.stage_ids.clone();
    if let Some(single_stage_id) = parsed.stage_id.as_ref() {
        admitted_stage_ids.push(single_stage_id.clone());
    }
    admitted_stage_ids.extend(parsed.planned_stage_ids.iter().cloned());
    admitted_stage_ids
}

fn load_domain_tool_yaml(repo_root: &Path, tool_id: &ToolId) -> Result<DomainToolYaml> {
    let yaml_path =
        repo_root.join("domain").join("bam").join("tools").join(format!("{tool_id}.yaml"));
    let raw = std::fs::read_to_string(&yaml_path)
        .with_context(|| format!("read governed tool yaml {}", yaml_path.display()))?;
    let parsed: DomainToolYaml = bijux_dna_infra::formats::parse_yaml(&raw)
        .with_context(|| format!("parse governed tool yaml {}", yaml_path.display()))?;

    if parsed.tool_id != tool_id.as_str() {
        return Err(anyhow!(
            "governed tool yaml {} declares tool_id {}, expected {}",
            yaml_path.display(),
            parsed.tool_id,
            tool_id.as_str()
        ));
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::load_bam_domain_tool_stage_output_contract;
    use anyhow::Result;
    use bijux_dna_core::prelude::{StageId, ToolId};
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap_or_else(|| panic!("workspace root"))
            .to_path_buf()
    }

    #[test]
    fn load_bam_domain_tool_stage_output_contract_reads_validate_outputs() -> Result<()> {
        let repo_root = repo_root();
        let stage_id = StageId::new("bam.validate".to_string());
        let tool_id = ToolId::new("samtools");

        let contract = load_bam_domain_tool_stage_output_contract(&repo_root, &stage_id, &tool_id)?;

        assert_eq!(contract.tool_id.as_str(), "samtools");
        assert_eq!(contract.stage_id.as_str(), "bam.validate");
        assert_eq!(
            contract.stage_expected_artifact_ids,
            vec![
                "validation_report".to_string(),
                "flagstat".to_string(),
                "stage_metrics".to_string(),
            ]
        );
        Ok(())
    }

    #[test]
    fn load_bam_domain_tool_stage_output_contract_falls_back_for_single_stage_tools() -> Result<()>
    {
        let repo_root = repo_root();
        let stage_id = StageId::new("bam.contamination".to_string());
        let tool_id = ToolId::new("verifybamid2");

        let contract = load_bam_domain_tool_stage_output_contract(&repo_root, &stage_id, &tool_id)?;

        assert_eq!(contract.tool_id.as_str(), "verifybamid2");
        assert_eq!(contract.stage_id.as_str(), "bam.contamination");
        assert_eq!(
            contract.stage_expected_artifact_ids,
            vec!["contamination_estimate".to_string()]
        );
        Ok(())
    }
}
