use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_fastq::params::defaults::filter_defaults;
use bijux_dna_domain_fastq::params::PairedMode;

use super::adna_fastq_defaults;
use crate::{DefaultParams, EffectiveDefaults};

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
