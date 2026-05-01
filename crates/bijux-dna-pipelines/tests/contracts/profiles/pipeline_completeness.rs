use bijux_dna_core::ids::StageId;
use bijux_dna_core::prelude::id_catalog;
use bijux_dna_pipelines::cross::{fastq_to_bam_adna_shotgun_profile, fastq_to_bam_default_profile};
use bijux_dna_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles, vcf_profiles};
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
        assert!(sections.contains(&"fastq"), "missing fastq report section for {}", profile.id);
    }
    if profile
        .capabilities
        .input_domains
        .iter()
        .chain(profile.capabilities.output_domains.iter())
        .any(|domain| matches!(domain, Domain::Vcf))
    {
        assert!(sections.contains(&"vcf"), "missing vcf report section for {}", profile.id);
    }
    if profile
        .capabilities
        .input_domains
        .iter()
        .chain(profile.capabilities.output_domains.iter())
        .any(|domain| matches!(domain, Domain::Bam))
    {
        assert!(sections.contains(&"bam"), "missing bam report section for {}", profile.id);
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
    profiles.extend(vcf_profiles());

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
            let stage = bijux_dna_core::ids::StageId::new(node.clone());
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
    for required in ["fastq.trim_reads", "fastq.filter_reads", "fastq.report_qc"] {
        assert!(
            required_stages.iter().any(|stage| stage == required),
            "default FASTQ profile missing metrics-critical stage {required}"
        );
    }
    assert!(
        profile.capabilities.required_metrics.contains(&"fastq.metrics"),
        "default FASTQ profile must require fastq.metrics bundle"
    );
}

#[test]
fn default_cross_pipeline_uses_modern_source_defaults() {
    let profile = fastq_to_bam_default_profile();
    let trim_stage = StageId::from_static(id_catalog::FASTQ_TRIM);
    let terminal_damage_stage = StageId::from_static(id_catalog::FASTQ_TRIM_TERMINAL_DAMAGE);

    assert_eq!(
        profile.defaults.tools.get(&trim_stage).map(|tool| tool.as_str()),
        Some(id_catalog::TOOL_FASTP),
        "default FASTQ-to-BAM must inherit the modern FASTQ trim tool"
    );
    assert!(
        !profile.defaults.tools.contains_key(&terminal_damage_stage),
        "default FASTQ-to-BAM must not inherit aDNA terminal-damage trimming"
    );
}

#[test]
fn adna_cross_pipeline_keeps_adna_source_defaults() {
    let profile = fastq_to_bam_adna_shotgun_profile();
    let trim_stage = StageId::from_static(id_catalog::FASTQ_TRIM);
    let terminal_damage_stage = StageId::from_static(id_catalog::FASTQ_TRIM_TERMINAL_DAMAGE);

    assert_eq!(
        profile.defaults.tools.get(&trim_stage).map(|tool| tool.as_str()),
        Some(id_catalog::TOOL_ADAPTERREMOVAL),
        "aDNA FASTQ-to-BAM must keep the aDNA trim tool"
    );
    assert!(
        profile.defaults.tools.contains_key(&terminal_damage_stage),
        "aDNA FASTQ-to-BAM must keep terminal-damage trimming"
    );
}

#[test]
fn cross_pipelines_declare_reference_not_bam_input_artifact() {
    for profile in [fastq_to_bam_default_profile(), fastq_to_bam_adna_shotgun_profile()] {
        assert!(
            profile
                .capabilities
                .input_artifacts
                .contains(&bijux_dna_pipelines::ArtifactType::FastqReads),
            "{} must consume FASTQ reads",
            profile.id
        );
        assert!(
            profile
                .capabilities
                .input_artifacts
                .contains(&bijux_dna_pipelines::ArtifactType::ReferenceFasta),
            "{} must consume a reference FASTA",
            profile.id
        );
        assert!(
            !profile.capabilities.input_artifacts.contains(&bijux_dna_pipelines::ArtifactType::Bam),
            "{} must not declare BAM as an input artifact",
            profile.id
        );
    }
}

#[test]
fn cross_fastq_to_bam_profiles_require_mapping_summary_before_coverage() {
    for profile in [fastq_to_bam_default_profile(), fastq_to_bam_adna_shotgun_profile()] {
        let stages = &profile.capabilities.required_stages;
        let mapping_idx = stages
            .iter()
            .position(|stage| stage == "bam.mapping_summary")
            .unwrap_or_else(|| panic!("{} missing bam.mapping_summary", profile.id));
        let coverage_idx = stages
            .iter()
            .position(|stage| stage == "bam.coverage")
            .unwrap_or_else(|| panic!("{} missing bam.coverage", profile.id));
        assert!(
            mapping_idx < coverage_idx,
            "{} must place bam.mapping_summary before bam.coverage",
            profile.id
        );
    }
}
