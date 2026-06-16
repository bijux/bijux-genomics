//! Metric semantics and contextual metadata.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricDirection {
    HigherBetter,
    LowerBetter,
}

#[derive(Debug, Clone, Copy)]
pub struct MetricSemantics {
    pub metric_id: &'static str,
    pub direction: MetricDirection,
    pub units: &'static str,
    pub range: &'static str,
    pub missing_data_policy: &'static str,
    pub influencing_params: &'static [&'static str],
}

const COMPARE_METRIC_SEMANTICS: &[MetricSemantics] = &[
    MetricSemantics {
        metric_id: "runtime_s",
        direction: MetricDirection::LowerBetter,
        units: "seconds",
        range: ">= 0",
        missing_data_policy: "treat_as_infinite",
        influencing_params: &[],
    },
    MetricSemantics {
        metric_id: "memory_mb",
        direction: MetricDirection::LowerBetter,
        units: "MB",
        range: ">= 0",
        missing_data_policy: "treat_as_infinite",
        influencing_params: &[],
    },
    MetricSemantics {
        metric_id: "read_retention",
        direction: MetricDirection::HigherBetter,
        units: "ratio",
        range: "[0, 1]",
        missing_data_policy: "treat_as_0.0",
        influencing_params: &["adapter_bank", "trim_settings", "filter_settings"],
    },
    MetricSemantics {
        metric_id: "base_retention",
        direction: MetricDirection::HigherBetter,
        units: "ratio",
        range: "[0, 1]",
        missing_data_policy: "treat_as_0.0",
        influencing_params: &["adapter_bank", "trim_settings", "filter_settings"],
    },
    MetricSemantics {
        metric_id: "merge_rate",
        direction: MetricDirection::HigherBetter,
        units: "ratio",
        range: "[0, 1]",
        missing_data_policy: "treat_as_0.0",
        influencing_params: &["merge_policy"],
    },
    MetricSemantics {
        metric_id: "error_reduction_proxy",
        direction: MetricDirection::HigherBetter,
        units: "mean_q_delta",
        range: "[0, 45]",
        missing_data_policy: "treat_as_0.0",
        influencing_params: &["corrector_settings"],
    },
];

#[must_use]
pub fn metric_semantics(metric_id: &str) -> Option<&'static MetricSemantics> {
    COMPARE_METRIC_SEMANTICS.iter().find(|spec| spec.metric_id == metric_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BankRefV1 {
    pub bank_id: String,
    pub bank_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricContextV1 {
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub runner: String,
    pub platform: String,
    pub input_hash: String,
    pub params_hash: String,
    #[serde(default)]
    pub presets: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub banks: std::collections::BTreeMap<String, BankRefV1>,
}
