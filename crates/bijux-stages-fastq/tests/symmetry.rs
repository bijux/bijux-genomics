use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn module_set(crate_name: &str) -> BTreeSet<String> {
    let src_dir = workspace_root().join(format!("crates/{crate_name}/src"));
    let mut modules = BTreeSet::new();
    for entry in std::fs::read_dir(&src_dir).expect("read src dir") {
        let entry = entry.expect("read entry");
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            modules.insert(name);
        } else if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if stem != "lib" {
                modules.insert(stem.to_string());
            }
        }
    }
    modules
}

#[test]
fn fastq_bam_stage_module_symmetry() {
    let expected: BTreeSet<String> = [
        "metrics".to_string(),
        "observer".to_string(),
        "plugin".to_string(),
        "stage_specs".to_string(),
    ]
    .into_iter()
    .collect();

    let fastq = module_set("bijux-stages-fastq");
    let bam = module_set("bijux-stages-bam");

    assert_eq!(fastq, expected, "fastq stage modules must match expected set");
    assert_eq!(bam, expected, "bam stage modules must match expected set");
}
