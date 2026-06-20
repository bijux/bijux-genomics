use std::collections::BTreeSet;
use std::fs;

use walkdir::WalkDir;

#[test]
fn cli_ci_profile_membership_is_bounded() -> anyhow::Result<()> {
    let repo_root = super::support::repo_root()?;
    let crates_dir = repo_root.join("crates");
    let mut counts = ProfileCounts::default();
    for entry in fs::read_dir(crates_dir)? {
        let entry = entry?;
        for source_root in [entry.path().join("src"), entry.path().join("tests")] {
            if !source_root.exists() {
                continue;
            }
            for file in WalkDir::new(&source_root).into_iter().filter_map(Result::ok) {
                let path = file.path();
                if !file.file_type().is_file()
                    || path.extension().and_then(|s| s.to_str()) != Some("rs")
                {
                    continue;
                }
                let content = fs::read_to_string(path)?;
                counts.record_from_source(&content);
            }
        }
    }
    let slow_roster_path = repo_root.join("configs/rust/nextest-slow-roster.txt");
    let slow_roster = load_slow_roster(&slow_roster_path)?;
    let science_max = 10usize;
    let e2e_min = 0usize;
    assert_eq!(
        slow_roster,
        {
            let mut sorted = slow_roster.clone();
            sorted.sort();
            sorted.dedup();
            sorted
        },
        "slow roster must stay sorted and unique: {}",
        slow_roster_path.display()
    );
    let missing =
        slow_roster.iter().filter(|name| !counts.all.contains(*name)).cloned().collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "slow roster entries must map to governed test names: {:?}",
        missing
    );
    let duplicated = slow_roster
        .iter()
        .filter(|name| counts.named_slow.contains(*name))
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        duplicated.is_empty(),
        "slow roster must not duplicate slow__-prefixed tests: {:?}",
        duplicated
    );
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
    all: BTreeSet<String>,
    named_slow: BTreeSet<String>,
    science: usize,
    e2e: usize,
}

impl ProfileCounts {
    fn record_from_source(&mut self, source: &str) {
        let mut pending_test_attr = false;
        for line in source.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("#[") {
                if trimmed.contains("test") {
                    pending_test_attr = true;
                }
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("fn ") {
                if pending_test_attr {
                    if let Some((name, _)) = rest.split_once('(') {
                        self.record_name(name.trim());
                    }
                }
                pending_test_attr = false;
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("async fn ") {
                if pending_test_attr {
                    if let Some((name, _)) = rest.split_once('(') {
                        self.record_name(name.trim());
                    }
                }
                pending_test_attr = false;
                continue;
            }
            if !trimmed.is_empty() {
                pending_test_attr = false;
            }
        }
    }

    fn record_name(&mut self, name: &str) {
        self.all.insert(name.to_string());
        if name.contains("slow__") {
            self.named_slow.insert(name.to_string());
        }
        if name.contains("science__") {
            self.science += 1;
        }
        if name.contains("e2e__") {
            self.e2e += 1;
        }
    }
}

fn load_slow_roster(path: &std::path::Path) -> anyhow::Result<Vec<String>> {
    let content = fs::read_to_string(path)?;
    Ok(content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect())
}
