//! FASTQ pipeline profiles and defaults.

use std::collections::BTreeMap;

pub mod profiles;

use bijux_domain_fastq::params::{
    detect_adapters::DetectAdaptersEffectiveParams, filter::FilterEffectiveParams,
    merge::MergeEffectiveParams, preprocess::PreprocessEffectiveParams,
    qc_post::QcPostEffectiveParams, screen::ScreenEffectiveParams, trim::TrimEffectiveParams,
    validate::ValidateEffectiveParams, PairedMode,
};

use crate::{
    ArtifactType, Domain, EffectiveDefaults, MetricsBundle, PipelineCapabilities, PipelineId,
    PipelineProfile, ReportSection, StabilityTier, StageNode,
};

fn required_stage_ids() -> Vec<String> {
    vec![
        "fastq.validate_pre".to_string(),
        "fastq.detect_adapters".to_string(),
        "fastq.trim".to_string(),
        "fastq.filter".to_string(),
        "fastq.stats_neutral".to_string(),
        "fastq.qc_post".to_string(),
    ]
}

#[must_use]
pub fn canonical_tool_defaults() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([
        ("fastq.validate_pre", "fastqvalidator_official"),
        ("fastq.detect_adapters", "fastqc"),
        ("fastq.trim", "fastp"),
        ("fastq.filter", "fastp"),
        ("fastq.stats_neutral", "seqkit_stats"),
        ("fastq.qc_post", "multiqc"),
        ("fastq.merge", "vsearch"),
        ("fastq.correct", "rcorrector"),
        ("fastq.umi", "umi_tools"),
        ("fastq.screen", "kraken2"),
    ])
}

fn fastq_defaults(paired: bool) -> EffectiveDefaults {
    let paired_mode = if paired {
        PairedMode::PairedEnd
    } else {
        PairedMode::SingleEnd
    };
    let mut tools = BTreeMap::new();
    let mut params = BTreeMap::new();
    for (stage, tool) in canonical_tool_defaults() {
        tools.insert(stage.to_string(), tool.to_string());
    }
    tools.insert("fastq.preprocess".to_string(), "planner".to_string());
    params.insert(
        "fastq.validate_pre".to_string(),
        serde_json::to_value(ValidateEffectiveParams {
            paired_mode,
            threads: 1,
            q_cutoff: None,
        })
        .unwrap_or(serde_json::Value::Null),
    );
    params.insert(
        "fastq.stats_neutral".to_string(),
        serde_json::to_value(ValidateEffectiveParams {
            paired_mode,
            threads: 1,
            q_cutoff: None,
        })
        .unwrap_or(serde_json::Value::Null),
    );
    params.insert(
        "fastq.correct".to_string(),
        serde_json::to_value(ValidateEffectiveParams {
            paired_mode,
            threads: 1,
            q_cutoff: None,
        })
        .unwrap_or(serde_json::Value::Null),
    );
    params.insert(
        "fastq.umi".to_string(),
        serde_json::to_value(ValidateEffectiveParams {
            paired_mode,
            threads: 1,
            q_cutoff: None,
        })
        .unwrap_or(serde_json::Value::Null),
    );
    params.insert(
        "fastq.detect_adapters".to_string(),
        serde_json::to_value(DetectAdaptersEffectiveParams {
            paired_mode,
            threads: 1,
            sample_reads: None,
        })
        .unwrap_or(serde_json::Value::Null),
    );
    params.insert(
        "fastq.trim".to_string(),
        serde_json::to_value(TrimEffectiveParams {
            paired_mode,
            threads: 1,
            min_len: 0,
            q_cutoff: None,
            adapter_policy: "none".to_string(),
            polyx_policy: None,
            n_policy: None,
            contaminant_policy: None,
        })
        .unwrap_or(serde_json::Value::Null),
    );
    params.insert(
        "fastq.filter".to_string(),
        serde_json::to_value(FilterEffectiveParams {
            paired_mode,
            threads: 1,
            max_n: None,
            max_n_fraction: None,
            max_n_count: None,
            low_complexity_threshold: None,
            entropy_threshold: None,
            contaminant_db: None,
            n_policy: None,
            polyx_policy: None,
        })
        .unwrap_or(serde_json::Value::Null),
    );
    params.insert(
        "fastq.qc_post".to_string(),
        serde_json::to_value(QcPostEffectiveParams {
            paired_mode,
            threads: 1,
        })
        .unwrap_or(serde_json::Value::Null),
    );
    params.insert(
        "fastq.preprocess".to_string(),
        serde_json::to_value(PreprocessEffectiveParams {
            paired_mode,
            threads: 1,
            stages: required_stage_ids(),
            enable_contaminant_removal: false,
        })
        .unwrap_or(serde_json::Value::Null),
    );
    params.insert(
        "fastq.merge".to_string(),
        serde_json::to_value(MergeEffectiveParams {
            paired_mode,
            threads: 1,
            merge_overlap: None,
            min_len: None,
        })
        .unwrap_or(serde_json::Value::Null),
    );
    params.insert(
        "fastq.screen".to_string(),
        serde_json::to_value(ScreenEffectiveParams {
            paired_mode,
            threads: 1,
            contaminant_db: None,
        })
        .unwrap_or(serde_json::Value::Null),
    );
    let mut rationales = BTreeMap::new();
    for stage_id in tools.keys() {
        rationales
            .entry(stage_id.to_string())
            .or_insert_with(|| "pipeline default".to_string());
    }
    EffectiveDefaults {
        tools,
        params,
        rationales,
    }
}

fn to_graph(stages: &[String]) -> Vec<StageNode> {
    stages
        .iter()
        .map(|stage| StageNode {
            stage_id: stage.clone(),
        })
        .collect()
}

#[must_use]
pub fn fastq_minimal_profile() -> PipelineProfile {
    PipelineProfile {
        id: PipelineId::new("fastq-to-fastq__minimal__v1"),
        description: "Minimal FASTQ pipeline",
        stability: StabilityTier::Stable,
        input_domains: vec![Domain::Fastq],
        output_domains: vec![Domain::Fastq],
        graph: to_graph(&required_stage_ids()),
        defaults: fastq_defaults(false),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Fastq],
            output_domains: vec![Domain::Fastq],
            input_artifacts: vec![ArtifactType::FastqReads],
            output_artifacts: vec![ArtifactType::FastqReads, ArtifactType::MetricsBundle],
            required_inputs: vec!["fastq"],
            produces_outputs: vec!["fastq", "fastq.metrics"],
            report_sections: vec!["fastq"],
            required_report_sections: vec![ReportSection::Fastq, ReportSection::PipelineDefaults],
            required_metrics_bundles: vec![MetricsBundle::FastqCore],
            required_stages: vec![
                "fastq.validate_pre",
                "fastq.detect_adapters",
                "fastq.trim",
                "fastq.filter",
                "fastq.stats_neutral",
                "fastq.qc_post",
            ],
            required_metrics: vec!["fastq.metrics"],
            required_artifacts: vec!["report.json", "run_manifest.json", "stage_summaries.json"],
            supports_benchmarking: true,
        },
    }
}

#[must_use]
pub fn fastq_default_profile() -> PipelineProfile {
    let stages = required_stage_ids();
    let required_stages = vec![
        "fastq.validate_pre",
        "fastq.detect_adapters",
        "fastq.trim",
        "fastq.filter",
        "fastq.stats_neutral",
        "fastq.qc_post",
    ];
    PipelineProfile {
        id: PipelineId::new("fastq-to-fastq__default__v1"),
        description: "Default FASTQ pipeline",
        stability: StabilityTier::Stable,
        input_domains: vec![Domain::Fastq],
        output_domains: vec![Domain::Fastq],
        graph: to_graph(&stages),
        defaults: fastq_defaults(false),
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: None,
        capabilities: PipelineCapabilities {
            input_domains: vec![Domain::Fastq],
            output_domains: vec![Domain::Fastq],
            input_artifacts: vec![ArtifactType::FastqReads],
            output_artifacts: vec![ArtifactType::FastqReads, ArtifactType::MetricsBundle],
            required_inputs: vec!["fastq"],
            produces_outputs: vec!["fastq", "fastq.metrics"],
            report_sections: vec!["fastq"],
            required_report_sections: vec![ReportSection::Fastq, ReportSection::PipelineDefaults],
            required_metrics_bundles: vec![MetricsBundle::FastqCore],
            required_stages,
            required_metrics: vec!["fastq.metrics"],
            required_artifacts: vec!["report.json", "run_manifest.json", "stage_summaries.json"],
            supports_benchmarking: true,
        },
    }
}

#[must_use]
pub fn fastq_default_pipeline_stage_ids() -> Vec<String> {
    required_stage_ids()
}

#[must_use]
pub fn fastq_minimal_pipeline_stage_ids() -> Vec<String> {
    required_stage_ids()
}

/// # Errors
/// Returns an error if the requested profile id is unknown.
pub fn fastq_profiles_by_id(id: &str) -> anyhow::Result<PipelineProfile> {
    match id {
        "fastq-to-fastq__default__v1" => Ok(fastq_default_profile()),
        "fastq-to-fastq__minimal__v1" => Ok(fastq_minimal_profile()),
        _ => Err(anyhow::anyhow!("unknown FASTQ profile: {id}")),
    }
}
