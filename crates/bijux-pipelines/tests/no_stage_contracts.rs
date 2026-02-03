use std::fs;
use std::path::PathBuf;

#[test]
fn pipelines_do_not_define_stage_contracts() -> anyhow::Result<()> {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in walkdir::WalkDir::new(&src_dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content = fs::read_to_string(entry.path())?;
        assert!(
            !content.contains("struct StageContract") && !content.contains("enum StageContract"),
            "pipeline must not define stage contracts ({})",
            entry.path().display()
        );
    }
    Ok(())
}
