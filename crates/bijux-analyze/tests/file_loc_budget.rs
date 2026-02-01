mod support;

use std::path::PathBuf;

#[test]
fn file_loc_budget() -> std::io::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let strict = std::env::var("BIJUX_STRICT").is_ok();
    let allowlist = [
        src_dir.join("load").join("sqlite").join("queries.rs"),
        src_dir.join("report").join("bench.rs"),
    ];
    if !allowlist.is_empty() {
        let mut files = Vec::new();
        support::guardrails::collect_rs_files(&src_dir, &mut files)?;
        let hard_limit = 1000usize;
        let soft_limit = 500usize;
        for path in files {
            if allowlist.iter().any(|allowed| allowed == &path) {
                continue;
            }
            let content = std::fs::read_to_string(&path)?;
            let lines = content.lines().count();
            assert!(
                lines <= hard_limit,
                "{} has {} lines (max {})",
                path.display(),
                lines,
                hard_limit
            );
            assert!(
                !(strict && lines > soft_limit),
                "{} has {} lines (strict max {})",
                path.display(),
                lines,
                soft_limit
            );
        }
        return Ok(());
    }
    support::guardrails::assert_loc_budget(&src_dir, strict)
}
