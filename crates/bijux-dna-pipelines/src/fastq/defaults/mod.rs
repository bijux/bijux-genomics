//! FASTQ pipeline default construction.

use crate::EffectiveDefaults;

mod adna;
mod analysis_params;
mod analysis_tools;
mod param_defaults;
mod preprocess_params;
mod preprocess_tools;
mod rationales;
mod reference_adna;
mod stage_order;
mod tooling;

pub(super) use adna::adna_fastq_defaults;
pub(super) use reference_adna::reference_adna_fastq_defaults;

use param_defaults::fastq_default_params;
use rationales::fastq_default_rationales;
use tooling::fastq_default_tools;

pub(super) use stage_order::{append_stage_once, default_shotgun_required_stages};

pub(super) fn fastq_defaults(paired: bool) -> EffectiveDefaults {
    let tools = fastq_default_tools();
    let params = fastq_default_params(paired);
    let rationales = fastq_default_rationales(&params);
    EffectiveDefaults {
        tools,
        params,
        rationales,
    }
}
