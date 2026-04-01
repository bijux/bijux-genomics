use super::adna_fastq_defaults;
use crate::EffectiveDefaults;

mod parameter_overrides;
mod rationale_overrides;
mod tool_overrides;

pub(crate) fn reference_adna_fastq_defaults() -> EffectiveDefaults {
    let mut defaults = adna_fastq_defaults();
    tool_overrides::apply(&mut defaults);
    parameter_overrides::apply(&mut defaults);
    rationale_overrides::apply(&mut defaults);
    defaults
}
