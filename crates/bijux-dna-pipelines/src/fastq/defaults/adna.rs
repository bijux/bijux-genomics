use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_fastq::params::{DamageMode, PairedMode};

use super::fastq_defaults;
use crate::{DefaultParams, EffectiveDefaults};

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
