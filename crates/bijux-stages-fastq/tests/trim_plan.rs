use anyhow::Result;

#[test]
fn trim_output_names_are_defined_for_known_tools() {
    assert_eq!(
        bijux_stages_fastq::fastq::trim::trim_output_name("fastp"),
        Some("fastp.fastq.gz")
    );
    assert_eq!(
        bijux_stages_fastq::fastq::trim::trim_output_name("trimmomatic"),
        Some("trimmomatic.fastq.gz")
    );
    assert_eq!(
        bijux_stages_fastq::fastq::trim::trim_output_name("unknown"),
        None
    );
}

#[test]
fn plan_trim_builds_expected_paths() -> Result<()> {
    let plan = bijux_stages_fastq::fastq::trim::plan(
        "fastp",
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    )?;
    assert_eq!(plan.output.to_string_lossy(), "out/fastp.fastq.gz");
    Ok(())
}

#[test]
fn plan_trim_rejects_unknown_tool() {
    match bijux_stages_fastq::fastq::trim::plan(
        "mystery",
        std::path::Path::new("reads.fastq.gz"),
        std::path::Path::new("out"),
    ) {
        Ok(_) => panic!("expected unsupported trim tool"),
        Err(err) => assert!(err.to_string().contains("unsupported trim tool")),
    }
}
