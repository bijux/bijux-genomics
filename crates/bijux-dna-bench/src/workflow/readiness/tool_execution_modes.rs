//! Owner: benchmark execution-mode observation and validation workflow.
//! Admit scope, acquire current runtime declarations, then produce a complete validation report.
use crate::repo::runtime_probe::load_runtime_probe;
use anyhow::Result;
use bijux_dna_bench_model::{
    ReadinessStageToolBinding, ToolExecutionModeValidationPlan, ToolExecutionModesConfig,
    ToolExecutionModesValidationReport,
};
use std::path::Path;

/// Validate the complete governed binding scope against current runtime declarations.
/// # Errors
/// Rejects scope or declaration mismatches before returning a successful report.
pub fn evaluate_tool_execution_modes(
    repo_root: &Path,
    config_path: &Path,
    config: &ToolExecutionModesConfig,
    bindings: &[ReadinessStageToolBinding],
) -> Result<ToolExecutionModesValidationReport> {
    let config_path = config_path
        .strip_prefix(repo_root)
        .unwrap_or(config_path)
        .to_string_lossy()
        .replace('\\', "/");
    let mut plan = ToolExecutionModeValidationPlan::admit(config, bindings, config_path)?;
    for (tool_id, domain) in plan.required_probes() {
        let probe = load_runtime_probe(repo_root, &domain, &tool_id)?;
        plan.observe(&domain, &tool_id, &probe)?;
    }
    plan.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use bijux_dna_core::id_catalog as stage;

    #[test]
    fn mode_validation_observes_current_sources_and_rejects_scope_before_io() -> Result<()> {
        let directory = bijux_dna_testkit::temp::tempdir_for("execution-mode-validation");
        let root = directory.path();
        let config_path = root.join("config.toml");
        let config: ToolExecutionModesConfig = serde_json::from_value(serde_json::json!({
            "schema_version":bijux_dna_bench_model::LOCAL_TOOL_EXECUTION_MODES_SCHEMA_VERSION,
            "classification_scope":"primary_operator_runtime", "modes":[{
                "execution_mode":"binary", "expected_install_kind":"host_binary",
                "required_runtime_fields":["command_template","install_kind"],
                "summary":"native", "tool_ids":["tool"]}]}))?;
        let bindings = [ReadinessStageToolBinding {
            domain: "fastq".into(),
            stage_id: stage::FASTQ_FILTER.into(),
            tool_id: "tool".into(),
        }];
        let error = evaluate_tool_execution_modes(root, &config_path, &config, &[])
            .err()
            .ok_or_else(|| anyhow!("empty scope must reject"))?;
        assert!(error.to_string().contains("outside the governed benchmark scope"));
        let path = root.join("domain/fastq/tools/tool.yaml");
        std::fs::create_dir_all(path.parent().ok_or_else(|| anyhow!("tool parent"))?)?;
        std::fs::write(
            &path,
            "tool_id: tool\ninstall_kind: host_binary\ncommand_template: [tool]\n",
        )?;
        let report = evaluate_tool_execution_modes(root, &config_path, &config, &bindings)?;
        report.validate()?;
        assert_eq!(report.config_path, "config.toml");
        assert_eq!(report.tool_count, 1);
        std::fs::write(&path, "tool_id: tool\ninstall_kind: external\ncommand_template: [tool]\n")?;
        assert!(evaluate_tool_execution_modes(root, &config_path, &config, &bindings).is_err());
        std::fs::write(
            &path,
            "tool_id: foreign\ninstall_kind: host_binary\ncommand_template: [tool]\n",
        )?;
        assert!(evaluate_tool_execution_modes(root, &config_path, &config, &bindings).is_err());
        std::fs::remove_file(path)?;
        assert!(evaluate_tool_execution_modes(root, &config_path, &config, &bindings).is_err());
        Ok(())
    }
}
