use super::acquisition::EvidenceSet;
use super::bindings::{
    binding_key_from_command_coverage_row, binding_key_from_expected_result_row,
    binding_key_from_local_smoke_row, binding_key_from_output_declaration_row,
    binding_key_from_parser_fixture_row, binding_key_from_rendered_command_row,
    binding_key_from_report_map_row, binding_key_from_tool_serving_map_row, BamBindingKey,
};
use bijux_dna_bench_bam_evidence::command_coverage::BamBenchmarkStatus;
use std::collections::BTreeSet;
pub(super) struct BindingInventory {
    pub retained_bindings: BTreeSet<BamBindingKey>,
    pub expected_retained_row_count: usize,
    pub expected_retained_stage_count: usize,
    pub expected_retained_tool_count: usize,
    pub command_bindings: BTreeSet<BamBindingKey>,
    pub parser_bindings: BTreeSet<BamBindingKey>,
    pub local_smoke_bindings: BTreeSet<BamBindingKey>,
    pub output_declaration_bindings: BTreeSet<BamBindingKey>,
    pub rendered_command_bindings: BTreeSet<BamBindingKey>,
    pub expected_result_bindings: BTreeSet<BamBindingKey>,
    pub report_map_bindings: BTreeSet<BamBindingKey>,
}
pub(super) fn inventory(evidence: &EvidenceSet) -> BindingInventory {
    let retained_bindings = evidence
        .tool_map
        .rows
        .iter()
        .map(binding_key_from_tool_serving_map_row)
        .collect::<BTreeSet<_>>();
    let expected_retained_row_count = retained_bindings.len();
    let expected_retained_stage_count = retained_bindings
        .iter()
        .map(|binding| binding.stage_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_retained_tool_count = retained_bindings
        .iter()
        .map(|binding| binding.tool_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let command_bindings = evidence
        .command_coverage
        .rows
        .iter()
        .filter(|row| row.benchmark_status == BamBenchmarkStatus::BenchmarkReady)
        .map(binding_key_from_command_coverage_row)
        .collect::<BTreeSet<_>>();
    let parser_bindings = evidence
        .parser_coverage
        .rows
        .iter()
        .map(binding_key_from_parser_fixture_row)
        .collect::<BTreeSet<_>>();
    let local_smoke_bindings = evidence
        .local_smoke
        .rows
        .iter()
        .map(binding_key_from_local_smoke_row)
        .collect::<BTreeSet<_>>();
    let output_declaration_bindings = evidence
        .output_declarations
        .rows
        .iter()
        .map(binding_key_from_output_declaration_row)
        .collect::<BTreeSet<_>>();
    let rendered_command_bindings = evidence
        .rendered_commands
        .rows
        .iter()
        .map(binding_key_from_rendered_command_row)
        .collect::<BTreeSet<_>>();
    let expected_result_bindings = evidence
        .expected_results
        .rows
        .iter()
        .filter(|row| row.domain == "bam")
        .map(binding_key_from_expected_result_row)
        .collect::<BTreeSet<_>>();
    let report_map_bindings = evidence
        .report_map
        .rows
        .iter()
        .map(binding_key_from_report_map_row)
        .collect::<BTreeSet<_>>();

    BindingInventory {
        retained_bindings,
        expected_retained_row_count,
        expected_retained_stage_count,
        expected_retained_tool_count,
        command_bindings,
        parser_bindings,
        local_smoke_bindings,
        output_declaration_bindings,
        rendered_command_bindings,
        expected_result_bindings,
        report_map_bindings,
    }
}
