use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::params::trim::TerminalDamageExecutionPolicy;
use crate::params::{DamageMode, PairedMode};

pub const TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION: &str =
    "bijux.fastq.trim_terminal_damage.report.v2";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TerminalDamageReportV1 {
    pub schema_version: String,
    pub stage: String,
    pub stage_id: String,
    pub tool_id: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    pub damage_mode: DamageMode,
    pub execution_policy: TerminalDamageExecutionPolicy,
    pub trim_5p_bases: u32,
    pub trim_3p_bases: u32,
    pub requested_trim_5p_bases: Option<u32>,
    pub requested_trim_3p_bases: Option<u32>,
    pub udg_classification: String,
    pub input_r1: String,
    pub input_r2: Option<String>,
    pub output_r1: String,
    pub output_r2: Option<String>,
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub bases_in: Option<u64>,
    pub bases_out: Option<u64>,
    pub mean_q_before: Option<f64>,
    pub mean_q_after: Option<f64>,
    pub ct_ga_asymmetry_pre: Option<f64>,
    pub ct_ga_asymmetry_post: Option<f64>,
    pub ct_ga_asymmetry_pre_r1: Option<f64>,
    pub ct_ga_asymmetry_post_r1: Option<f64>,
    pub ct_ga_asymmetry_pre_r2: Option<f64>,
    pub ct_ga_asymmetry_post_r2: Option<f64>,
    pub terminal_base_composition_pre_r1: Option<BTreeMap<String, u64>>,
    pub terminal_base_composition_post_r1: Option<BTreeMap<String, u64>>,
    pub terminal_base_composition_pre_r2: Option<BTreeMap<String, u64>>,
    pub terminal_base_composition_post_r2: Option<BTreeMap<String, u64>>,
    pub raw_backend_report: Option<String>,
    pub raw_backend_report_format: Option<String>,
    pub runtime_s: Option<f64>,
    pub memory_mb: Option<f64>,
    pub used_fallback: bool,
    pub backend_metrics: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::{TerminalDamageReportV1, TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION};
    use crate::params::trim::TerminalDamageExecutionPolicy;
    use crate::params::{DamageMode, PairedMode};

    #[test]
    fn terminal_damage_report_contract_round_trips() {
        let report = TerminalDamageReportV1 {
            schema_version: TERMINAL_DAMAGE_REPORT_SCHEMA_VERSION.to_string(),
            stage: "fastq.trim_terminal_damage".to_string(),
            stage_id: "fastq.trim_terminal_damage".to_string(),
            tool_id: "cutadapt".to_string(),
            paired_mode: PairedMode::PairedEnd,
            threads: 4,
            damage_mode: DamageMode::Ancient,
            execution_policy: TerminalDamageExecutionPolicy::ExplicitTerminalTrim,
            trim_5p_bases: 2,
            trim_3p_bases: 1,
            requested_trim_5p_bases: Some(2),
            requested_trim_3p_bases: Some(1),
            udg_classification: "non_udg".to_string(),
            input_r1: "reads_R1.fastq.gz".to_string(),
            input_r2: Some("reads_R2.fastq.gz".to_string()),
            output_r1: "trimmed_R1.fastq.gz".to_string(),
            output_r2: Some("trimmed_R2.fastq.gz".to_string()),
            reads_in: Some(200),
            reads_out: Some(198),
            bases_in: Some(20_000),
            bases_out: Some(19_100),
            mean_q_before: Some(28.0),
            mean_q_after: Some(28.5),
            ct_ga_asymmetry_pre: Some(0.45),
            ct_ga_asymmetry_post: Some(0.12),
            ct_ga_asymmetry_pre_r1: Some(0.50),
            ct_ga_asymmetry_post_r1: Some(0.15),
            ct_ga_asymmetry_pre_r2: Some(0.40),
            ct_ga_asymmetry_post_r2: Some(0.09),
            terminal_base_composition_pre_r1: Some(
                [("C".to_string(), 80_u64)].into_iter().collect(),
            ),
            terminal_base_composition_post_r1: Some(
                [("C".to_string(), 30_u64)].into_iter().collect(),
            ),
            terminal_base_composition_pre_r2: Some(
                [("G".to_string(), 75_u64)].into_iter().collect(),
            ),
            terminal_base_composition_post_r2: Some(
                [("G".to_string(), 28_u64)].into_iter().collect(),
            ),
            raw_backend_report: Some("cutadapt.damage.json".to_string()),
            raw_backend_report_format: Some("cutadapt_json".to_string()),
            runtime_s: Some(12.4),
            memory_mb: Some(256.0),
            used_fallback: false,
            backend_metrics: Some(serde_json::json!({"reads_profiled": 200})),
        };

        let encoded =
            serde_json::to_string(&report).unwrap_or_else(|err| panic!("serialize failed: {err}"));
        let decoded: TerminalDamageReportV1 = serde_json::from_str(&encoded)
            .unwrap_or_else(|err| panic!("deserialize failed: {err}"));
        assert_eq!(decoded.tool_id, "cutadapt");
        assert_eq!(decoded.paired_mode, PairedMode::PairedEnd);
        assert_eq!(decoded.threads, 4);
        assert_eq!(
            decoded.raw_backend_report_format.as_deref(),
            Some("cutadapt_json")
        );
        assert!(!decoded.used_fallback);
    }
}
