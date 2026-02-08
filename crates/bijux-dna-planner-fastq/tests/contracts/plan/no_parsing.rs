use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, out);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
}

#[test]
fn planner_does_not_use_observer_parsing() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_root = crate_root.join("src");
    let mut files = Vec::new();
    collect_rs_files(&src_root, &mut files);

    let forbidden = [
        "bijux_dna_stages_fastq::observer",
        "bijux_dna_stages_fastq::observer::",
        "observer::parse_",
    ];

    for path in files {
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {path:?}: {err}"));
        for needle in &forbidden {
            assert!(
                !content.contains(needle),
                "planner must not use observer parsing APIs (found {needle} in {path:?})"
            );
        }
    }
}
