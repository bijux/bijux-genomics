//! Owner: bijux-dna-bench
//! Shared immutable authorities for BAM observational benchmark attempts.

mod inputs;
mod parameters;
mod report;

pub(super) use inputs::execution_inputs;
pub(super) use parameters::{ensure_parameter_evidence, BamParameterPolicy};
pub(crate) use report::{
    numeric_value, publish_bam_observation_stage_report, read_numeric_summary,
    render_bam_observation_markdown, validate_byte_counts, BamObservationReportPolicy,
    BamObservationStageReportPublication, ScientificObservation,
};
