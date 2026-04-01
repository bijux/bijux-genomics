//! FASTQ pipeline default construction.

use crate::EffectiveDefaults;

mod adna;
mod analysis_params;
mod analysis_tools;
mod parameter_defaults;
mod preprocess_params;
mod preprocess_tools;
mod rationales;
mod reference_adna;
mod stage_order;
mod tool_defaults;

pub(crate) use adna::adna_fastq_defaults;
pub(crate) use reference_adna::reference_adna_fastq_defaults;

use parameter_defaults::fastq_default_params;
use rationales::fastq_default_rationales;
use tool_defaults::fastq_default_tools;

pub(crate) use stage_order::{append_stage_once, default_shotgun_required_stages};

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
