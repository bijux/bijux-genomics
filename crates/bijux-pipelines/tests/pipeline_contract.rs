use bijux_domain_bam::BamStage;
use bijux_domain_fastq::{contract_for_stage, parse_effective_params};
use bijux_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles};

#[test]
fn pipeline_profiles_reference_known_stages_and_defaults() {
    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());

    for profile in profiles {
        for node in &profile.graph {
            let stage_id = node.stage_id.as_str();
            assert!(
                profile.defaults.tools.contains_key(stage_id),
                "missing default tool for {stage_id} in profile {}",
                profile.id
            );
            assert!(
                profile.defaults.params.contains_key(stage_id),
                "missing default params for {stage_id} in profile {}",
                profile.id
            );

            if stage_id.starts_with("fastq.") {
                assert!(
                    contract_for_stage(stage_id).is_some(),
                    "unknown FASTQ stage {stage_id} in profile {}",
                    profile.id
                );
                let Some(params) = profile.defaults.params.get(stage_id) else {
                    panic!(
                        "missing FASTQ params for {stage_id} in profile {}",
                        profile.id
                    );
                };
                let parsed = parse_effective_params(stage_id, params)
                    .unwrap_or_else(|| panic!("failed to parse FASTQ params for {stage_id}"));
                assert!(
                    parsed.missing_required_fields().is_empty(),
                    "missing required FASTQ fields for {stage_id} in profile {}",
                    profile.id
                );
            } else if stage_id.starts_with("bam.") {
                let stage = BamStage::try_from(stage_id)
                    .unwrap_or_else(|_| panic!("unknown BAM stage {stage_id}"));
                let Some(params) = profile.defaults.params.get(stage_id) else {
                    panic!(
                        "missing BAM params for {stage_id} in profile {}",
                        profile.id
                    );
                };
                stage
                    .parse_effective_params(params)
                    .unwrap_or_else(|_| panic!("failed to parse BAM params for {stage_id}"));
            } else if stage_id.starts_with("cross.") {
                // Cross-domain placeholders are allowed without a domain registry.
            } else if stage_id.starts_with("core.") {
                // Core stages are validated at runtime.
            } else {
                panic!("unknown stage prefix for {stage_id}");
            }
        }
    }
}
