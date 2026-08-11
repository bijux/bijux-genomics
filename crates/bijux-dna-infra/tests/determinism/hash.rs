#[test]
fn hash_is_deterministic() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("bijux-dna-infra-hash")?;
    let path = temp.path().join("data.txt");
    bijux_dna_infra::write_bytes(&path, b"bijux")?;
    let hash1 = bijux_dna_infra::hash_file_sha256(&path)?;
    let hash2 = bijux_dna_infra::hash_file_sha256(&path)?;
    assert_eq!(hash1, hash2);
    Ok(())
}

#[test]
fn byte_hash_matches_sha256_known_answer() {
    assert_eq!(
        bijux_dna_infra::hash_bytes_sha256(b"bijux\n"),
        "5043e50c2f6db002cc969ca9c644f30665c118d5a3d45dc68ade05b1f982ec69"
    );
}

#[test]
fn path_hash_preserves_file_identity() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("bijux-dna-infra-path-file-hash")?;
    let path = temp.path().join("data.txt");
    bijux_dna_infra::write_bytes(&path, b"bijux-path")?;

    assert_eq!(
        bijux_dna_infra::hash_path_sha256(&path)?,
        bijux_dna_infra::hash_file_sha256(&path)?
    );
    Ok(())
}

#[test]
fn directory_hash_is_order_independent_and_content_sensitive() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("bijux-dna-infra-directory-hash")?;
    let first = temp.path().join("first");
    let second = temp.path().join("second");
    for root in [&first, &second] {
        bijux_dna_infra::ensure_dir(root.join("nested"))?;
    }
    bijux_dna_infra::write_bytes(first.join("a.txt"), b"alpha")?;
    bijux_dna_infra::write_bytes(first.join("nested/b.txt"), b"beta")?;
    bijux_dna_infra::write_bytes(second.join("nested/b.txt"), b"beta")?;
    bijux_dna_infra::write_bytes(second.join("a.txt"), b"alpha")?;

    let expected = bijux_dna_infra::hash_path_sha256(&first)?;
    assert_eq!(bijux_dna_infra::hash_path_sha256(&second)?, expected);

    bijux_dna_infra::write_bytes(second.join("nested/b.txt"), b"changed")?;
    assert_ne!(bijux_dna_infra::hash_path_sha256(&second)?, expected);
    Ok(())
}

#[cfg(unix)]
#[test]
fn directory_hash_records_symlink_identity_without_following_it() -> anyhow::Result<()> {
    let temp = bijux_dna_infra::temp_dir("bijux-dna-infra-directory-symlink-hash")?;
    let root = temp.path().join("database");
    bijux_dna_infra::ensure_dir(root.join("payload"))?;
    bijux_dna_infra::write_bytes(root.join("payload/index.bin"), b"index")?;
    std::os::unix::fs::symlink("payload", root.join("current"))?;

    let first = bijux_dna_infra::hash_path_sha256(&root)?;
    std::fs::remove_file(root.join("current"))?;
    std::os::unix::fs::symlink("payload/index.bin", root.join("current"))?;

    assert_ne!(bijux_dna_infra::hash_path_sha256(&root)?, first);
    Ok(())
}
