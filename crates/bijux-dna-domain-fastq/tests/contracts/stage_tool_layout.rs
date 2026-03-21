use bijux_dna_core::ids::{StageId, ToolId};

#[test]
fn domain_layout_filter_excludes_paired_only_dedup_backend_for_single_end_inputs() {
    let filtered = bijux_dna_domain_fastq::filter_tools_for_input_layout(
        &StageId::from_static("fastq.remove_duplicates"),
        vec![
            ToolId::from_static("fastuniq"),
            ToolId::from_static("clumpify"),
        ],
        false,
    );
    assert_eq!(filtered, vec![ToolId::from_static("clumpify")]);
}

#[test]
fn domain_layout_filter_keeps_paired_dedup_backends_for_paired_inputs() {
    let filtered = bijux_dna_domain_fastq::filter_tools_for_input_layout(
        &StageId::from_static("fastq.remove_duplicates"),
        vec![
            ToolId::from_static("fastuniq"),
            ToolId::from_static("clumpify"),
        ],
        true,
    );
    assert_eq!(
        filtered,
        vec![
            ToolId::from_static("fastuniq"),
            ToolId::from_static("clumpify"),
        ]
    );
}

#[test]
fn domain_layout_filter_excludes_paired_only_correction_backends_for_single_end_inputs() {
    let filtered = bijux_dna_domain_fastq::filter_tools_for_input_layout(
        &StageId::from_static("fastq.correct_errors"),
        vec![
            ToolId::from_static("rcorrector"),
            ToolId::from_static("musket"),
        ],
        false,
    );
    assert_eq!(filtered.len(), 2);
}
