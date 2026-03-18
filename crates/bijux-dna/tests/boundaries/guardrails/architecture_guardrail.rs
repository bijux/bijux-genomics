use std::path::PathBuf;

#[test]
fn cli_does_not_contain_fastq_exec_modules() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    let path = repo_root.join("../../../src/fastq_exec");
    assert!(
        !path.exists(),
        "fastq_exec modules must not exist in CLI; move logic to bijux-dna-api"
    );
}
