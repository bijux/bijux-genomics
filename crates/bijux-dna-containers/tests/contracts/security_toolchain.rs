use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::Result;

#[test]
fn scanner_toolchain_admission_is_manifest_and_binary_bound() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let root = temp.path().join("evidence");
    fs::create_dir_all(root.join("bin"))?;
    fs::create_dir_all(root.join("downloads"))?;
    fs::write(root.join("job.sbatch"), b"governed acquisition")?;

    let tools = [
        ("syft", "1.50.0", "syft.tar.gz", "syft.checksums"),
        ("grype", "0.116.1", "grype.tar.gz", "grype.checksums"),
        ("gitleaks", "8.29.1", "gitleaks.tar.gz", "gitleaks.checksums"),
    ];
    let mut contract_rows = String::new();
    let mut binaries = serde_json::Map::new();
    for (id, version, archive, checksums) in tools {
        let binary = format!("{id}-binary");
        let archive_bytes = format!("{id}-archive");
        let checksum_bytes = format!("{id}-checksums");
        fs::write(root.join("bin").join(id), binary.as_bytes())?;
        fs::write(root.join("downloads").join(archive), archive_bytes.as_bytes())?;
        fs::write(root.join("downloads").join(checksums), checksum_bytes.as_bytes())?;
        fs::write(root.join(format!("{id}-version.txt")), version.as_bytes())?;
        let binary_sha = digest(binary.as_bytes());
        binaries
            .insert(id.to_string(), serde_json::json!({"version": version, "sha256": binary_sha}));
        write!(
            &mut contract_rows,
            "\n[[tools]]\nid = \"{id}\"\nversion = \"{version}\"\narchive = \"{archive}\"\narchive_sha256 = \"{}\"\nchecksums = \"{checksums}\"\nchecksums_sha256 = \"{}\"\nrelease = \"https://github.com/bijux/{id}/releases/tag/v{version}\"\n",
            digest(archive_bytes.as_bytes()),
            digest(checksum_bytes.as_bytes()),
        )?;
    }
    let completion = serde_json::json!({
        "schema_version": "bijux.security_scanner_toolchain_completion.v1",
        "attempt_id": "attempt-001",
        "slurm_job_id": "12345",
        "completed_at": "2026-08-08T19:39:56Z",
        "binaries": binaries,
    });
    fs::write(root.join("completion.json"), serde_json::to_vec_pretty(&completion)?)?;
    let contract_path = temp.path().join("contract.toml");
    fs::write(
        &contract_path,
        format!(
            "schema_version = \"bijux.container.security_scanner_toolchain.v1\"\nacquisition_script_sha256 = \"{}\"\n{contract_rows}",
            digest(b"governed acquisition")
        ),
    )?;
    write_manifest(&root)?;

    let admitted =
        bijux_dna_containers::evidence::verify_security_scanner_toolchain(&contract_path, &root)?;
    assert_eq!(admitted.slurm_job_id, "12345");

    fs::write(root.join("bin/grype"), b"mutated")?;
    assert!(bijux_dna_containers::evidence::verify_security_scanner_toolchain(
        &contract_path,
        &root,
    )
    .is_err_and(|error| error.to_string().contains("binary digest differs")));
    Ok(())
}

fn write_manifest(root: &Path) -> Result<()> {
    let mut paths = vec![
        "completion.json".to_string(),
        "syft-version.txt".to_string(),
        "grype-version.txt".to_string(),
        "gitleaks-version.txt".to_string(),
    ];
    for (id, archive, checksums) in [
        ("syft", "syft.tar.gz", "syft.checksums"),
        ("grype", "grype.tar.gz", "grype.checksums"),
        ("gitleaks", "gitleaks.tar.gz", "gitleaks.checksums"),
    ] {
        paths.extend([
            format!("bin/{id}"),
            format!("downloads/{archive}"),
            format!("downloads/{checksums}"),
        ]);
    }
    paths.sort();
    let rows = paths
        .iter()
        .map(|path| Ok(format!("{}  {path}", digest(&fs::read(root.join(path))?))))
        .collect::<Result<Vec<_>>>()?;
    fs::write(root.join("manifest.sha256"), format!("{}\n", rows.join("\n")))?;
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    bijux_dna_infra::hash_bytes_sha256(bytes)
}
