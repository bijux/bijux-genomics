use crate::evidence::serving::VcfToolServingMapReport;
use bijux_dna_bench_vcf_evidence::model::{
    command_reports::VcfRenderedCommandsReport, outputs::VcfAdapterOutputCoverageReport,
};
#[derive(Clone, Copy, Default)]
pub(super) struct ReadinessCounts {
    pub benchmark_ready: usize,
    pub adapter_complete: usize,
    pub output_complete: usize,
    pub rendered_commands: usize,
}
#[derive(Default)]
pub(super) struct ReadinessState {
    pub counts: ReadinessCounts,
    pub tool_serving_map_report: Option<VcfToolServingMapReport>,
    pub adapter_output_coverage_report: Option<VcfAdapterOutputCoverageReport>,
    pub rendered_commands_report: Option<VcfRenderedCommandsReport>,
}
