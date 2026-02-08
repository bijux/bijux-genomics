use std::collections::BTreeMap;

use bijux_pipelines::defaults::{merge_overrides, OverrideScope};
use bijux_testkit::snapshot_name;

#[test]
fn override_precedence_is_stable() {
    let mut base = BTreeMap::new();
    base.insert("fastq.trim".to_string(), "fastp".to_string());

    let mut profile = BTreeMap::new();
    profile.insert("fastq.trim".to_string(), "cutadapt".to_string());

    let mut cli = BTreeMap::new();
    cli.insert("fastq.trim".to_string(), "bbduk".to_string());

    let mut forced = BTreeMap::new();
    forced.insert("fastq.trim".to_string(), "trimmomatic".to_string());

    let snapshot = merge_overrides(base, profile, cli, forced, OverrideScope::Stage);
    let name = snapshot_name("contracts", "override_precedence");
    insta::assert_json_snapshot!(name, snapshot);
}
