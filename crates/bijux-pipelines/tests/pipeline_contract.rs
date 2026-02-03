use bijux_domain_bam::{bam_stage_is_complete, bam_stage_is_stable, BamStage};
use bijux_domain_fastq::{
    canonical_stage_order, forbidden_transitions, optional_branches, stage_criticality,
    StageCriticality,
};
use bijux_domain_fastq::{contract_for_stage, parse_effective_params};
use bijux_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles};
use bijux_pipelines::{validate_pipeline_id, StabilityTier};

#[test]
fn pipeline_profiles_reference_known_stages_and_defaults() {
    let mut profiles = Vec::new();
    profiles.extend(fastq_profiles());
    profiles.extend(bam_profiles());
    profiles.extend(cross_profiles());

    for profile in profiles {
        validate_pipeline_id(profile.id)
            .unwrap_or_else(|_| panic!("invalid pipeline id {}", profile.id));
        let mut stage_positions = std::collections::BTreeMap::new();
        for (idx, node) in profile.graph.iter().enumerate() {
            stage_positions.insert(node.stage_id.as_str(), idx);
        }
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
                if profile.stability == StabilityTier::Stable {
                    if let Some(criticality) = stage_criticality(stage_id) {
                        assert!(
                            criticality != StageCriticality::Experimental,
                            "stable pipeline {} includes experimental FASTQ stage {stage_id}",
                            profile.id
                        );
                    }
                }
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

        let fastq_in_profile = profile
            .graph
            .iter()
            .any(|node| node.stage_id.starts_with("fastq."));
        if fastq_in_profile {
            let canonical = canonical_stage_order();
            for window in canonical.windows(2) {
                let first = window[0];
                let second = window[1];
                if let (Some(a), Some(b)) =
                    (stage_positions.get(first), stage_positions.get(second))
                {
                    assert!(
                        a < b,
                        "FASTQ stage order violated in {}: {first} must precede {second}",
                        profile.id
                    );
                }
            }
            for (optional, predecessors) in optional_branches() {
                if let Some(optional_idx) = stage_positions.get(optional) {
                    for predecessor in predecessors {
                        if let Some(pred_idx) = stage_positions.get(predecessor) {
                            assert!(
                                pred_idx < optional_idx,
                                "FASTQ optional stage {optional} must follow {predecessor} in {}",
                                profile.id
                            );
                        }
                    }
                }
            }
            for (a, b) in forbidden_transitions() {
                if let (Some(a_idx), Some(b_idx)) =
                    (stage_positions.get(a), stage_positions.get(b))
                {
                    assert!(
                        a_idx + 1 != *b_idx,
                        "FASTQ forbidden transition in {}: {a} must not directly precede {b}",
                        profile.id
                    );
                }
            }
        }
    }
}
