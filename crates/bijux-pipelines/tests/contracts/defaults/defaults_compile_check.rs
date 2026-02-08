use bijux_domain_bam::BamStage;
use bijux_domain_fastq::params::parse_effective_params;
use bijux_pipelines::registry::PipelineRegistry;

fn bam_stage_from_id(stage_id: &str) -> Option<BamStage> {
    BamStage::all()
        .iter()
        .copied()
        .find(|stage| stage.as_str() == stage_id)
}

#[test]
fn defaults_compile_against_domain_params() {
    let registry = PipelineRegistry::v1();
    for profile in registry.list(true) {
        let ledger = profile.defaults_ledger();
        for (stage_id, params_value) in ledger.params {
            let stage_id_str = stage_id.as_str();
            if stage_id_str.starts_with("fastq.") {
                let stage = bijux_core::ids::StageId::new(stage_id_str.to_string());
                let parsed = parse_effective_params(&stage, &params_value)
                    .unwrap_or_else(|| panic!("fastq defaults failed to parse for {stage_id_str}"));
                let missing = parsed.missing_required_fields();
                assert!(
                    missing.is_empty(),
                    "fastq defaults missing required fields for {}: {:?}",
                    stage_id_str,
                    missing
                );
                continue;
            }
            if stage_id_str.starts_with("bam.") {
                let stage = bam_stage_from_id(stage_id_str)
                    .unwrap_or_else(|| panic!("unknown bam stage in defaults: {stage_id_str}"));
                stage
                    .parse_effective_params(&params_value)
                    .unwrap_or_else(|err| {
                        panic!("bam defaults failed to parse for {stage_id_str}: {err}")
                    });
                continue;
            }
            if stage_id_str.starts_with("core.") {
                continue;
            }
            panic!("unknown stage namespace in defaults: {stage_id_str}");
        }
    }
}
