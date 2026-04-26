use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use bijux_dna_pipelines::registry::PipelineRegistry;

#[test]
fn pipeline_registry_ids_are_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs").join("PIPELINES.md");
    let content = fs::read_to_string(&doc).expect("read PIPELINES.md");

    let registry_ids: BTreeSet<String> = PipelineRegistry::v1()
        .list(true)
        .into_iter()
        .map(|profile| profile.id.to_string())
        .collect();
    let documented_ids = documented_pipeline_ids(&content);

    assert_eq!(
        documented_ids, registry_ids,
        "PIPELINES.md must document the exact registry pipeline ID set"
    );
}

fn documented_pipeline_ids(content: &str) -> BTreeSet<String> {
    content
        .split('`')
        .filter(|segment| segment.contains("__") && segment.ends_with("__v1"))
        .map(str::to_string)
        .collect()
}
