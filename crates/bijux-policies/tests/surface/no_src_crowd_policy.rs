#[path = "../support/fs.rs"]
mod support;

const MAX_DIRECT_CHILDREN: usize = 10;
const ALLOWLIST: &[&str] = &["bijux-analyze", "bijux-domain-fastq"];

#[test]
fn src_directory_is_not_overcrowded() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        let crate_name = crate_root.file_name().unwrap().to_string_lossy();
        if ALLOWLIST.contains(&crate_name.as_ref()) {
            continue;
        }
        let src_dir = crate_root.join("src");
        if !src_dir.exists() {
            continue;
        }
        let count = std::fs::read_dir(&src_dir)
            .map(|entries| entries.filter_map(|entry| entry.ok()).count())
            .unwrap_or(0);
        if count > MAX_DIRECT_CHILDREN {
            offenders.push(format!("{} ({} entries)", crate_root.display(), count));
        }
    }

    assert!(
        offenders.is_empty(),
        "src/ must have <= {MAX_DIRECT_CHILDREN} direct children:\n{}",
        offenders.join("\n")
    );
}
