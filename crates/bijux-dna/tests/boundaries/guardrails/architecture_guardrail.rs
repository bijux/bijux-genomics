#[test]
fn cli_does_not_contain_fastq_exec_modules() {
    let path = super::support::crate_src("bijux-dna")
        .unwrap_or_else(|err| panic!("resolve bijux-dna src: {err}"))
        .join("fastq_exec");
    assert!(
        !path.exists(),
        "fastq_exec modules must not exist in CLI; move logic to bijux-dna-api"
    );
}
