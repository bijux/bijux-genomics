use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_domain_fastq::params::defaults::filter_defaults;
use bijux_dna_domain_fastq::params::PairedMode;

use crate::{DefaultParams, EffectiveDefaults};

pub(super) fn apply(defaults: &mut EffectiveDefaults) {
    defaults.params.insert(
        StageId::from_static(id_catalog::FASTQ_LOW_COMPLEXITY),
        DefaultParams::FastqFilter(filter_defaults(true)),
    );

    if let Some(DefaultParams::FastqScreen(mut params)) =
        defaults.params.get(&StageId::from_static(id_catalog::FASTQ_SCREEN)).cloned()
    {
        params.paired_mode = PairedMode::PairedEnd;
        params.contaminant_db = Some("host_depletion_db".to_string());
        defaults.params.insert(
            StageId::from_static(id_catalog::FASTQ_SCREEN),
            DefaultParams::FastqScreen(params),
        );
    }
}
