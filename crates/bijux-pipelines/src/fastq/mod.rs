//! FASTQ pipeline profiles and defaults.

use std::collections::BTreeMap;

use bijux_core::domain::PipelineSpec;
use bijux_domain_fastq::params::{
    detect_adapters::DetectAdaptersEffectiveParams, filter::FilterEffectiveParams,
    merge::MergeEffectiveParams, preprocess::PreprocessEffectiveParams,
    qc_post::QcPostEffectiveParams, screen::ScreenEffectiveParams, trim::TrimEffectiveParams,
    validate::ValidateEffectiveParams, PairedMode,
};

use crate::{Domain, EffectiveDefaults, PipelineCapabilities, PipelineProfile, StageNode};

#[derive(Debug, Clone)]
pub struct CanonicalPipeline {
    pub required: Vec<String>,
    pub optional: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct DefaultPipelineOptions {
    pub paired: bool,
    pub enable_merge: bool,
    pub enable_correct: bool,
    pub enable_qc_post: bool,
    pub enable_screen: bool,
}

impl Default for DefaultPipelineOptions {
    fn default() -> Self {
        Self {
            paired: false,
            enable_merge: true,
            enable_correct: false,
            enable_qc_post: true,
            enable_screen: false,
        }
    }
}

#[must_use]
pub fn canonical_pipeline() -> CanonicalPipeline {
    CanonicalPipeline {
        required: vec![
            "fastq.validate_pre".to_string(),
            "fastq.detect_adapters".to_string(),
            "fastq.trim".to_string(),
            "fastq.filter".to_string(),
            "fastq.stats_neutral".to_string(),
            "fastq.qc_post".to_string(),
        ],
        optional: vec![
            "fastq.merge".to_string(),
            "fastq.correct".to_string(),
            "fastq.umi".to_string(),
            "fastq.screen".to_string(),
        ],
    }
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
            stages: canonical_pipeline().required,
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
    EffectiveDefaults { tools, params }
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
    let canonical = canonical_pipeline();
    PipelineProfile {
        id: "minimal",
        description: "Minimal FASTQ pipeline",
        domains: vec![Domain::Fastq],
        graph: to_graph(&canonical.required),
        defaults: fastq_defaults(false),
        invariants_preset: None,
        capabilities: PipelineCapabilities {
            required_inputs: vec!["fastq"],
            produces_outputs: vec!["fastq", "fastq.metrics"],
            supports_benchmarking: true,
        },
    }
}

#[must_use]
pub fn fastq_default_profile(options: DefaultPipelineOptions) -> PipelineProfile {
    let canonical = canonical_pipeline();
    let mut stages = canonical.required;
    if options.paired && options.enable_correct {
        stages.push("fastq.correct".to_string());
    }
    if options.paired && options.enable_merge {
        stages.push("fastq.merge".to_string());
    }
    if options.enable_screen && !stages.iter().any(|stage| stage == "fastq.screen") {
        stages.push("fastq.screen".to_string());
    }
    if options.enable_qc_post && !stages.iter().any(|stage| stage == "fastq.qc_post") {
        stages.push("fastq.qc_post".to_string());
    }
    PipelineProfile {
        id: "default",
        description: "Default FASTQ pipeline",
        domains: vec![Domain::Fastq],
        graph: to_graph(&stages),
        defaults: fastq_defaults(options.paired),
        invariants_preset: None,
        capabilities: PipelineCapabilities {
            required_inputs: vec!["fastq"],
            produces_outputs: vec!["fastq", "fastq.metrics"],
            supports_benchmarking: true,
        },
    }
}

#[must_use]
pub fn fastq_default_pipeline_spec(options: DefaultPipelineOptions) -> PipelineSpec {
    let profile = fastq_default_profile(options);
    PipelineSpec {
        stages: profile
            .graph
            .iter()
            .map(|node| node.stage_id.clone())
            .collect(),
    }
}

#[must_use]
pub fn fastq_minimal_pipeline_spec() -> PipelineSpec {
    let profile = fastq_minimal_profile();
    PipelineSpec {
        stages: profile
            .graph
            .iter()
            .map(|node| node.stage_id.clone())
            .collect(),
    }
}

/// # Errors
/// Returns an error if the requested profile id is unknown.
pub fn fastq_profiles_by_id(id: &str) -> anyhow::Result<PipelineProfile> {
    match id {
        "default" => Ok(fastq_default_profile(DefaultPipelineOptions::default())),
        "minimal" => Ok(fastq_minimal_profile()),
        _ => Err(anyhow::anyhow!("unknown FASTQ profile: {id}")),
    }
}
