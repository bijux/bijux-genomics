use crate::EffectiveDefaults;

use super::parameter_defaults::fastq_default_params;
use super::rationales::fastq_default_rationales;
use super::tool_defaults::fastq_default_tools;

pub(crate) fn fastq_defaults(paired: bool) -> EffectiveDefaults {
    let tools = fastq_default_tools();
    let params = fastq_default_params(paired);
    let rationales = fastq_default_rationales(&params);
    EffectiveDefaults { tools, params, rationales }
}
