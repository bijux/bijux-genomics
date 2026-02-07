#[test]
fn hash_is_deterministic() -> anyhow::Result<()> {
    let temp = bijux_infra::temp_dir("bijux-infra-hash")?;
    let path = temp.path().join("data.txt");
    bijux_infra::write_bytes(&path, b"bijux")?;
    let hash1 = bijux_infra::hash_file_sha256(&path)?;
    let hash2 = bijux_infra::hash_file_sha256(&path)?;
    assert_eq!(hash1, hash2);
    Ok(())
}
