//! Owner: current execution-mode TOML acquisition and early schema admission.
use anyhow::{Context, Result};
use bijux_dna_bench_model::{ToolExecutionModeAssignment, ToolExecutionModesConfig};
use std::{collections::BTreeMap, fs, path::Path};

pub const DEFAULT_TOOL_EXECUTION_MODES_PATH: &str = "configs/bench/local/tool-execution-modes.toml";

/// Read the current configuration and admit its declared schema and scope.
/// # Errors
/// Returns file, TOML, unknown-field or header errors before runtime observation.
pub fn load_tool_execution_modes_config(config_path: &Path) -> Result<ToolExecutionModesConfig> {
    let raw = fs::read_to_string(config_path)
        .with_context(|| format!("read {}", config_path.display()))?;
    let config: ToolExecutionModesConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", config_path.display()))?;
    config.validate_header()?;
    Ok(config)
}

/// Load and admit the tool assignment map without resolving external executables.
/// # Errors
/// Returns acquisition or declaration admission failures.
pub fn load_tool_execution_mode_assignments(
    config_path: &Path,
) -> Result<BTreeMap<String, ToolExecutionModeAssignment>> {
    load_tool_execution_modes_config(config_path)?.assignments()
}
