use std::collections::HashSet;

use bijux_pipelines::registry::{bam_profiles, cross_profiles, fastq_profiles, PipelineRegistry};

#[test]
fn pipeline_ids_are_unique_and_stable() {
    let mut ids = HashSet::new();
    let mut all_ids = Vec::new();

    for profile in fastq_profiles()
        .into_iter()
        .chain(bam_profiles())
        .chain(cross_profiles())
    {
        let id = profile.id.as_str().to_string();
        assert!(ids.insert(id.clone()), "duplicate pipeline id: {id}");
        all_ids.push(id);
    }

    let registry_ids = PipelineRegistry::v1()
        .list(true)
        .into_iter()
        .map(|profile| profile.id.as_str().to_string())
        .collect::<Vec<_>>();
    let mut sorted = registry_ids.clone();
    sorted.sort();
    assert_eq!(
        registry_ids, sorted,
        "pipeline registry ordering must be stable and sorted"
    );
    assert_eq!(
        all_ids.len(),
        ids.len(),
        "pipeline id registry should not contain duplicates"
    );
}
