use std::fs;

#[test]
fn cli_ci_profile_membership_is_bounded() -> anyhow::Result<()> {
    let repo_root = super::support::repo_root()?;
    let crates_dir = repo_root.join("crates");
    let mut counts = ProfileCounts::default();
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
            counts.record_from_source(&content);
        }
    }
    let slow_max = 10usize;
    let science_max = 10usize;
    let e2e_min = 0usize;
    assert!(counts.slow <= slow_max, "slow tests exceed cap: {} > {slow_max}", counts.slow);
    assert!(
        counts.science <= science_max,
        "science tests exceed cap: {} > {science_max}",
        counts.science
    );
    assert!(counts.e2e >= e2e_min, "e2e tests below minimum: {} < {e2e_min}", counts.e2e);
    Ok(())
}

#[derive(Default)]
struct ProfileCounts {
    slow: usize,
    science: usize,
    e2e: usize,
}

impl ProfileCounts {
    fn record_from_source(&mut self, source: &str) {
        for line in source.lines() {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix("fn ") {
                if let Some((name, _)) = rest.split_once('(') {
                    self.record_name(name.trim());
                }
            }
        }
    }

    fn record_name(&mut self, name: &str) {
        if name.contains("slow__") {
            self.slow += 1;
        }
        if name.contains("science__") {
            self.science += 1;
        }
        if name.contains("e2e__") {
            self.e2e += 1;
        }
    }
}
