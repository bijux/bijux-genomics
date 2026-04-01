use super::fastq_defaults;
use crate::EffectiveDefaults;

mod parameter_overrides;
mod rationale_overrides;
mod tool_overrides;

pub(crate) fn adna_fastq_defaults() -> EffectiveDefaults {
    let mut defaults = fastq_defaults(true);
    tool_overrides::apply(&mut defaults);
    parameter_overrides::apply(&mut defaults);
    rationale_overrides::apply(&mut defaults);
    defaults
}
