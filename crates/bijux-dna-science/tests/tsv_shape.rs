use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("repo root")
        .to_path_buf()
}

fn collect_directory_tsvs(dir: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read TSV directory") {
        let path = entry.expect("read TSV entry").path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("tsv") {
            paths.push(path);
        }
    }
}

fn assert_rectangular_tsv(path: &Path) {
    let text = fs::read_to_string(path).expect("read TSV");
    let mut expected_columns = None;

    for (index, line) in text.lines().enumerate() {
        assert!(
            !line.starts_with('#'),
            "{}:{} contains a comment line in a governed TSV",
            path.display(),
            index + 1
        );

        let columns = line.split('\t').count();
        match expected_columns {
            Some(expected) => assert_eq!(
                columns,
                expected,
                "{}:{} has {columns} columns, expected {expected}",
                path.display(),
                index + 1
            ),
            None => expected_columns = Some(columns),
        }
    }

    assert!(expected_columns.is_some(), "{} must not be empty", path.display());
}

#[test]
fn governed_science_tsvs_are_rectangular() {
    let root = repo_root();
    let mut paths = Vec::new();

    collect_directory_tsvs(&root.join("science/generated/current/evidence"), &mut paths);
    collect_directory_tsvs(&root.join("science-docs/upstream/fastq"), &mut paths);
    collect_directory_tsvs(&root.join("science-docs/upstream/fastq/tools"), &mut paths);
    collect_directory_tsvs(&root.join("science-docs/upstream/papers"), &mut paths);
    paths.push(root.join("science-docs/upstream/github-repos/MANIFEST.tsv"));
    paths.sort();

    assert!(!paths.is_empty(), "expected governed science TSV files");
    for path in paths {
        assert_rectangular_tsv(&path);
    }
}
