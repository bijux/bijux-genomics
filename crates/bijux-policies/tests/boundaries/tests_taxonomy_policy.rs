#![allow(non_snake_case)]
#![allow(non_snake_case)]
#[path = "../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__boundaries__tests_taxonomy_policy__taxonomy_buckets_only() {
    let allowed = [
        "contracts",
        "schemas",
        "semantics",
        "determinism",
        "boundaries",
        "fixtures",
        "snapshots",
        "support",
    ];
    let mut offenders = Vec::new();

    for crate_root in support::crate_roots() {
        let tests_root = crate_root.join("tests");
        if !tests_root.exists() {
            continue;
        }
        for entry in WalkDir::new(&tests_root)
            .max_depth(2)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_dir())
        {
            let dir = entry.path();
            if dir == tests_root {
                continue;
            }
            let rel = dir.strip_prefix(&tests_root).unwrap();
            let name = rel
                .components()
                .next()
                .unwrap()
                .as_os_str()
                .to_string_lossy();
            if !allowed.contains(&name.as_ref()) {
                let readme = dir.join("README.md");
                if !readme.exists() {
                    offenders.push(dir.display().to_string());
                }
            }
        }
    }

    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "tests must use canonical buckets or include README.md justification.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
