use std::fs;
use std::path::Path;

use walkdir::WalkDir;

#[test]
fn effects_doc_matches_runtime_effect_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let effects_doc = read(root.join("docs/EFFECTS.md"));

    for allowed in [
        "Create directories under declared run/layout roots",
        "Write canonical JSON runtime artifacts",
        "Append runtime-owned JSONL event files",
        "Acquire file locks",
        "Record explicit timestamps",
        "BIJUX_OTEL=1",
    ] {
        assert!(
            effects_doc.contains(allowed),
            "EFFECTS.md must document allowed effect: {allowed}"
        );
    }

    for forbidden in [
        "No process spawning",
        "No Docker or Apptainer invocation",
        "No network access",
        "No CLI parsing",
        "No writes outside declared run-layout or tool-run roots",
    ] {
        assert!(
            effects_doc.contains(forbidden),
            "EFFECTS.md must document forbidden effect: {forbidden}"
        );
    }
}

#[test]
fn runtime_source_does_not_spawn_processes() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    let needles = [
        concat!("std::process::", "Command"),
        concat!("Command::", "new"),
        concat!("tokio::process::", "Command"),
    ];

    for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs")
        {
            continue;
        }
        let content = read(entry.path());
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(entry.path().display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "bijux-dna-runtime must not spawn processes:\n{}",
        offenders.join("\n")
    );
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}
