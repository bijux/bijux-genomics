#![allow(non_snake_case)]

#[path = "../../support/fs.rs"]
mod support;

use support::workspace_root;

#[test]
fn policy__contracts__examples_golden_hygiene_policy__goldens_are_redacted_and_stamped() {
    let root = workspace_root().join("examples");
    let mut offenders = Vec::new();

    let entries = std::fs::read_dir(&root).expect("read examples/");
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !name.starts_with("example-") {
            continue;
        }
        let suffix = &name["example-".len()..];
        if suffix.len() != 3 || !suffix.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        let golden_dir = path.join("golden");
        if !golden_dir.exists() {
            offenders.push(format!("{}: missing golden/ directory", path.display()));
            continue;
        }

        let stamp_path = golden_dir.join("provenance_stamp.json");
        if !stamp_path.exists() {
            offenders.push(format!(
                "{}: missing {}",
                path.display(),
                stamp_path.display()
            ));
        } else if let Ok(raw) = std::fs::read_to_string(&stamp_path) {
            let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap_or_default();
            let has_commit = parsed
                .get("commit")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|v| !v.trim().is_empty());
            let has_registry_hash = parsed
                .get("registry_hash")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|v| v.starts_with("sha256:"));
            if !has_commit || !has_registry_hash {
                offenders.push(format!(
                    "{}: invalid provenance stamp {}",
                    path.display(),
                    stamp_path.display()
                ));
            }
        }

        for golden in std::fs::read_dir(&golden_dir)
            .into_iter()
            .flatten()
            .flatten()
        {
            let p = golden.path();
            if p.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let raw = std::fs::read_to_string(&p).unwrap_or_default();
            if raw.contains("/home/") || raw.contains("/Users/") {
                offenders.push(format!(
                    "{}: contains absolute host path literal",
                    p.display()
                ));
            }
            if raw.contains("http://") || raw.contains("https://") {
                offenders.push(format!(
                    "{}: contains absolute URL/hostname literal",
                    p.display()
                ));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "example golden hygiene policy violations:\n{}",
        offenders.join("\n")
    );
}
