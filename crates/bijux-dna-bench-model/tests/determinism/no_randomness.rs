use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[test]
fn randomness_requires_seed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rs_files(&root, &mut files);
    let mut offenders = Vec::new();
    for path in files {
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read source {}: {err}", path.display()));
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
            offenders.push(path.display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "randomness must be seeded; offenders:\n{}",
        offenders.join("\n")
    );
}
