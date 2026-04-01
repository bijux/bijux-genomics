//! FASTQ pipeline default construction.

use std::collections::BTreeMap;

use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::params::{DamageMode, PairedMode};

use crate::{DefaultParams, EffectiveDefaults};

mod param_defaults;
mod stage_order;
mod tooling;

use param_defaults::fastq_default_params;
use tooling::fastq_default_tools;

pub(super) use stage_order::{append_stage_once, default_shotgun_required_stages};

pub(super) fn fastq_defaults(paired: bool) -> EffectiveDefaults {
    let tools = fastq_default_tools();
    let params = fastq_default_params(paired);
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

pub(super) fn adna_fastq_defaults() -> EffectiveDefaults {
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

pub(super) fn reference_adna_fastq_defaults() -> EffectiveDefaults {
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
