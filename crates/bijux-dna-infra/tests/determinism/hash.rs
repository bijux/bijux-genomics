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
