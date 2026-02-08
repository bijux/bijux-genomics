use std::fs;
use std::path::PathBuf;

#[test]
fn pipeline_registry_order_documented() {
    let doc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("PIPELINES.md");
    let content = fs::read_to_string(&doc).expect("read PIPELINES.md");
    assert!(
        content.contains("fastq-to-fastq__default__v1"),
        "PIPELINES.md must include fastq-to-fastq__default__v1"
    );
    assert!(
        content.contains("bam-to-bam__default__v1"),
        "PIPELINES.md must include bam-to-bam__default__v1"
    );
}
