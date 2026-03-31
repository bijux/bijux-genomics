use anyhow::{anyhow, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::{DamageMode, PairedMode};

pub const TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION: &str = "bijux.fastq.params.trim_terminal_damage.v1";
pub const DEFAULT_TERMINAL_DAMAGE_TRIM_5P_BASES: u32 = 2;
pub const DEFAULT_TERMINAL_DAMAGE_TRIM_3P_BASES: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrimTerminalDamageParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub damage_mode: DamageMode,
    #[serde(default = "default_terminal_damage_execution_policy")]
    pub execution_policy: TerminalDamageExecutionPolicy,
    pub trim_5p_bases: u32,
    pub trim_3p_bases: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_trim_5p_bases: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_trim_3p_bases: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TerminalDamageExecutionPolicy {
    ExplicitTerminalTrim,
    PreserveUdgTrimmedEnds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedTerminalDamagePolicy {
    pub execution_policy: TerminalDamageExecutionPolicy,
    pub effective_trim_5p_bases: u32,
    pub effective_trim_3p_bases: u32,
    pub requested_trim_5p_bases: u32,
    pub requested_trim_3p_bases: u32,
}

#[must_use]
pub fn default_terminal_damage_execution_policy() -> TerminalDamageExecutionPolicy {
    TerminalDamageExecutionPolicy::ExplicitTerminalTrim
}

#[must_use]
/// # Panics
/// Panics only if the built-in default terminal-damage contract becomes internally inconsistent.
pub fn resolve_terminal_damage_policy(
    damage_mode: DamageMode,
    trim_5p_bases: u32,
    trim_3p_bases: u32,
) -> ResolvedTerminalDamagePolicy {
    resolve_terminal_damage_policy_with_override(damage_mode, trim_5p_bases, trim_3p_bases, None)
        .unwrap_or_else(|err| panic!("derived terminal damage policy must always be valid: {err}"))
}

/// # Errors
/// Returns an error when an explicit execution policy conflicts with the requested damage-mode trim contract.
pub fn resolve_terminal_damage_policy_with_override(
    damage_mode: DamageMode,
    trim_5p_bases: u32,
    trim_3p_bases: u32,
    execution_policy: Option<TerminalDamageExecutionPolicy>,
) -> Result<ResolvedTerminalDamagePolicy> {
    let preserves_udg_trimmed_ends = damage_mode == DamageMode::UdgTrimmed
        && trim_5p_bases == DEFAULT_TERMINAL_DAMAGE_TRIM_5P_BASES
        && trim_3p_bases == DEFAULT_TERMINAL_DAMAGE_TRIM_3P_BASES;
    let derived = if preserves_udg_trimmed_ends {
        ResolvedTerminalDamagePolicy {
            execution_policy: TerminalDamageExecutionPolicy::PreserveUdgTrimmedEnds,
            effective_trim_5p_bases: 0,
            effective_trim_3p_bases: 0,
            requested_trim_5p_bases: trim_5p_bases,
            requested_trim_3p_bases: trim_3p_bases,
        }
    } else {
        ResolvedTerminalDamagePolicy {
            execution_policy: TerminalDamageExecutionPolicy::ExplicitTerminalTrim,
            effective_trim_5p_bases: trim_5p_bases,
            effective_trim_3p_bases: trim_3p_bases,
            requested_trim_5p_bases: trim_5p_bases,
            requested_trim_3p_bases: trim_3p_bases,
        }
    };
    match execution_policy {
        None => Ok(derived),
        Some(TerminalDamageExecutionPolicy::ExplicitTerminalTrim) => {
            Ok(ResolvedTerminalDamagePolicy {
                execution_policy: TerminalDamageExecutionPolicy::ExplicitTerminalTrim,
                effective_trim_5p_bases: trim_5p_bases,
                effective_trim_3p_bases: trim_3p_bases,
                requested_trim_5p_bases: trim_5p_bases,
                requested_trim_3p_bases: trim_3p_bases,
            })
        }
        Some(TerminalDamageExecutionPolicy::PreserveUdgTrimmedEnds) => {
            if damage_mode != DamageMode::UdgTrimmed {
                return Err(anyhow!(
                    "preserve_udg_trimmed_ends requires damage_mode=udg_trimmed"
                ));
            }
            if trim_5p_bases != DEFAULT_TERMINAL_DAMAGE_TRIM_5P_BASES
                || trim_3p_bases != DEFAULT_TERMINAL_DAMAGE_TRIM_3P_BASES
            {
                return Err(anyhow!(
                    "preserve_udg_trimmed_ends requires the governed default trim window \
(5p={DEFAULT_TERMINAL_DAMAGE_TRIM_5P_BASES}, 3p={DEFAULT_TERMINAL_DAMAGE_TRIM_3P_BASES})"
                ));
            }
            Ok(ResolvedTerminalDamagePolicy {
                execution_policy: TerminalDamageExecutionPolicy::PreserveUdgTrimmedEnds,
                effective_trim_5p_bases: 0,
                effective_trim_3p_bases: 0,
                requested_trim_5p_bases: trim_5p_bases,
                requested_trim_3p_bases: trim_3p_bases,
            })
        }
    }
}

#[must_use]
pub fn parse_terminal_damage_execution_policy(
    value: &str,
) -> Option<Option<TerminalDamageExecutionPolicy>> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "policy_derived" | "auto" => Some(None),
        "explicit_terminal_trim" => Some(Some(TerminalDamageExecutionPolicy::ExplicitTerminalTrim)),
        "preserve_udg_trimmed_ends" => {
            Some(Some(TerminalDamageExecutionPolicy::PreserveUdgTrimmedEnds))
        }
        _ => None,
    }
}

#[must_use]
pub fn terminal_damage_execution_policy_label(
    value: Option<TerminalDamageExecutionPolicy>,
) -> &'static str {
    match value {
        None => "policy_derived",
        Some(TerminalDamageExecutionPolicy::ExplicitTerminalTrim) => "explicit_terminal_trim",
        Some(TerminalDamageExecutionPolicy::PreserveUdgTrimmedEnds) => "preserve_udg_trimmed_ends",
    }
}

impl TrimTerminalDamageParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.schema_version.trim().is_empty() {
            missing.push("schema_version");
        }
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
            "damage_mode": self.damage_mode,
            "execution_policy": self.execution_policy,
            "trim_5p_bases": self.trim_5p_bases,
            "trim_3p_bases": self.trim_3p_bases,
            "requested_trim_5p_bases": self.requested_trim_5p_bases,
            "requested_trim_3p_bases": self.requested_trim_3p_bases,
        })
    }
}
