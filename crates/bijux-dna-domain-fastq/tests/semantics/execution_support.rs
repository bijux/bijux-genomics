use std::collections::BTreeSet;

use bijux_dna_domain_fastq::{
    all_stage_execution_support, execution_closed_stage_ids, execution_declared_only_stage_ids,
    FASTQ_STAGE_ID_CATALOG,
};

#[test]
fn execution_support_manifest_covers_every_fastq_stage() {
    let domain_stage_ids = FASTQ_STAGE_ID_CATALOG
        .iter()
        .map(|stage| stage.to_string())
        .collect::<BTreeSet<_>>();
    let execution_stage_ids = all_stage_execution_support()
        .into_iter()
        .map(|stage| stage.stage_id.to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        execution_stage_ids, domain_stage_ids,
        "execution support drifted from FASTQ stage catalog",
    );
}

#[test]
fn closed_stages_have_defaults_and_declared_only_stages_do_not() {
    for support in all_stage_execution_support() {
        match support.execution_status {
            bijux_dna_domain_fastq::ExecutionStatus::Closed => {
                assert!(
                    support.default_tool.is_some(),
                    "closed stage {} must declare a default tool",
                    support.stage_id.as_str(),
                );
                assert!(
                    !support.admitted_tools.is_empty(),
                    "closed stage {} must admit at least one tool",
                    support.stage_id.as_str(),
                );
            }
            bijux_dna_domain_fastq::ExecutionStatus::DeclaredOnly => {
                assert!(
                    support.default_tool.is_none(),
                    "declared-only stage {} must not expose a default tool",
                    support.stage_id.as_str(),
                );
                assert!(
                    support.admitted_tools.is_empty(),
                    "declared-only stage {} must not admit execution tools",
                    support.stage_id.as_str(),
                );
            }
        }
    }
}

#[test]
fn execution_support_separates_closed_and_declared_only_stage_sets() {
    let closed = execution_closed_stage_ids()
        .into_iter()
        .map(|stage| stage.to_string())
        .collect::<BTreeSet<_>>();
    let declared_only = execution_declared_only_stage_ids()
        .into_iter()
        .map(|stage| stage.to_string())
        .collect::<BTreeSet<_>>();

    assert!(
        declared_only.contains("fastq.infer_asvs"),
        "planned FASTQ ASV inference must stay declared-only until the runtime closes",
    );
    assert!(
        !closed.contains("fastq.infer_asvs"),
        "declared-only stages must not appear in the closed execution set",
    );
}
