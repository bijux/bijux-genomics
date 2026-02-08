use std::fs;
use std::path::Path;

#[test]
fn serde_json_value_is_confined_to_load_and_render() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut offenders = Vec::new();
    visit_rs_files(&src_dir, &mut |path| {
        let path_str = path.to_string_lossy();
        if path_str.contains("/load/")
            || path_str.contains("/report")
            || path_str.contains("/model/")
            || path_str.contains("/export")
        {
            return;
        }
        let Ok(contents) = fs::read_to_string(path) else {
            return;
        };
        if contents.contains("serde_json::Value") || contents.contains("serde_json::json!") {
            offenders.push(path_str.to_string());
        }
    });
    assert!(
        offenders.is_empty(),
        "serde_json::Value usage must be confined to load/report/model: {offenders:?}"
    );
}

fn visit_rs_files(dir: &Path, visitor: &mut impl FnMut(&Path)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs_files(&path, visitor);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            visitor(&path);
        }
    }
}
