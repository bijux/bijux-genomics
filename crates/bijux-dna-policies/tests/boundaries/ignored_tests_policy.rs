#![allow(non_snake_case)]
#[path = "../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__boundaries__ignored_tests_policy__ignored_tests_documented() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        let tests_root = crate_root.join("tests");
        if !tests_root.exists() {
            continue;
        }
        for entry in WalkDir::new(&tests_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("rs"))
        {
            let content = support::read_to_string(entry.path());
            if content.contains("#[ignore]") {
                let has_reason = content.contains("// ignore:") || content.contains("/// ignore:");
                if !has_reason {
                    offenders.push(entry.path().display().to_string());
                }
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "ignored tests must include a comment explaining why.\n\
Add `// ignore: <reason>` near the ignored test.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
