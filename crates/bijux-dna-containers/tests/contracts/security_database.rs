use std::fs;
use std::path::Path;

use anyhow::Result;
use bijux_dna_containers::api::verify_grype_database;

#[test]
fn grype_database_admission_is_source_status_and_manifest_bound() -> Result<()> {
    let fixture = tempfile::tempdir()?;
    let scanner = fixture.path().join("scanner");
    let evidence = fixture.path().join("evidence");
    fs::create_dir_all(scanner.join("bin"))?;
    fs::create_dir_all(evidence.join("database/6"))?;
    fs::write(scanner.join("bin/grype"), b"grype-binary")?;
    fs::write(evidence.join("job.sbatch"), b"acquisition-worker")?;
    fs::write(evidence.join("database/6/import.json"), b"{}\n")?;
    fs::write(evidence.join("database/6/last_update_check"), b"checked\n")?;
    fs::write(evidence.join("database/6/vulnerability.db"), b"database")?;
    fs::write(evidence.join("providers.txt"), b"ubuntu 3 fixture\n")?;
    bijux_dna_infra::atomic_write_json(
        &evidence.join("status.json"),
        &serde_json::json!({
            "schemaVersion": "v6.1.9",
            "from": format!(
                "https://grype.anchore.io/databases/v6/database.tar.zst?checksum=sha256%3A{}",
                "c".repeat(64)
            ),
            "built": "2026-08-08T06:22:53Z",
            "path": evidence.join("database/6/vulnerability.db"),
            "valid": true
        }),
    )?;
    let database_paths =
        ["database/6/import.json", "database/6/last_update_check", "database/6/vulnerability.db"];
    write_manifest(&evidence, "database-manifest.sha256", &database_paths)?;
    let status_sha256 = bijux_dna_infra::hash_file_sha256(&evidence.join("status.json"))?;
    let database_manifest_sha256 =
        bijux_dna_infra::hash_file_sha256(&evidence.join("database-manifest.sha256"))?;
    bijux_dna_infra::atomic_write_json(
        &evidence.join("completion.json"),
        &serde_json::json!({
            "schema_version": "bijux.grype_database_acquisition.v1",
            "attempt_id": "fixture-attempt-001",
            "slurm_job_id": "12345",
            "completed_at": "2026-08-08T20:10:35Z",
            "grype_version": "0.116.1",
            "grype_sha256": bijux_dna_infra::hash_file_sha256(&scanner.join("bin/grype"))?,
            "status_sha256": status_sha256,
            "database_manifest_sha256": database_manifest_sha256
        }),
    )?;
    let all_paths = [
        "database/6/import.json",
        "database/6/last_update_check",
        "database/6/vulnerability.db",
        "completion.json",
        "database-manifest.sha256",
        "job.sbatch",
        "providers.txt",
        "status.json",
    ];
    write_manifest(&evidence, "manifest.sha256", &all_paths)?;
    let contract = fixture.path().join("contract.toml");
    fs::write(
        &contract,
        format!(
            r#"schema_version = "bijux.container.grype_database_contract.v1"
acquisition_script_sha256 = "{}"
grype_version = "0.116.1"
grype_sha256 = "{}"
update_url = "https://grype.anchore.io/databases"
schema_major = 6
maximum_age_days = 5
"#,
            bijux_dna_infra::hash_file_sha256(&evidence.join("job.sbatch"))?,
            bijux_dna_infra::hash_file_sha256(&scanner.join("bin/grype"))?,
        ),
    )?;

    let admitted = verify_grype_database(&contract, &scanner, &evidence)?;
    assert_eq!(
        admitted.database_sha256,
        bijux_dna_infra::hash_file_sha256(&evidence.join("database/6/vulnerability.db"))?
    );
    assert!(admitted.status.valid);

    fs::write(evidence.join("database/6/vulnerability.db"), b"mutated")?;
    assert!(verify_grype_database(&contract, &scanner, &evidence)
        .is_err_and(|error| error.to_string().contains("database file digest differs")));
    Ok(())
}

fn write_manifest(root: &Path, name: &str, paths: &[&str]) -> Result<()> {
    let mut rows = paths
        .iter()
        .map(|path| Ok(format!("{}  {path}", bijux_dna_infra::hash_file_sha256(&root.join(path))?)))
        .collect::<Result<Vec<_>>>()?;
    rows.sort();
    fs::write(root.join(name), rows.join("\n") + "\n")?;
    Ok(())
}
