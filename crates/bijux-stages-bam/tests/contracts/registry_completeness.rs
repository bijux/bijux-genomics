use bijux_domain_bam::BamStage;

fn sort_stages(mut stages: Vec<BamStage>) -> Vec<String> {
    stages.sort_by_key(|stage| stage.as_str().to_string());
    stages
        .into_iter()
        .map(|stage| stage.as_str().to_string())
        .collect()
}

#[test]
fn bam_stage_registry_is_complete() {
    let domain = sort_stages(BamStage::all().to_vec());
    let implemented = sort_stages(bijux_stages_bam::implemented_stages());
    assert_eq!(
        domain, implemented,
        "stages-bam registry must match domain list"
    );
}
