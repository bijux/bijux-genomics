use bijux_dna_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles};
use bijux_dna_pipelines::{fastq::fastq_default_profile, Domain, PipelineProfile};

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
        for node in &profile.capabilities.required_stages {
            let stage = bijux_dna_core::ids::StageId::from_static(node);
            assert!(
                ledger.params.contains_key(&stage),
                "defaults ledger missing params for {} in {}",
                node,
                profile.id
            );
            assert!(
                ledger.param_provenance.contains_key(&stage),
                "defaults ledger missing params provenance for {} in {}",
                node,
                profile.id
            );
        }
        assert!(
            !profile.capabilities.required_stages.is_empty(),
            "missing stage list for {}",
            profile.id
        );
        assert!(
            !profile.capabilities.report_sections.is_empty(),
            "missing report sections for {}",
            profile.id
        );
        assert!(
            !profile.capabilities.required_stages.is_empty(),
            "missing required stages for {}",
            profile.id
        );
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
        assert!(
            !profile.capabilities.input_artifacts.is_empty(),
            "missing input artifacts for {}",
            profile.id
        );
        assert!(
            !profile.capabilities.output_artifacts.is_empty(),
            "missing mandatory outputs for {}",
            profile.id
        );
        assert!(
            !profile.capabilities.required_report_sections.is_empty(),
            "missing required report sections for {}",
            profile.id
        );
        assert!(
            !profile.capabilities.required_metrics_bundles.is_empty(),
            "missing required metrics bundles for {}",
            profile.id
        );
        assert_report_sections(&profile);
        let contract = profile.contract();
        assert!(
            !contract.required_stages.is_empty(),
            "pipeline contract missing stages for {}",
            profile.id
        );
        assert!(
            !contract.required_artifacts.is_empty(),
            "pipeline contract missing required artifacts for {}",
            profile.id
        );
        assert!(
            !contract.required_metrics_bundles.is_empty(),
            "pipeline contract missing required metrics bundles for {}",
            profile.id
        );
        assert!(
            !contract.required_report_sections.is_empty(),
            "pipeline contract missing report sections for {}",
            profile.id
        );
    }
}

#[test]
fn default_fastq_pipeline_declares_required_metrics_objects() {
    let profile = fastq_default_profile();
    let required_stages = &profile.capabilities.required_stages;
    for required in ["fastq.trim", "fastq.filter", "fastq.qc_post"] {
        assert!(
            required_stages.iter().any(|stage| *stage == required),
            "default FASTQ profile missing metrics-critical stage {required}"
        );
    }
    assert!(
        profile
            .capabilities
            .required_metrics
            .iter()
            .any(|metric| *metric == "fastq.metrics"),
        "default FASTQ profile must require fastq.metrics bundle"
    );
}
