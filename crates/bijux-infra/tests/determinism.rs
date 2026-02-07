#[test]
fn hash_is_deterministic() {
    let data = b"bijux";
    let hash1 = bijux_infra::hash_bytes_sha256(data);
    let hash2 = bijux_infra::hash_bytes_sha256(data);
    assert_eq!(hash1, hash2);
}
