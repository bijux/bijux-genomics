use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_containers::api::{
    container_credential_reference_schema, load_container_credential_reference,
    CONTAINER_CREDENTIAL_REFERENCE_SCHEMA_V1,
};

fn repository_root() -> Result<std::path::PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?)
}

#[test]
fn tracked_run_credentials_are_typed_and_schema_owned() -> Result<()> {
    let root = repository_root()?;
    let runs = root.join("containers/runs");
    let mut count = 0;
    for entry in std::fs::read_dir(runs)? {
        let path = entry?.path().join("config-credential.yml");
        if !path.is_file() {
            continue;
        }
        let contract = load_container_credential_reference(&path)?;
        assert_eq!(contract.schema_version, CONTAINER_CREDENTIAL_REFERENCE_SCHEMA_V1);
        count += 1;
    }
    assert!(count > 0, "expected tracked credential references");

    let checked: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        root.join("containers/contracts/config-credential.schema.json"),
    )?)?;
    assert_eq!(checked, container_credential_reference_schema());
    Ok(())
}

#[test]
fn credential_admission_rejects_secret_values_and_inconsistent_references() -> Result<()> {
    let root = tempfile::tempdir()?;
    let secret = root.path().join("secret.yml");
    std::fs::write(
        &secret,
        "schema_version: bijux.container.credential_reference.v1\ncredentials_required: true\nreferences:\n  - id: access\n    purpose: download\n    source_type: environment\n    variable_names: [TOKEN]\n    failure_behavior: block\n    token: exposed\nredaction:\n  publish_values: false\n",
    )?;
    let error = load_container_credential_reference(&secret)
        .err()
        .context("embedded secret value was accepted")?;
    assert!(format!("{error:#}").contains("unknown field `token`"));

    let inconsistent = root.path().join("inconsistent.yml");
    std::fs::write(
        &inconsistent,
        "schema_version: bijux.container.credential_reference.v1\ncredentials_required: false\nreferences:\n  - id: access\n    purpose: download\n    source_type: environment\n    variable_names: [TOKEN]\n    failure_behavior: block\nredaction:\n  publish_values: false\n",
    )?;
    let error = load_container_credential_reference(&inconsistent)
        .err()
        .context("inconsistent credential declaration was accepted")?;
    assert!(format!("{error:#}").contains("credentials_required must agree"));
    Ok(())
}
