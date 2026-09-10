//! Owner: bijux-dna-bench-workflows
//! Identity of the producer of a workflow observation.
#[derive(Clone, Copy)]
pub(crate) struct StageIdentity<'a> {
    pub(crate) stage_id: &'a str,
    pub(crate) domain: &'a str,
    pub(crate) tool_id: &'a str,
    pub(crate) execution_mode: &'a str,
}

#[cfg(feature = "genomics")]
pub(crate) struct HandoffBinding {
    pub(crate) source_stage: &'static str,
    pub(crate) source_output: &'static str,
    pub(crate) target_stage: &'static str,
    pub(crate) target_input: &'static str,
}
