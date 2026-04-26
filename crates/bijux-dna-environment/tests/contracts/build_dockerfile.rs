use bijux_dna_environment::build::extract_version_from_dockerfile;
use bijux_dna_environment::resolve::EnvError;

fn write_dockerfile(
    root: &std::path::Path,
    name: &str,
    contents: &[u8],
) -> Result<std::path::PathBuf, EnvError> {
    let path = root.join(name);
    bijux_dna_infra::atomic_write_bytes(&path, contents).map_err(std::io::Error::other)?;
    Ok(path)
}

#[test]
fn extract_version_from_dockerfile_parses() -> Result<(), EnvError> {
    let temp = bijux_dna_testkit::tempdir_for("environment-dockerfile-version");
    let path = write_dockerfile(
        temp.path(),
        "bijux_test_fastp.Dockerfile",
        b"FROM ubuntu:20.04\nARG VERSION_FASTP=0.23.4\n",
    )?;
    let version = extract_version_from_dockerfile(&path, "fastp")?;
    assert_eq!(version, "0.23.4");
    Ok(())
}

#[test]
fn extract_version_from_dockerfile_ignores_unrelated_arg_names() -> Result<(), EnvError> {
    let temp = bijux_dna_testkit::tempdir_for("environment-dockerfile-missing-version");
    let path = write_dockerfile(
        temp.path(),
        "bijux_test_fastqvalidator_missing_version.Dockerfile",
        b"FROM ubuntu:20.04\nARG BASE_VERSION=24.04\n",
    )?;
    let error = extract_version_from_dockerfile(&path, "fastqvalidator")
        .err()
        .ok_or_else(|| EnvError::Dockerfile("expected missing tool version".to_string()))?;
    assert!(error.to_string().contains("no version ARG found for tool fastqvalidator"));
    Ok(())
}

#[test]
fn extract_version_from_dockerfile_strips_optional_quotes() -> Result<(), EnvError> {
    let temp = bijux_dna_testkit::tempdir_for("environment-dockerfile-quoted-version");
    let path = write_dockerfile(
        temp.path(),
        "bijux_test_fastp_quoted_version.Dockerfile",
        b"FROM ubuntu:20.04\nARG VERSION_FASTP=\"0.23.4\"\n",
    )?;
    let version = extract_version_from_dockerfile(&path, "fastp")?;
    assert_eq!(version, "0.23.4");
    Ok(())
}

#[test]
fn extract_version_from_dockerfile_trims_tool_selector() -> Result<(), EnvError> {
    let temp = bijux_dna_testkit::tempdir_for("environment-dockerfile-spaced-tool");
    let path = write_dockerfile(
        temp.path(),
        "bijux_test_fastp_spaced_tool.Dockerfile",
        b"FROM ubuntu:20.04\nARG VERSION_FASTP=0.23.4\n",
    )?;
    let version = extract_version_from_dockerfile(&path, " fastp ")?;
    assert_eq!(version, "0.23.4");
    Ok(())
}

#[test]
fn extract_version_from_dockerfile_rejects_empty_tool_selector() -> Result<(), EnvError> {
    let temp = bijux_dna_testkit::tempdir_for("environment-dockerfile-empty-tool");
    let path = write_dockerfile(
        temp.path(),
        "bijux_test_empty_tool_selector.Dockerfile",
        b"FROM ubuntu:20.04\nARG VERSION_FASTP=0.23.4\n",
    )?;
    let error = extract_version_from_dockerfile(&path, " ").err().ok_or_else(|| {
        EnvError::Dockerfile("expected empty tool selector rejection".to_string())
    })?;
    assert!(error.to_string().contains("tool name is empty"));
    Ok(())
}
