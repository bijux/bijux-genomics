#[test]
fn cli_does_not_contain_fastq_exec_modules() {
    let path = super::support::repo_root()
        .unwrap_or_else(|err| panic!("resolve repo root: {err}"))
        .join("src/fastq_exec");
    assert!(
        !path.exists(),
        "fastq_exec modules must not exist in CLI; move logic to bijux-dna-api"
    );
}
