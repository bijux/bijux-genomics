use std::collections::BTreeSet;

use bijux_domain_fastq::stage_registry::STAGES;
use bijux_domain_fastq::tool_registry::canonical_tools_for_stage;

#[test]
fn stage_registry_is_complete() {
    let domain: BTreeSet<_> = STAGES.iter().map(|id| id.as_str().to_string()).collect();
    let stages: BTreeSet<_> = bijux_stages_fastq::fastq::registry()
        .iter()
        .map(|entry| entry.id.as_str().to_string())
        .collect();
    assert_eq!(
        domain, stages,
        "stages-fastq must implement exactly the domain catalog"
    );
}

#[test]
fn tool_registry_is_complete() {
    for stage_id in STAGES {
        let domain_tools: BTreeSet<_> = canonical_tools_for_stage(&stage_id)
            .iter()
            .map(|tool| (*tool).to_string())
            .collect();
        let stage_tools: BTreeSet<_> =
            bijux_stages_fastq::tools::registry::allowed_tools_for_stage(&stage_id)
                .into_iter()
                .collect();
        assert_eq!(
            domain_tools,
            stage_tools,
            "tools for {} must match domain catalog",
            stage_id.as_str()
        );
    }
}
