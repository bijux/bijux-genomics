use std::path::{Path, PathBuf};

use walkdir::WalkDir;

#[test]
fn production_code_keeps_effects_inside_stage_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let forbidden_everywhere = [
        "std::process",
        "Command::",
        "tokio::process",
        "TcpStream",
        "UdpSocket",
        "reqwest",
        "bijux_dna_engine::",
        "bijux_dna_runner::",
        "bijux_dna_environment::",
    ];
    let write_effects = [
        "std::fs::write",
        "write_bytes",
        "atomic_write_json",
        "create_dir",
        "remove_file",
        "remove_dir",
    ];
    let write_allow_path = root.join("src/observer/artifacts.rs");
    let mut offenders = Vec::new();

    for path in rust_source_files(&root.join("src")) {
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        let production_content = strip_cfg_test_items(&content);
        for needle in forbidden_everywhere {
            if production_content.contains(needle) {
                offenders.push(format!("{} contains `{needle}`", path.display()));
            }
        }
        if path != write_allow_path {
            for needle in write_effects {
                if production_content.contains(needle) {
                    offenders.push(format!("{} contains write effect `{needle}`", path.display()));
                }
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "stages-fastq production effects must stay inside the documented boundary:\n{}",
        offenders.join("\n")
    );
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("rs"))
        .filter(|path| {
            path.file_name().and_then(|name| name.to_str()) != Some("plugin_contracts.rs")
        })
        .collect()
}

fn strip_cfg_test_items(source: &str) -> String {
    let mut output = Vec::new();
    let mut pending_cfg_test = false;
    let mut skipped_brace_depth = 0usize;

    for line in source.lines() {
        if skipped_brace_depth > 0 {
            skipped_brace_depth += line.matches('{').count();
            skipped_brace_depth = skipped_brace_depth.saturating_sub(line.matches('}').count());
            continue;
        }

        let trimmed = line.trim_start();
        if trimmed == "#[cfg(test)]" {
            pending_cfg_test = true;
            continue;
        }

        if pending_cfg_test {
            skipped_brace_depth += line.matches('{').count();
            skipped_brace_depth = skipped_brace_depth.saturating_sub(line.matches('}').count());
            pending_cfg_test = false;
            continue;
        }

        output.push(line);
    }

    output.join("\n")
}
