use std::path::Path;

#[test]
fn randomness_requires_seed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let contents = std::fs::read_to_string(entry.path()).expect("read source");
        let banned = [
            "fastrand::Rng::new",
            "fastrand::Rng::default",
            "fastrand::u64(",
            "fastrand::usize(",
            "fastrand::f64(",
            "rand::random",
            "thread_rng",
            "StdRng::from_entropy",
        ];
        if banned.iter().any(|needle| contents.contains(needle)) {
            offenders.push(entry.path().display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "randomness must be seeded; offenders:\n{}",
        offenders.join("\n")
    );
}
