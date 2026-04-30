use bijux_dna_core::ids::{StageId, ToolId};

use crate::{
    benchmark_readiness_for_stage_tool, stage_tool_bindings_for_stage, RuntimeNormalizationLevel,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimComparisonToolProfile {
    pub tool_id: ToolId,
    pub required_lane: bool,
    pub legacy_backend: bool,
    pub benchmark_readiness: Option<crate::BenchmarkReadinessLevel>,
    pub caveats: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimBackendComparisonContract {
    pub stage_id: StageId,
    pub required_tool_ids: Vec<ToolId>,
    pub comparison_tool_profiles: Vec<TrimComparisonToolProfile>,
    pub normalized_metric_ids: Vec<&'static str>,
    pub fairness_rules: Vec<&'static str>,
}

const REQUIRED_TRIM_TOOL_IDS: &[&str] =
    &["cutadapt", "fastp", "adapterremoval", "trimmomatic", "fastx_clipper"];

const NORMALIZED_TRIM_METRIC_IDS: &[&str] = &[
    "reads_in",
    "reads_out",
    "bases_in",
    "bases_out",
    "pairs_in",
    "pairs_out",
    "mean_q_before",
    "mean_q_after",
];

const FAIRNESS_RULES: &[&str] =
    &["same_input_hash", "same_previous_stage_defaults", "same_dataset_class"];

#[must_use]
pub fn trim_backend_comparison_contract() -> TrimBackendComparisonContract {
    let stage_id = StageId::from_static("fastq.trim_reads");
    let available_tools = stage_tool_bindings_for_stage(&stage_id)
        .into_iter()
        .map(|binding| binding.tool_id)
        .collect::<Vec<_>>();
    let required_tool_ids = REQUIRED_TRIM_TOOL_IDS
        .iter()
        .map(|tool_id| ToolId::new((*tool_id).to_string()))
        .collect::<Vec<_>>();
    let comparison_tool_profiles = available_tools
        .into_iter()
        .map(|tool_id| TrimComparisonToolProfile {
            required_lane: REQUIRED_TRIM_TOOL_IDS
                .iter()
                .any(|required| *required == tool_id.as_str()),
            legacy_backend: matches!(tool_id.as_str(), "fastx_clipper" | "alientrimmer"),
            benchmark_readiness: benchmark_readiness_for_stage_tool(
                &stage_id,
                &tool_id,
                RuntimeNormalizationLevel::GenericEnvelope,
            ),
            caveats: trim_backend_caveats(tool_id.as_str()),
            tool_id,
        })
        .collect::<Vec<_>>();

    TrimBackendComparisonContract {
        stage_id,
        required_tool_ids,
        comparison_tool_profiles,
        normalized_metric_ids: NORMALIZED_TRIM_METRIC_IDS.to_vec(),
        fairness_rules: FAIRNESS_RULES.to_vec(),
    }
}

fn trim_backend_caveats(tool_id: &str) -> Vec<&'static str> {
    match tool_id {
        "fastp" => vec![
            "bundles adapter, quality, and polyG heuristics into one backend",
            "native JSON reports expose backend-specific counters not all peers share",
        ],
        "cutadapt" => vec![
            "adapter anchor and linked-adapter semantics differ from quality-first trimmers",
            "paired synchronization depends on explicit cutadapt pair filtering policy",
        ],
        "adapterremoval" => vec![
            "paired overlap handling can change effective trimming outcomes on short inserts",
            "backend is most comparable when merge/collapse side effects stay disabled",
        ],
        "trimmomatic" => vec![
            "sliding-window quality heuristics are legacy and not directly equivalent to fastp",
            "backend-native adapter shorthand must be normalized through the governed adapter bank",
        ],
        "fastx_clipper" => vec![
            "legacy single-end oriented adapter clipping limits paired-end comparability",
            "quality trimming semantics require separate governed caveats when compared with modern tools",
        ],
        "skewer" => vec!["overlap-aware trimming can favor paired-end adapter inference over explicit bank selection"],
        "leehom" => vec!["merge-aware behavior can blur the boundary between trim-only and collapse-capable backends"],
        "alientrimmer" => vec!["legacy adapter matching semantics can diverge on ambiguous IUPAC-rich banks"],
        _ => vec![
            "backend-specific report semantics require governed normalization before cross-tool comparison",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::trim_backend_comparison_contract;

    #[test]
    fn trim_backend_comparison_contract_covers_required_lanes() {
        let contract = trim_backend_comparison_contract();
        for required_tool in ["cutadapt", "fastp", "adapterremoval", "trimmomatic", "fastx_clipper"]
        {
            assert!(
                contract
                    .comparison_tool_profiles
                    .iter()
                    .any(|profile| profile.tool_id.as_str() == required_tool
                        && profile.required_lane),
                "required trim comparison lane missing for {required_tool}",
            );
        }
        assert_eq!(
            contract.normalized_metric_ids,
            vec![
                "reads_in",
                "reads_out",
                "bases_in",
                "bases_out",
                "pairs_in",
                "pairs_out",
                "mean_q_before",
                "mean_q_after",
            ]
        );
    }
}
