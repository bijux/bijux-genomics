//! FASTQ pipeline profiles and defaults.

use std::collections::BTreeMap;

pub mod invariants;
pub mod profiles;

use bijux_dna_core::ids::{
    AssayKind, LibraryLayout, LibraryModel, PlatformHint, StageId, ToolId, UdgTreatment,
};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_fastq::params::defaults::{
    correct_defaults, detect_adapters_defaults, filter_defaults, merge_defaults,
    preprocess_defaults, qc_post_defaults, screen_defaults, stats_defaults, trim_defaults,
    umi_defaults, validate_defaults,
};
use bijux_dna_domain_fastq::params::preprocess::LibraryDamageTreatment;
use bijux_dna_domain_fastq::params::{DamageMode, PairedMode};

use crate::{
    ArtifactType, DefaultParams, Domain, EffectiveDefaults, InvariantsPreset, MetricsBundle,
    PipelineCapabilities, PipelineId, PipelineProfile, ReportSection, StabilityTier,
};

pub use invariants::{
    validate_fastq_profile, FastqProfileValidationReport, FastqProfileViolation, FASTQ_INVARIANTS,
};

fn fastq_defaults(paired: bool) -> EffectiveDefaults {
    let tools = BTreeMap::from([
        (
            StageId::from_static("fastq.validate_pre"),
            ToolId::from_static(id_catalog::TOOL_FASTQVALIDATOR_OFFICIAL),
        ),
        (
            StageId::from_static("fastq.stats_neutral"),
            ToolId::from_static(id_catalog::TOOL_SEQKIT_STATS),
        ),
        (
            StageId::from_static("fastq.correct"),
            ToolId::from_static(id_catalog::TOOL_RCORRECTOR),
        ),
        (
            StageId::from_static("fastq.umi"),
            ToolId::from_static(id_catalog::TOOL_UMI_TOOLS),
        ),
        (
            StageId::from_static("fastq.detect_adapters"),
            ToolId::from_static(id_catalog::TOOL_FASTQC),
        ),
        (
            StageId::from_static("fastq.trim"),
            ToolId::from_static(id_catalog::TOOL_FASTP),
        ),
        (
            StageId::from_static("fastq.filter"),
            ToolId::from_static(id_catalog::TOOL_SEQKIT),
        ),
        (
            StageId::from_static("fastq.qc_post"),
            ToolId::from_static(id_catalog::TOOL_MULTIQC),
        ),
        (
            StageId::from_static("fastq.preprocess"),
            ToolId::from_static(id_catalog::TOOL_PLANNER),
        ),
        (
            StageId::from_static("fastq.merge"),
            ToolId::from_static(id_catalog::TOOL_VSEARCH),
        ),
        (
            StageId::from_static("fastq.screen"),
            ToolId::from_static(id_catalog::TOOL_KRAKEN2),
        ),
    ]);
    let mut params = BTreeMap::new();
    params.insert(
        StageId::from_static("fastq.validate_pre"),
        DefaultParams::FastqValidate(validate_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.stats_neutral"),
        DefaultParams::FastqStats(stats_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.correct"),
        DefaultParams::FastqCorrect(correct_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.umi"),
        DefaultParams::FastqUmi(umi_defaults(paired)),
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

fn adna_fastq_defaults() -> EffectiveDefaults {
    let mut defaults = fastq_defaults(true);

    defaults.tools.insert(
        StageId::from_static("fastq.trim"),
        ToolId::from_static(id_catalog::TOOL_ADAPTERREMOVAL),
    );
    defaults.tools.insert(
        StageId::from_static("fastq.merge"),
        ToolId::from_static(id_catalog::TOOL_LEEHOM),
    );

    if let Some(DefaultParams::FastqTrim(mut params)) = defaults
        .params
        .get(&StageId::from_static("fastq.trim"))
        .cloned()
    {
        params.paired_mode = PairedMode::PairedEnd;
        params.min_len = 25;
        params.q_cutoff = Some(20);
        params.adapter_policy = "ancient_strict".to_string();
        params.damage_mode = Some(DamageMode::Ancient);
        params.polyx_policy = Some("trim".to_string());
        defaults.params.insert(
            StageId::from_static("fastq.trim"),
            DefaultParams::FastqTrim(params),
        );
    }

    if let Some(DefaultParams::FastqFilter(mut params)) = defaults
        .params
        .get(&StageId::from_static("fastq.filter"))
        .cloned()
    {
        params.paired_mode = PairedMode::PairedEnd;
        params.damage_mode = Some(DamageMode::Ancient);
        params.polyx_policy = Some("trim".to_string());
        params.max_n_fraction = Some(0.02);
        defaults.params.insert(
            StageId::from_static("fastq.filter"),
            DefaultParams::FastqFilter(params),
        );
    }

    if let Some(DefaultParams::FastqDetectAdapters(mut params)) = defaults
        .params
        .get(&StageId::from_static("fastq.detect_adapters"))
        .cloned()
    {
        params.paired_mode = PairedMode::PairedEnd;
        params.sample_reads = Some(2_000_000);
        defaults.params.insert(
            StageId::from_static("fastq.detect_adapters"),
            DefaultParams::FastqDetectAdapters(params),
        );
    }

    if let Some(DefaultParams::FastqPreprocess(mut params)) = defaults
        .params
        .get(&StageId::from_static("fastq.preprocess"))
        .cloned()
    {
        params.paired_mode = PairedMode::PairedEnd;
        params.library_declared_paired = true;
        params.library_damage_treatment = LibraryDamageTreatment::NoUdg;
        defaults.params.insert(
            StageId::from_static("fastq.preprocess"),
            DefaultParams::FastqPreprocess(params),
        );
    }

    if let Some(DefaultParams::FastqMerge(mut params)) = defaults
        .params
        .get(&StageId::from_static("fastq.merge"))
        .cloned()
    {
        params.paired_mode = PairedMode::PairedEnd;
        params.min_len = Some(20);
        params.merge_overlap = Some(11);
        defaults.params.insert(
            StageId::from_static("fastq.merge"),
            DefaultParams::FastqMerge(params),
        );
    }

    defaults.rationales.insert(
        StageId::from_static("fastq.trim"),
        "aDNA preset: short-read preserving trim with strict adapter handling".to_string(),
    );
    defaults.rationales.insert(
        StageId::from_static("fastq.merge"),
        "aDNA preset: aggressive overlap merge/collapse for fragmented paired-end reads"
            .to_string(),
    );
    defaults.rationales.insert(
        StageId::from_static("fastq.detect_adapters"),
        "aDNA preset: stricter adapter detection depth for short fragments".to_string(),
    );

    defaults
}

fn reference_adna_fastq_defaults() -> EffectiveDefaults {
    let mut defaults = adna_fastq_defaults();

    defaults.tools.insert(
        StageId::from_static(id_catalog::FASTQ_TRIM),
        ToolId::from_static(id_catalog::TOOL_FASTP),
    );
    defaults.tools.insert(
        StageId::from_static(id_catalog::FASTQ_MERGE),
        ToolId::from_static(id_catalog::TOOL_VSEARCH),
    );
    defaults.rationales.insert(
        StageId::from_static(id_catalog::FASTQ_TRIM),
        "reference-grade gate: production-pinned trim tool with aDNA-safe parameters".to_string(),
    );
    defaults.rationales.insert(
        StageId::from_static(id_catalog::FASTQ_MERGE),
        "reference-grade gate: production-pinned merge tool with explicit overlap/min-length defaults"
            .to_string(),
    );

    defaults.tools.insert(
        StageId::from_static(id_catalog::FASTQ_LOW_COMPLEXITY),
        ToolId::from_static(id_catalog::TOOL_BBDUK),
    );
    defaults.params.insert(
        StageId::from_static(id_catalog::FASTQ_LOW_COMPLEXITY),
        DefaultParams::FastqFilter(filter_defaults(true)),
    );
    defaults.rationales.insert(
        StageId::from_static(id_catalog::FASTQ_LOW_COMPLEXITY),
        "reference-grade aDNA: pre-alignment low-complexity/duplication proxy estimate stage"
            .to_string(),
    );

    defaults.tools.insert(
        StageId::from_static(id_catalog::FASTQ_SCREEN),
        ToolId::from_static(id_catalog::TOOL_KRAKEN2),
    );
    if let Some(DefaultParams::FastqScreen(mut params)) = defaults
        .params
        .get(&StageId::from_static(id_catalog::FASTQ_SCREEN))
        .cloned()
    {
        params.paired_mode = PairedMode::PairedEnd;
        params.contaminant_db = Some("host_depletion_db".to_string());
        defaults.params.insert(
            StageId::from_static(id_catalog::FASTQ_SCREEN),
            DefaultParams::FastqScreen(params),
        );
    }
    defaults.rationales.insert(
        StageId::from_static(id_catalog::FASTQ_SCREEN),
        "reference-grade aDNA: contamination/host depletion hook with declared reference DB"
            .to_string(),
    );

    defaults
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
        library_model: LibraryModel {
            layout: LibraryLayout::SingleEnd,
            udg_treatment: UdgTreatment::Unknown,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Unknown,
        },
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
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "stage_summaries.json",
                "invariants_report.json",
            ],
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
        library_model: LibraryModel {
            layout: LibraryLayout::SingleEnd,
            udg_treatment: UdgTreatment::Unknown,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Unknown,
        },
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
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "stage_summaries.json",
                "invariants_report.json",
            ],
            supports_benchmarks: true,
        },
    }
}

#[must_use]
pub fn fastq_adna_profile() -> PipelineProfile {
    let defaults = adna_fastq_defaults();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_ADNA),
        description: "aDNA-oriented FASTQ pipeline defaults",
        stability: StabilityTier::Beta,
        input_domains: vec![Domain::Fastq],
        output_domains: vec![Domain::Fastq],
        defaults,
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some(InvariantsPreset::Adna),
        library_model: LibraryModel {
            layout: LibraryLayout::PairedEnd,
            udg_treatment: UdgTreatment::None,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Shotgun,
        },
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
                "fastq.merge",
                "fastq.stats_neutral",
                "fastq.qc_post",
            ],
            required_metrics: vec!["fastq.metrics"],
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "stage_summaries.json",
                "invariants_report.json",
            ],
            supports_benchmarks: true,
        },
    }
}

#[must_use]
pub fn fastq_reference_adna_profile() -> PipelineProfile {
    let defaults = reference_adna_fastq_defaults();
    PipelineProfile {
        id: PipelineId::from_static(id_catalog::PIPELINE_FASTQ_REFERENCE_ADNA),
        description: "Reference-grade aDNA FASTQ pipeline defaults",
        stability: StabilityTier::Beta,
        input_domains: vec![Domain::Fastq],
        output_domains: vec![Domain::Fastq],
        defaults,
        defaults_ledger_ref: "defaults_ledger.json",
        invariants_preset: Some(InvariantsPreset::ReferenceAdna),
        library_model: LibraryModel {
            layout: LibraryLayout::PairedEnd,
            udg_treatment: UdgTreatment::None,
            platform_hint: PlatformHint::Illumina,
            assay_kind: AssayKind::Shotgun,
        },
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
                id_catalog::FASTQ_VALIDATE_PRE,
                id_catalog::FASTQ_DETECT_ADAPTERS,
                id_catalog::FASTQ_TRIM,
                id_catalog::FASTQ_LOW_COMPLEXITY,
                id_catalog::FASTQ_MERGE,
                id_catalog::FASTQ_FILTER,
                id_catalog::FASTQ_STATS_NEUTRAL,
                id_catalog::FASTQ_QC_POST,
            ],
            required_metrics: vec!["fastq.metrics"],
            required_artifacts: vec![
                "report.json",
                "run_manifest.json",
                "stage_summaries.json",
                "invariants_report.json",
            ],
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
        id_catalog::PIPELINE_FASTQ_REFERENCE_ADNA => Ok(fastq_reference_adna_profile()),
        _ => Err(anyhow::anyhow!("unknown FASTQ profile: {id}")),
    }
}
