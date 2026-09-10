use super::*;

#[test]
fn local_case_output_admission_preserves_relative_and_absolute_disposable_paths() -> Result<()> {
    let root = std::env::current_dir()?.join("repository with spaces");
    let relative = Path::new("artifacts/runs/bench/local-smoke/fastq.trim_reads");
    let absolute = root.join(relative);
    for output in [relative, absolute.as_path()] {
        assert_eq!(
            local_smoke_case_output_dir(&root, output, "sample with spaces", "cutadapt")?,
            output.join("sample with spaces").join("cutadapt")
        );
    }
    Ok(())
}

#[test]
fn local_case_output_admission_rejects_root_escape_and_non_disposable_destinations() -> Result<()> {
    let root = std::env::current_dir()?.join("repository");
    for output in [
        "",
        ".",
        "assets",
        "benchmarks/readiness",
        "artifacts/benchmark-smoke/readiness",
        "artifacts/runs/bench/local-smoke/../../../../docs",
        "artifacts/runs/bench/local-smoke-other",
    ] {
        assert!(
            local_smoke_case_output_dir(&root, Path::new(output), "sample", "cutadapt").is_err(),
            "accepted {output}"
        );
    }
    assert!(local_smoke_case_output_dir(&root, &root.join("docs"), "sample", "cutadapt").is_err());
    assert!(local_smoke_case_output_dir(
        &root,
        &std::env::current_dir()?.join(LOCAL_CASE_OUTPUT_ROOT),
        "sample",
        "cutadapt"
    )
    .is_err());
    Ok(())
}

#[test]
fn local_case_output_admission_rejects_sample_or_tool_path_injection() -> Result<()> {
    let root = std::env::current_dir()?;
    let output = Path::new(LOCAL_CASE_OUTPUT_ROOT);
    for value in ["", " ", ".", "..", "a/b", "sample/", "sample/.", "/absolute", "a\\b", "\0"] {
        assert!(local_smoke_case_output_dir(&root, output, value, "cutadapt").is_err());
        assert!(local_smoke_case_output_dir(&root, output, "sample", value).is_err());
    }
    Ok(())
}
