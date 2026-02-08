#![allow(non_snake_case)]
use crate::support::{crate_roots, read_to_string};

const MAX_ARCHITECTURE_LINES: usize = 40;

#[test]
fn policy__surface__architecture_pointer_policy__architecture_docs_are_brief_pointers() {
    let mut offenders = Vec::new();

    for crate_root in crate_roots() {
        let doc = crate_root.join("docs").join("ARCHITECTURE.md");
        if !doc.exists() {
            continue;
        }
        let content = read_to_string(&doc);
        let line_count = content.lines().count();
        if line_count > MAX_ARCHITECTURE_LINES {
            offenders.push(format!("{} ({} lines)", doc.display(), line_count));
        }
    }

    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "docs/ARCHITECTURE.md must remain a short pointer, not a duplicate essay.\nOffenders:\n{}",
        offenders.join("\n")
    );
}
