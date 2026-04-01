//! FASTQ pipeline default construction.

use std::collections::BTreeMap;

use bijux_dna_core::ids::StageId;

use crate::{DefaultParams, EffectiveDefaults};

mod adna;
mod analysis_tools;
mod param_defaults;
mod preprocess_tools;
mod reference_adna;
mod stage_order;
mod tooling;

pub(super) use adna::adna_fastq_defaults;
pub(super) use reference_adna::reference_adna_fastq_defaults;

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
