use bijux_core::foundation::hashing::run_id_from_hashes;

#[test]
fn run_id_is_deterministic_for_same_inputs() {
    let input_hashes = vec!["sha256:a".to_string(), "sha256:b".to_string()];
    let run_id1 = run_id_from_hashes("pipeline", "sample", "params", &input_hashes, None);
    let run_id2 = run_id_from_hashes("pipeline", "sample", "params", &input_hashes, None);
    assert_eq!(run_id1, run_id2);
}
