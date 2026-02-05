use std::path::Path;

use walkdir::WalkDir;

fn stage_files(root: &Path) -> Vec<std::path::PathBuf> {
    WalkDir::new(root.join("src"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("rs"))
        .map(walkdir::DirEntry::into_path)
        .collect()
}

#[test]
fn stages_do_not_compose_pipelines() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = stage_files(crate_root);
    let forbidden = [
        "PipelineSpec",
        "plan_preprocess_pipeline",
        "compose_fastq_pipeline",
        "compose_pipeline",
        "pipeline_spec",
    ];
    for path in files {
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read stage file {}: {err}", path.display()));
        assert!(
            !forbidden.iter().any(|needle| content.contains(needle)),
            "stage crate must not compose pipelines; forbidden token found in {}",
            path.display()
        );
    }
}
