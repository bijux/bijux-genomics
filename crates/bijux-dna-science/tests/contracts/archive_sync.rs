use std::path::Path;
use std::process::Command;

#[test]
fn upstream_archive_sync_stays_external_and_preserves_repository_identity() -> anyhow::Result<()> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repository = manifest.join("../..");
    let scratch = repository.join("artifacts/tests/science-archive");
    std::fs::create_dir_all(&scratch)?;
    let output = Command::new("python3")
        .arg("-B")
        .arg(manifest.join("tests/contracts/archive_sync.py"))
        .arg(repository.join("makes/bin/sync_science_docs_github_repos.py"))
        .arg(scratch)
        .output()?;
    assert!(
        output.status.success(),
        "archive sync failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
