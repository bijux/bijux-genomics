use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent().and_then(|p| p.parent()) {
        Some(root) => root.to_path_buf(),
        None => panic!("repo root not found"),
    }
}

fn first_kmer_ref(root: &Path) -> PathBuf {
    let refs = root.join("assets").join("contaminants").join("references");
    let mut entries: Vec<PathBuf> = fs::read_dir(&refs)
        .unwrap_or_else(|_| panic!("missing references dir: {}", refs.display()))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("fasta"))
        .collect();
    entries.sort();
    entries
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("no fasta refs in {}", refs.display()))
}

fn find_metrics_json(root: &Path, sample_id: &str) -> PathBuf {
    let run_dir = root.join("artifacts/bench/filter").join(sample_id);
    let mut stack = vec![run_dir];
    while let Some(dir) = stack.pop() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.file_name().and_then(|s| s.to_str()) == Some("metrics.json") {
                    return path;
                }
            }
        }
    }
    panic!("metrics.json not found under filter run dir");
}

#[test]
#[ignore = "slow e2e (run with make test-slow)"]
fn fastq_filter_e2e_emits_removal_metrics() {
    if std::env::var("BIJUX_E2E").is_err() {
        return;
    }

    let root = repo_root();
    let artifacts = root.join("artifacts");
    let r1 = root.join("tests/data/fastq/ERR769587/ERR769587.fastq.gz");
    let kmer_ref = first_kmer_ref(&root);

    let mut cmd = assert_cmd::cargo_bin_cmd!("bijux");
    cmd.current_dir(&root).args([
        "bench",
        "fastq",
        "filter",
        "--sample-id",
        "ERR769587",
        "--r1",
        r1.to_str()
            .unwrap_or_else(|| panic!("invalid utf-8 path: {r1:?}")),
        "--out",
        artifacts
            .to_str()
            .unwrap_or_else(|| panic!("invalid utf-8 path: {artifacts:?}")),
        "--tools",
        "bbduk",
        "--max-n",
        "1",
        "--low-complexity-threshold",
        "0.6",
        "--kmer-ref",
        kmer_ref
            .to_str()
            .unwrap_or_else(|| panic!("invalid utf-8 path: {kmer_ref:?}")),
    ]);
    cmd.assert().success();

    let metrics_path = find_metrics_json(&root, "ERR769587");
    let raw = match fs::read_to_string(&metrics_path) {
        Ok(value) => value,
        Err(err) => panic!("read metrics failed: {err}"),
    };
    let metrics: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(value) => value,
        Err(err) => panic!("parse metrics failed: {err}"),
    };
    assert!(
        metrics.get("reads_removed_by_n").is_some(),
        "reads_removed_by_n missing"
    );
    assert!(
        metrics.get("reads_removed_low_complexity").is_some(),
        "reads_removed_low_complexity missing"
    );
    assert!(
        metrics.get("reads_removed_contaminant_kmer").is_some(),
        "reads_removed_contaminant_kmer missing"
    );
}
