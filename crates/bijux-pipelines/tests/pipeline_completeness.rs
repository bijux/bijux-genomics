use bijux_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles};
use bijux_pipelines::{Domain, PipelineProfile};

fn assert_report_sections(profile: &PipelineProfile) {
    let sections = &profile.capabilities.report_sections;
    if profile
        .capabilities
        .input_domains
        .iter()
        .chain(profile.capabilities.output_domains.iter())
        .any(|domain| matches!(domain, Domain::Fastq))
    {
        assert!(
            sections.contains(&"fastq"),
            "missing fastq report section for {}",
            profile.id
        );
    }
    if profile
        .capabilities
        .input_domains
        .iter()
        .chain(profile.capabilities.output_domains.iter())
        .any(|domain| matches!(domain, Domain::Bam))
    {
        assert!(
            sections.contains(&"bam"),
            "missing bam report section for {}",
            profile.id
        );
    }
    if profile
        .capabilities
        .input_domains
        .iter()
        .chain(profile.capabilities.output_domains.iter())
        .any(|domain| matches!(domain, Domain::Cross))
    {
        assert!(
            sections.contains(&"cross.handoff"),
            "missing cross.handoff report section for {}",
            profile.id
        );
    }
}

#[test]
fn pipeline_profiles_are_complete() {
    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());

    for profile in profiles {
        assert!(
            !profile.defaults_ledger_ref.is_empty(),
            "missing defaults ledger ref for {}",
            profile.id
        );
        assert_eq!(
            profile.defaults_ledger_ref, "defaults_ledger.json",
            "unexpected defaults ledger ref for {}",
            profile.id
        );
        let ledger = profile.defaults_ledger();
        assert_eq!(
            ledger.pipeline_id.as_str(),
            profile.id.as_str(),
            "defaults ledger id mismatch for {}",
            profile.id
        );
        for node in &profile.graph {
            assert!(
                ledger.tools.contains_key(&node.stage_id),
                "defaults ledger missing tool for {} in {}",
                node.stage_id,
                profile.id
            );
            assert!(
                ledger.params.contains_key(&node.stage_id),
                "defaults ledger missing params for {} in {}",
                node.stage_id,
                profile.id
            );
        }
        assert!(
            !profile.capabilities.required_stages.is_empty(),
            "missing required stages for {}",
            profile.id
        );
        for stage in &profile.capabilities.required_stages {
            assert!(
                profile.graph.iter().any(|node| node.stage_id == *stage),
                "required stage {stage} missing from graph for {}",
                profile.id
            );
        }
        assert!(
            !profile.capabilities.required_metrics.is_empty(),
            "missing required metrics for {}",
            profile.id
        );
        assert!(
            !profile.capabilities.required_artifacts.is_empty(),
            "missing required artifacts for {}",
            profile.id
        );
        assert_report_sections(&profile);
    }
}
