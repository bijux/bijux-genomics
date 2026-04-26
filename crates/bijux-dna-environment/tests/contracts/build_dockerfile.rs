use bijux_dna_environment::build::extract_version_from_dockerfile;
use bijux_dna_environment::resolve::EnvError;

fn write_dockerfile(name: &str, contents: &[u8]) -> Result<std::path::PathBuf, EnvError> {
    let path = std::env::temp_dir().join(name);
    bijux_dna_infra::atomic_write_bytes(&path, contents).map_err(std::io::Error::other)?;
    Ok(path)
}

#[test]
fn extract_version_from_dockerfile_parses() -> Result<(), EnvError> {
    let path = write_dockerfile(
        "bijux_test_fastp.Dockerfile",
        b"FROM ubuntu:20.04\nARG VERSION_FASTP=0.23.4\n",
    )?;
    let version = extract_version_from_dockerfile(&path, "fastp")?;
    assert_eq!(version, "0.23.4");
    let _ = bijux_dna_infra::remove_file(&path);
    Ok(())
}

#[test]
fn extract_version_from_dockerfile_ignores_unrelated_arg_names() -> Result<(), EnvError> {
    let path = write_dockerfile(
        "bijux_test_fastqvalidator_missing_version.Dockerfile",
        b"FROM ubuntu:20.04\nARG BASE_VERSION=24.04\n",
    )?;
    let error = extract_version_from_dockerfile(&path, "fastqvalidator")
        .err()
        .ok_or_else(|| EnvError::Dockerfile("expected missing tool version".to_string()))?;
    assert!(error.to_string().contains("no version ARG found for tool fastqvalidator"));
    let _ = bijux_dna_infra::remove_file(&path);
    Ok(())
}

#[test]
fn extract_version_from_dockerfile_strips_optional_quotes() -> Result<(), EnvError> {
    let path = write_dockerfile(
        "bijux_test_fastp_quoted_version.Dockerfile",
        b"FROM ubuntu:20.04\nARG VERSION_FASTP=\"0.23.4\"\n",
    )?;
    let version = extract_version_from_dockerfile(&path, "fastp")?;
    assert_eq!(version, "0.23.4");
    let _ = bijux_dna_infra::remove_file(&path);
    Ok(())
}

#[test]
fn extract_version_from_dockerfile_trims_tool_selector() -> Result<(), EnvError> {
    let path = write_dockerfile(
        "bijux_test_fastp_spaced_tool.Dockerfile",
        b"FROM ubuntu:20.04\nARG VERSION_FASTP=0.23.4\n",
    )?;
    let version = extract_version_from_dockerfile(&path, " fastp ")?;
    assert_eq!(version, "0.23.4");
    let _ = bijux_dna_infra::remove_file(&path);
    Ok(())
}

#[test]
fn extract_version_from_dockerfile_rejects_empty_tool_selector() -> Result<(), EnvError> {
    let path = write_dockerfile(
        "bijux_test_empty_tool_selector.Dockerfile",
        b"FROM ubuntu:20.04\nARG VERSION_FASTP=0.23.4\n",
    )?;
    let error = extract_version_from_dockerfile(&path, " ")
        .err()
        .ok_or_else(|| EnvError::Dockerfile("expected empty tool selector rejection".to_string()))?;
    assert!(error.to_string().contains("tool name is empty"));
    let _ = bijux_dna_infra::remove_file(&path);
    Ok(())
}
