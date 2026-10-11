//! Owner: execution-mode schema admission, field policy and unique tool assignment.
use crate::model::tool_execution_modes::{
    ToolExecutionModeAssignment, ToolExecutionModeDefinition, ToolExecutionModesConfig,
    LOCAL_TOOL_EXECUTION_MODES_SCHEMA_VERSION, TOOL_EXECUTION_CLASSIFICATION_SCOPE,
};
use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, BTreeSet};

impl ToolExecutionModesConfig {
    /// Admit the version and classification scope before acquiring tool observations.
    /// # Errors
    /// Rejects unknown schema, scope and empty mode declarations.
    pub fn validate_header(&self) -> Result<()> {
        validate_tool_execution_modes_contract(self)
    }
    /// Resolve an unambiguous governed tool assignment map.
    /// # Errors
    /// Rejects invalid declarations, repeated modes/tools and unsupported required fields.
    pub fn assignments(&self) -> Result<BTreeMap<String, ToolExecutionModeAssignment>> {
        self.validate_header()?;
        build_mode_assignment_index(self)
    }
}

fn validate_tool_execution_modes_contract(config: &ToolExecutionModesConfig) -> Result<()> {
    if config.schema_version != LOCAL_TOOL_EXECUTION_MODES_SCHEMA_VERSION {
        return Err(anyhow!("unsupported tool execution mode schema `{}`", config.schema_version));
    }
    if config.classification_scope != TOOL_EXECUTION_CLASSIFICATION_SCOPE {
        return Err(anyhow!(
            "unsupported tool execution classification scope `{}`",
            config.classification_scope
        ));
    }
    if config.modes.is_empty() {
        return Err(anyhow!(
            "tool execution mode config must declare at least one `[[modes]]` entry"
        ));
    }
    Ok(())
}

fn build_mode_assignment_index(
    config: &ToolExecutionModesConfig,
) -> Result<BTreeMap<String, ToolExecutionModeAssignment>> {
    let mut seen_modes = BTreeSet::<String>::new();
    let mut assignments = BTreeMap::<String, ToolExecutionModeAssignment>::new();
    for mode in &config.modes {
        validate_mode_definition(mode)?;
        if !seen_modes.insert(mode.execution_mode.clone()) {
            return Err(anyhow!(
                "tool execution mode config repeats execution_mode `{}`",
                mode.execution_mode
            ));
        }
        for tool_id in &mode.tool_ids {
            if let Some(previous) = assignments.get(tool_id) {
                return Err(anyhow!(
                    "tool `{}` is assigned to both execution mode `{}` and `{}`",
                    tool_id,
                    previous.execution_mode,
                    mode.execution_mode
                ));
            }
            assignments.insert(
                tool_id.clone(),
                ToolExecutionModeAssignment {
                    execution_mode: mode.execution_mode.clone(),
                    expected_install_kind: mode.expected_install_kind.clone(),
                    required_runtime_fields: mode.required_runtime_fields.clone(),
                    summary: mode.summary.clone(),
                },
            );
        }
    }
    Ok(assignments)
}

pub(super) fn validate_mode_definition(mode: &ToolExecutionModeDefinition) -> Result<()> {
    if !matches!(
        mode.execution_mode.as_str(),
        "internal" | "binary" | "containerized" | "external" | "r" | "java" | "python" | "mixed"
    ) {
        return Err(anyhow!(
            "tool execution mode `{}` is not one of the governed allowed values",
            mode.execution_mode
        ));
    }
    if mode.expected_install_kind.trim().is_empty() {
        return Err(anyhow!(
            "execution mode `{}` must declare a non-empty `expected_install_kind`",
            mode.execution_mode
        ));
    }
    if mode.summary.trim().is_empty() {
        return Err(anyhow!(
            "execution mode `{}` must declare a non-empty `summary`",
            mode.execution_mode
        ));
    }
    if mode.required_runtime_fields.is_empty() {
        return Err(anyhow!(
            "execution mode `{}` must declare at least one required runtime field",
            mode.execution_mode
        ));
    }
    let mut sorted_fields = mode.required_runtime_fields.clone();
    sorted_fields.sort();
    if sorted_fields != mode.required_runtime_fields {
        return Err(anyhow!(
            "execution mode `{}` must keep `required_runtime_fields` sorted lexically",
            mode.execution_mode
        ));
    }
    let unique_fields = mode.required_runtime_fields.iter().cloned().collect::<BTreeSet<_>>();
    if unique_fields.len() != mode.required_runtime_fields.len() {
        return Err(anyhow!(
            "execution mode `{}` repeats one or more required runtime fields",
            mode.execution_mode
        ));
    }
    for field in &mode.required_runtime_fields {
        if !matches!(
            field.as_str(),
            "install_kind" | "container.image" | "help_cmd" | "version_cmd" | "command_template"
        ) {
            return Err(anyhow!(
                "execution mode `{}` declares unsupported runtime field `{}`",
                mode.execution_mode,
                field
            ));
        }
    }
    if mode.tool_ids.is_empty() {
        return Err(anyhow!(
            "execution mode `{}` must declare at least one tool id",
            mode.execution_mode
        ));
    }
    let mut sorted_tool_ids = mode.tool_ids.clone();
    sorted_tool_ids.sort();
    if sorted_tool_ids != mode.tool_ids {
        return Err(anyhow!(
            "execution mode `{}` must keep `tool_ids` sorted lexically",
            mode.execution_mode
        ));
    }
    let unique_tool_ids = mode.tool_ids.iter().cloned().collect::<BTreeSet<_>>();
    if unique_tool_ids.len() != mode.tool_ids.len() {
        return Err(anyhow!(
            "execution mode `{}` repeats one or more tool ids",
            mode.execution_mode
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn declarations_admit_unique_assignments_and_reject_ambiguous_or_unknown_contracts(
    ) -> Result<()> {
        let source = json!({"schema_version": LOCAL_TOOL_EXECUTION_MODES_SCHEMA_VERSION,
            "classification_scope": "primary_operator_runtime", "modes": [{
            "execution_mode":"binary", "expected_install_kind":"binary",
            "required_runtime_fields":["command_template", "install_kind"],
            "summary":"native executable", "tool_ids":["alpha", "beta"]}]});
        let decode = |value| serde_json::from_value::<ToolExecutionModesConfig>(value);
        let config = decode(source.clone())?;
        let assignments = config.assignments()?;
        assert_eq!(assignments.len(), 2);
        assert_eq!(assignments["alpha"].execution_mode, "binary");
        assert_eq!(
            assignments["beta"].required_runtime_fields,
            ["command_template", "install_kind"]
        );
        for (pointer, value) in [
            ("/schema_version", json!("unknown")),
            ("/classification_scope", json!("unknown")),
            ("/modes", json!([])),
            ("/modes/0/execution_mode", json!("unknown")),
            ("/modes/0/expected_install_kind", json!(" ")),
            ("/modes/0/summary", json!(" ")),
            ("/modes/0/required_runtime_fields", json!([])),
            ("/modes/0/required_runtime_fields", json!(["unknown"])),
            ("/modes/0/required_runtime_fields", json!(["install_kind", "command_template"])),
            ("/modes/0/required_runtime_fields", json!(["install_kind", "install_kind"])),
            ("/modes/0/tool_ids", json!([])),
            ("/modes/0/tool_ids", json!(["beta", "alpha"])),
            ("/modes/0/tool_ids", json!(["alpha", "alpha"])),
        ] {
            let mut bad = source.clone();
            *bad.pointer_mut(pointer)
                .ok_or_else(|| anyhow!("missing mutation field {pointer}"))? = value;
            assert!(decode(bad)?.assignments().is_err(), "{pointer}");
        }
        let mut duplicate = source.clone();
        duplicate["modes"]
            .as_array_mut()
            .ok_or_else(|| anyhow!("missing modes"))?
            .push(source["modes"][0].clone());
        assert!(decode(duplicate.clone())?.assignments().is_err());
        duplicate["modes"][1]["execution_mode"] = json!("python");
        assert!(decode(duplicate)?.assignments().is_err());
        let mut unknown = source.clone();
        unknown["unexpected"] = json!(true);
        assert!(decode(unknown).is_err());
        let mut unknown_mode = source;
        unknown_mode["modes"][0]["unexpected"] = json!(true);
        assert!(decode(unknown_mode).is_err());
        Ok(())
    }
}
