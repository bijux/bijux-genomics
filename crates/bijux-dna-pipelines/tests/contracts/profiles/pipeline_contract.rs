use bijux_dna_core::ids::StageId;
use bijux_dna_domain_bam::{bam_stage_is_complete, bam_stage_is_stable, BamStage};
use bijux_dna_domain_fastq::{
    canonical_stage_order, fastq_stage_is_stable, forbidden_transitions, optional_branches,
};
use bijux_dna_domain_fastq::{contract_for_stage, parse_effective_params};
use bijux_dna_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles};
use bijux_dna_pipelines::{validate_pipeline_id, StabilityTier};
use bijux_dna_pipelines::vcf::{validate_vcf_profile, vcf_minimal_profile};

#[test]
fn pipeline_profiles_reference_known_stages_and_defaults() {
    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());

    for profile in profiles {
        validate_pipeline_id(&profile.id)
            .unwrap_or_else(|_| panic!("invalid pipeline id {}", profile.id));
        for stage_id in &profile.capabilities.required_stages {
            let stage_key = StageId::new(stage_id.clone());
            assert!(
                profile.defaults.params.contains_key(&stage_key),
                "missing default params for {stage_id} in profile {}",
                profile.id
            );

            if stage_id.starts_with("fastq.") {
                let stage = StageId::new(stage_id.clone());
                assert!(
                    contract_for_stage(stage_id).is_some(),
                    "unknown FASTQ stage {stage_id} in profile {}",
                    profile.id
                );
                if profile.stability == StabilityTier::Stable {
                    assert!(
                        fastq_stage_is_stable(&stage),
                        "stable pipeline {} includes non-stable FASTQ stage {stage_id}",
                        profile.id
                    );
                }
                let Some(params) = profile.defaults.params.get(&stage_key) else {
                    panic!("missing FASTQ params for {stage_id} in profile {}", profile.id);
                };
                let parsed = parse_effective_params(&stage, &params.to_json())
                    .unwrap_or_else(|| panic!("failed to parse FASTQ params for {stage_id}"));
                assert!(
                    parsed.missing_required_fields().is_empty(),
                    "missing required FASTQ fields for {stage_id} in profile {}",
                    profile.id
                );
            } else if stage_id.starts_with("bam.") {
                let stage = BamStage::try_from(stage_id.as_str())
                    .unwrap_or_else(|_| panic!("unknown BAM stage {stage_id}"));
                if profile.stability == StabilityTier::Stable {
                    assert!(
                        bam_stage_is_stable(stage),
                        "stable pipeline {} includes non-stable BAM stage {stage_id}",
                        profile.id
                    );
                    assert!(
                        bam_stage_is_complete(stage),
                        "stable pipeline {} includes incomplete BAM stage {stage_id}",
                        profile.id
                    );
                }
                let stage_key = StageId::new(stage_id.clone());
                let Some(params) = profile.defaults.params.get(&stage_key) else {
                    panic!("missing BAM params for {stage_id} in profile {}", profile.id);
                };
                stage
                    .parse_effective_params(&params.to_json())
                    .unwrap_or_else(|_| panic!("failed to parse BAM params for {stage_id}"));
            } else if stage_id.starts_with("vcf.") {
                let validation = validate_vcf_profile(&vcf_minimal_profile());
                assert!(validation.valid, "vcf reference validation fixture must stay valid");
                let Some(params) = profile.defaults.params.get(&stage_key) else {
                    panic!("missing VCF params for {stage_id} in profile {}", profile.id);
                };
                assert!(
                    matches!(params, bijux_dna_pipelines::DefaultParams::Vcf(_)),
                    "profile {} stage {} must use VCF param kind",
                    profile.id,
                    stage_id
                );
            } else if stage_id.starts_with("cross.") {
                // Cross-domain placeholders are allowed without a domain registry.
            } else if stage_id.starts_with("core.") {
                // Core stages are validated at runtime.
            } else {
                panic!("unknown stage prefix for {stage_id}");
            }
        }

        let _ = (canonical_stage_order(), optional_branches(), forbidden_transitions());
    }
}
