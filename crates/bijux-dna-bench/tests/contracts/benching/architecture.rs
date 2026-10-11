use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn collect_rs_files(root: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
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

fn assert_no_imports(dir: &str, forbidden: &[&str]) {
    let mut files = Vec::new();
    collect_rs_files(Path::new(dir), &mut files);
    for file in files {
        let Ok(contents) = fs::read_to_string(&file) else {
            continue;
        };
        for needle in forbidden {
            assert!(
                !contents.contains(needle),
                "forbidden import in {}: {}",
                file.display(),
                needle
            );
        }
    }
}

#[test]
fn bench_does_not_import_selection_or_reports() {
    assert_no_imports(
        "crates/bijux-dna-bench/src",
        &["bijux_selection", "bijux_dna_analyze::report"],
    );
}

// The local simulation repository owns its disposable destination. No publication
// authority may consume that destination, and this owner may not adopt other roots.
fn owns_disposable_destination(source: &Path, destination: &str) -> bool {
    source == Path::new("crates/bijux-dna-bench/src/repo/simulation/mod.rs")
        && destination == "artifacts/benchmark-smoke/simulated-runs"
}

#[test]
fn disposable_destination_ownership_does_not_exempt_adjacent_authorities() {
    let owner = Path::new("crates/bijux-dna-bench/src/repo/simulation/mod.rs");
    let destination = "artifacts/benchmark-smoke/simulated-runs";
    assert!(owns_disposable_destination(owner, destination));
    assert!(!owns_disposable_destination(owner, "assets/toy"));
    for source in [
        "crates/bijux-dna-bench/src/repo/hpc_campaign/copy_back.rs",
        "crates/bijux-dna-bench/src/workflow/simulation/mod.rs",
        "crates/bijux-dna-runner/src/backend/bundle/mod.rs",
        "benchmarks/corpora/simulation.json",
    ] {
        assert!(!owns_disposable_destination(Path::new(source), destination));
    }
}

#[test]
fn publishable_benchmark_authorities_exclude_disposable_data_roots() -> Result<()> {
    const AUTHORITY_ROOTS: &[&str] = &[
        "crates/bijux-dna-bench/src",
        "crates/bijux-dna-runner/src",
        "crates/bijux-dna-runtime/src",
        "benchmarks/contracts",
        "benchmarks/corpora",
        "configs/hpc",
    ];
    const DISPOSABLE_ROOTS: &[&str] = &[
        "assets/toy",
        "artifacts/runs/bench/local-fake-runs",
        "artifacts/runs/bench/readiness-probes",
        "artifacts/benchmark-smoke/readiness",
        "artifacts/benchmark-smoke/simulated-runs",
    ];

    let repository_root = crate::support::repo_root()?;
    let mut violations = Vec::new();
    for relative_root in AUTHORITY_ROOTS {
        let authority_root = repository_root.join(relative_root);
        for entry in WalkDir::new(&authority_root).sort_by_file_name() {
            let entry = entry.with_context(|| format!("walk {}", authority_root.display()))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let Ok(contents) = fs::read_to_string(entry.path()) else {
                continue;
            };
            let contents = bijux_dna_testkit::workspace_support::strip_rust_test_modules(&contents);
            for disposable_root in DISPOSABLE_ROOTS {
                if contents.contains(disposable_root)
                    && !owns_disposable_destination(
                        entry.path().strip_prefix(&repository_root)?,
                        disposable_root,
                    )
                {
                    violations.push(format!(
                        "{} references {disposable_root}",
                        entry.path().strip_prefix(&repository_root)?.display()
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "publishable benchmark authorities reference disposable data roots:\n{}",
        violations.join("\n")
    );
    Ok(())
}
