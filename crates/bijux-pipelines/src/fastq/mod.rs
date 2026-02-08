//! FASTQ pipeline profiles and defaults.

use std::collections::BTreeMap;

pub mod invariants;
pub mod profiles;

use bijux_core::ids::StageId;
use bijux_core::prelude::id_catalog;
use bijux_domain_fastq::params::defaults::{
    detect_adapters_defaults, filter_defaults, merge_defaults, preprocess_defaults,
    qc_post_defaults, screen_defaults, trim_defaults, validate_defaults,
};

use crate::{
    ArtifactType, DefaultParams, Domain, EffectiveDefaults, MetricsBundle, PipelineCapabilities,
    PipelineId, PipelineProfile, ReportSection, StabilityTier,
};

fn fastq_defaults(paired: bool) -> EffectiveDefaults {
    let tools = BTreeMap::new();
    let mut params = BTreeMap::new();
    params.insert(
        StageId::from_static("fastq.validate_pre"),
        DefaultParams::FastqValidate(validate_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.stats_neutral"),
        DefaultParams::FastqValidate(validate_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.correct"),
        DefaultParams::FastqValidate(validate_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.umi"),
        DefaultParams::FastqValidate(validate_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.detect_adapters"),
        DefaultParams::FastqDetectAdapters(detect_adapters_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.trim"),
        DefaultParams::FastqTrim(trim_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.filter"),
        DefaultParams::FastqFilter(filter_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.qc_post"),
        DefaultParams::FastqQcPost(qc_post_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.preprocess"),
        DefaultParams::FastqPreprocess(preprocess_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.merge"),
        DefaultParams::FastqMerge(merge_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.screen"),
        DefaultParams::FastqScreen(screen_defaults(paired)),
    );
    let mut rationales = BTreeMap::new();
    for stage_id in params.keys() {
        rationales
            .entry(stage_id.clone())
            .or_insert_with(|| "pipeline default".to_string());
    }
    EffectiveDefaults {
        tools,
        params,
        rationales,
    }
}

#[must_use]
pub fn fastq_minimal_profile() -> PipelineProfile {
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_MINIMAL),
        description: "Minimal FASTQ pipeline",
        stability: StabilityTier::Stable,
        input_domains: vec![Domain::Fastq],
        output_domains: vec![Domain::Fastq],
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
            supports_benchmarks: true,
        },
    }
}

#[must_use]
pub fn fastq_default_profile() -> PipelineProfile {
    let required_stages = vec![
        "fastq.validate_pre",
        "fastq.detect_adapters",
        "fastq.trim",
        "fastq.filter",
        "fastq.stats_neutral",
        "fastq.qc_post",
    ];
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_DEFAULT),
        description: "Default FASTQ pipeline",
        stability: StabilityTier::Stable,
        input_domains: vec![Domain::Fastq],
        output_domains: vec![Domain::Fastq],
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
            supports_benchmarks: true,
        },
    }
}

#[must_use]
pub fn fastq_adna_profile() -> PipelineProfile {
    let mut defaults = fastq_defaults(false);
    if let Some(params) = defaults
        .params
        .get(&StageId::from_static("fastq.trim"))
        .cloned()
    {
        let mut json = params.to_json();
        json["damage_mode"] = serde_json::json!("adna");
        json["min_len"] = serde_json::json!(25);
        defaults.params.insert(
            StageId::from_static("fastq.trim"),
            DefaultParams::Json(json),
        );
    }
    if let Some(params) = defaults
        .params
        .get(&StageId::from_static("fastq.filter"))
        .cloned()
    {
        let mut json = params.to_json();
        json["damage_mode"] = serde_json::json!("adna");
        defaults.params.insert(
            StageId::from_static("fastq.filter"),
            DefaultParams::Json(json),
        );
    }
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_ADNA),
        description: "aDNA-oriented FASTQ pipeline defaults",
        stability: StabilityTier::Beta,
        input_domains: vec![Domain::Fastq],
        output_domains: vec![Domain::Fastq],
        defaults,
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some("adna"),
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
            supports_benchmarks: true,
        },
    }
}

/// # Errors
/// Returns an error if the requested profile id is unknown.
pub fn fastq_profiles_by_id(id: &str) -> anyhow::Result<PipelineProfile> {
    match id {
        id_catalog::PIPELINE_FASTQ_DEFAULT => Ok(fastq_default_profile()),
        id_catalog::PIPELINE_FASTQ_MINIMAL => Ok(fastq_minimal_profile()),
        id_catalog::PIPELINE_FASTQ_ADNA => Ok(fastq_adna_profile()),
        _ => Err(anyhow::anyhow!("unknown FASTQ profile: {id}")),
    }
}
