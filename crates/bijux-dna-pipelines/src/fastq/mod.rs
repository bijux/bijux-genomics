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
    overrepresented_profile_defaults, qc_post_defaults, read_length_profile_defaults,
    screen_defaults, stats_defaults, trim_defaults, trim_polyg_tails_defaults,
    trim_terminal_damage_defaults, umi_defaults, validate_defaults,
};
use bijux_dna_domain_fastq::params::{DamageMode, PairedMode};

use crate::{
    ArtifactType, DefaultParams, Domain, EffectiveDefaults, InvariantsPreset, MetricsBundle,
    PipelineCapabilities, PipelineId, PipelineProfile, ReportSection, StabilityTier,
};

pub use invariants::{
    validate_fastq_profile, FastqProfileValidationReport, FastqProfileViolation, FASTQ_INVARIANTS,
};

fn append_stage_once(stages: &mut Vec<String>, stage_id: &str) {
    if !stages.iter().any(|stage| stage == stage_id) {
        stages.push(stage_id.to_string());
    }
}

fn default_shotgun_required_stages() -> Vec<String> {
    bijux_dna_domain_fastq::default_shotgun_preprocess_stage_order()
        .into_iter()
        .map(|stage| match stage.as_str() {
            "fastq.validate_reads" => id_catalog::FASTQ_VALIDATE_PRE.to_string(),
            "fastq.profile_read_lengths" => "fastq.profile_read_lengths".to_string(),
            "fastq.detect_adapters" => id_catalog::FASTQ_DETECT_ADAPTERS.to_string(),
            "fastq.trim_polyg_tails" => "fastq.trim_polyg_tails".to_string(),
            "fastq.trim_terminal_damage" => "fastq.trim_terminal_damage".to_string(),
            "fastq.trim_reads" => id_catalog::FASTQ_TRIM.to_string(),
            "fastq.filter_reads" => id_catalog::FASTQ_FILTER.to_string(),
            "fastq.profile_reads" => id_catalog::FASTQ_STATS_NEUTRAL.to_string(),
            "fastq.profile_overrepresented_sequences" => {
                "fastq.profile_overrepresented_sequences".to_string()
            }
            "fastq.report_qc" => id_catalog::FASTQ_QC_POST.to_string(),
            other => other.to_string(),
        })
        .collect()
}

fn fastq_defaults(paired: bool) -> EffectiveDefaults {
    let tools = BTreeMap::from([
        (
            StageId::from_static("fastq.validate_reads"),
            ToolId::from_static(id_catalog::TOOL_FASTQVALIDATOR_OFFICIAL),
        ),
        (
            StageId::from_static("fastq.profile_reads"),
            ToolId::from_static(id_catalog::TOOL_SEQKIT_STATS),
        ),
        (
            StageId::from_static("fastq.profile_read_lengths"),
            ToolId::from_static(id_catalog::TOOL_SEQKIT_STATS),
        ),
        (
            StageId::from_static("fastq.correct_errors"),
            ToolId::from_static(id_catalog::TOOL_RCORRECTOR),
        ),
        (
            StageId::from_static("fastq.extract_umis"),
            ToolId::from_static(id_catalog::TOOL_UMI_TOOLS),
        ),
        (
            StageId::from_static("fastq.detect_adapters"),
            ToolId::from_static(id_catalog::TOOL_FASTQC),
        ),
        (
            StageId::from_static("fastq.trim_reads"),
            ToolId::from_static(id_catalog::TOOL_FASTP),
        ),
        (
            StageId::from_static("fastq.trim_polyg_tails"),
            ToolId::from_static(id_catalog::TOOL_FASTP),
        ),
        (
            StageId::from_static("fastq.trim_terminal_damage"),
            ToolId::from_static(id_catalog::TOOL_CUTADAPT),
        ),
        (
            StageId::from_static("fastq.filter_reads"),
            ToolId::from_static(id_catalog::TOOL_FASTP),
        ),
        (
            StageId::from_static("fastq.profile_overrepresented_sequences"),
            ToolId::from_static(id_catalog::TOOL_FASTQC),
        ),
        (
            StageId::from_static("fastq.report_qc"),
            ToolId::from_static(id_catalog::TOOL_MULTIQC),
        ),
        (
            StageId::from_static("fastq.merge_pairs"),
            ToolId::from_static(id_catalog::TOOL_PEAR),
        ),
        (
            StageId::from_static("fastq.screen_taxonomy"),
            ToolId::from_static(id_catalog::TOOL_KRAKEN2),
        ),
    ]);
    let mut params = BTreeMap::new();
    params.insert(
        StageId::from_static("fastq.validate_reads"),
        DefaultParams::FastqValidate(validate_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.profile_reads"),
        DefaultParams::FastqStats(stats_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.profile_read_lengths"),
        DefaultParams::FastqReadLengthProfile(read_length_profile_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.correct_errors"),
        DefaultParams::FastqCorrect(correct_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.extract_umis"),
        DefaultParams::FastqUmi(umi_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.detect_adapters"),
        DefaultParams::FastqDetectAdapters(detect_adapters_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.trim_reads"),
        DefaultParams::FastqTrim(trim_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.trim_polyg_tails"),
        DefaultParams::FastqTrimPolygTails(trim_polyg_tails_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.trim_terminal_damage"),
        DefaultParams::FastqTrimTerminalDamage(trim_terminal_damage_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.filter_reads"),
        DefaultParams::FastqFilter(filter_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.profile_overrepresented_sequences"),
        DefaultParams::FastqOverrepresentedProfile(overrepresented_profile_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.report_qc"),
        DefaultParams::FastqQcPost(qc_post_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.merge_pairs"),
        DefaultParams::FastqMerge(merge_defaults(paired)),
    );
    params.insert(
        StageId::from_static("fastq.screen_taxonomy"),
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
        StageId::from_static("fastq.trim_reads"),
        ToolId::from_static(id_catalog::TOOL_ADAPTERREMOVAL),
    );
    defaults.tools.insert(
        StageId::from_static("fastq.merge_pairs"),
        ToolId::from_static(id_catalog::TOOL_LEEHOM),
    );

    if let Some(DefaultParams::FastqTrim(mut params)) = defaults
        .params
        .get(&StageId::from_static("fastq.trim_reads"))
        .cloned()
    {
        params.paired_mode = PairedMode::PairedEnd;
        params.min_len = 25;
        params.q_cutoff = Some(20);
        params.adapter_policy = "ancient_strict".to_string();
        params.damage_mode = Some(DamageMode::Ancient);
        params.polyx_policy = Some("trim".to_string());
        defaults.params.insert(
            StageId::from_static("fastq.trim_reads"),
            DefaultParams::FastqTrim(params),
        );
    }

    if let Some(DefaultParams::FastqFilter(mut params)) = defaults
        .params
        .get(&StageId::from_static("fastq.filter_reads"))
        .cloned()
    {
        params.paired_mode = PairedMode::PairedEnd;
        params.damage_mode = Some(DamageMode::Ancient);
        params.polyx_policy = Some("trim".to_string());
        params.max_n_fraction = Some(0.02);
        defaults.params.insert(
            StageId::from_static("fastq.filter_reads"),
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

    if let Some(DefaultParams::FastqMerge(mut params)) = defaults
        .params
        .get(&StageId::from_static("fastq.merge_pairs"))
        .cloned()
    {
        params.paired_mode = PairedMode::PairedEnd;
        params.min_len = Some(20);
        params.merge_overlap = Some(11);
        defaults.params.insert(
            StageId::from_static("fastq.merge_pairs"),
            DefaultParams::FastqMerge(params),
        );
    }

    if let Some(DefaultParams::FastqTrim(mut params)) = defaults
        .params
        .get(&StageId::from_static("fastq.trim_terminal_damage"))
        .cloned()
    {
        params.paired_mode = PairedMode::PairedEnd;
        params.damage_mode = Some(DamageMode::Ancient);
        defaults.params.insert(
            StageId::from_static("fastq.trim_terminal_damage"),
            DefaultParams::FastqTrim(params),
        );
    }

    defaults.rationales.insert(
        StageId::from_static("fastq.trim_reads"),
        "aDNA preset: short-read preserving trim with strict adapter handling".to_string(),
    );
    defaults.rationales.insert(
        StageId::from_static("fastq.merge_pairs"),
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
    let required_stages = default_shotgun_required_stages();
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
pub fn fastq_default_profile() -> PipelineProfile {
    let required_stages = default_shotgun_required_stages();
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
    let mut required_stages = default_shotgun_required_stages();
    append_stage_once(&mut required_stages, id_catalog::FASTQ_MERGE);
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
pub fn fastq_reference_adna_profile() -> PipelineProfile {
    let defaults = reference_adna_fastq_defaults();
    let mut required_stages = default_shotgun_required_stages();
    append_stage_once(&mut required_stages, id_catalog::FASTQ_LOW_COMPLEXITY);
    append_stage_once(&mut required_stages, id_catalog::FASTQ_MERGE);
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
