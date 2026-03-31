use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{DamageMode, PairedMode};

pub const TRIM_POLYG_TAILS_SCHEMA_VERSION: &str = "bijux.fastq.params.trim_polyg_tails.v1";
mod terminal_damage;
mod tool_profiles;

pub use terminal_damage::{
    default_terminal_damage_execution_policy, parse_terminal_damage_execution_policy,
    resolve_terminal_damage_policy, resolve_terminal_damage_policy_with_override,
    terminal_damage_execution_policy_label, ResolvedTerminalDamagePolicy,
    TerminalDamageExecutionPolicy, TrimTerminalDamageParams, DEFAULT_TERMINAL_DAMAGE_TRIM_3P_BASES,
    DEFAULT_TERMINAL_DAMAGE_TRIM_5P_BASES, TRIM_TERMINAL_DAMAGE_SCHEMA_VERSION,
};
pub use tool_profiles::{
    AlienTrimmerParamsV1, FastxClipperParamsV1, LeeHomTrimParamsV1, OverlapCollapseMode,
    ReadHandlingMode, SkewerTrimParamsV1, TrimAdapterMode, TrimQualityMode, TrimToolParamsV1,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrimEffectiveParams {
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub min_len: u32,
    #[serde(default)]
    pub q_cutoff: Option<u32>,
    pub adapter_policy: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_mode: Option<DamageMode>,
    #[serde(default)]
    pub polyx_policy: Option<String>,
    #[serde(default)]
    pub n_policy: Option<String>,
    #[serde(default)]
    pub contaminant_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrimPolygTailsParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub trim_polyg: bool,
    pub min_polyg_run: u32,
}

impl TrimPolygTailsParams {
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
        if self.min_polyg_run == 0 {
            missing.push("min_polyg_run");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
            "trim_polyg": self.trim_polyg,
            "min_polyg_run": self.min_polyg_run,
        })
    }
}

impl TrimEffectiveParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        if self.adapter_policy.trim().is_empty() {
            missing.push("adapter_policy");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "min_len": self.min_len,
            "q": self.q_cutoff,
            "adapter_policy": self.adapter_policy,
            "damage_mode": self.damage_mode,
            "polyx_policy": self.polyx_policy,
            "n_policy": self.n_policy,
            "contaminant_policy": self.contaminant_policy,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
        })
    }
}
