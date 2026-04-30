use std::process::Command;

#[path = "support/mod.rs"]
mod support;

fn command_output(command: &mut Command) -> anyhow::Result<serde_json::Value> {
    let output = command.output()?;
    if !output.status.success() {
        anyhow::bail!(
            "command failed: {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

#[test]
fn domain_registry_bundle_binary_emits_release_bundle_json() -> anyhow::Result<()> {
    let root = support::repo_root();
    let json = command_output(
        Command::new(env!("CARGO_BIN_EXE_domain_registry_bundle"))
            .arg("--domain-dir")
            .arg(root.join("domain")),
    )?;
    let domains = json
        .get("domains")
        .and_then(serde_json::Value::as_array)
        .expect("bundle must expose domains array");
    assert_eq!(domains.len(), 3);
    assert!(
        domains.iter().any(|domain| {
            domain.get("domain_id").and_then(serde_json::Value::as_str) == Some("vcf")
        }),
        "bundle binary must emit the VCF registry slice"
    );
    Ok(())
}

#[test]
fn domain_registry_query_binary_filters_stage_registry_rows() -> anyhow::Result<()> {
    let root = support::repo_root();
    let json = command_output(
        Command::new(env!("CARGO_BIN_EXE_domain_registry_query"))
            .arg("--domain-dir")
            .arg(root.join("domain"))
            .arg("--kind")
            .arg("stages")
            .arg("--domain")
            .arg("fastq")
            .arg("--stage-id")
            .arg("fastq.trim_reads"),
    )?;
    let rows = json.as_array().expect("query must return array");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(
        row.get("stage_id").and_then(serde_json::Value::as_str),
        Some("fastq.trim_reads")
    );
    assert!(
        row.get("parameters")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|parameters| !parameters.is_empty()),
        "query output must preserve default parameter contracts"
    );
    Ok(())
}
