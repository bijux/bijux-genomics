use crate::EffectiveDefaults;
use bijux_dna_core::ids::StageId;

use super::parameter_defaults::fastq_default_params;
use super::rationales::fastq_default_rationales;
use super::tool_defaults::fastq_default_tools;

pub(crate) fn fastq_defaults(paired: bool) -> EffectiveDefaults {
    let tools = fastq_default_tools();
    let params = fastq_default_params(paired);
    let rationales = fastq_default_rationales(&params);
    EffectiveDefaults { tools, params, rationales }
}

pub(crate) fn generic_fastq_defaults() -> EffectiveDefaults {
    let mut defaults = fastq_defaults(false);
    let damage_stage = StageId::from_static("fastq.trim_terminal_damage");
    defaults.tools.remove(&damage_stage);
    defaults.params.remove(&damage_stage);
    defaults.rationales.remove(&damage_stage);
    defaults
}
