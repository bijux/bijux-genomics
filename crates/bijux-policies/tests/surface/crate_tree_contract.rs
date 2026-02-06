#[path = "../support/fs.rs"]
mod support;

use std::fmt::Write;

#[test]
fn crate_tree_contract_snapshot() {
    let mut output = String::new();
    for crate_root in support::crate_roots() {
        let crate_name = crate_root.file_name().unwrap().to_string_lossy();
        writeln!(&mut output, "[{crate_name}]").expect("write");
        let mut entries: Vec<String> = std::fs::read_dir(&crate_root)
            .expect("read crate root")
            .filter_map(|entry| entry.ok())
            .map(|entry| {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if path.is_dir() {
                    format!("{name}/")
                } else {
                    name
                }
            })
            .filter(|name| !name.starts_with('.'))
            .collect();
        entries.sort();
        for entry in entries {
            writeln!(&mut output, "- {entry}").expect("write");
        }
        output.push('\n');
    }

    insta::assert_snapshot!("crate_tree_contract", output);
}
