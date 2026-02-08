use std::fs;
use std::path::PathBuf;

#[test]
fn cli_ci_profile_membership_is_bounded() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| anyhow::anyhow!("repo root not found"))?;
    let crates_dir = repo_root.join("crates");
    let mut slow = 0usize;
    let mut science = 0usize;
    let mut e2e = 0usize;
    for entry in fs::read_dir(crates_dir)? {
        let entry = entry?;
        let tests_dir = entry.path().join("tests");
        if !tests_dir.exists() {
            continue;
        }
        for file in fs::read_dir(&tests_dir)? {
            let file = file?;
            let path = file.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let content = fs::read_to_string(&path)?;
            slow += content.matches("_slow_").count();
            science += content.matches("_science_").count();
            e2e += content.matches("_e2e_").count();
        }
    }
    let slow_max = 10usize;
    let science_max = 10usize;
    let e2e_min = 0usize;
    assert!(
        slow <= slow_max,
        "slow tests exceed cap: {slow} > {slow_max}"
    );
    assert!(
        science <= science_max,
        "science tests exceed cap: {science} > {science_max}"
    );
    assert!(e2e >= e2e_min, "e2e tests below minimum: {e2e} < {e2e_min}");
    Ok(())
}
