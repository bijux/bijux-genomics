#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

fn stage_ids_from(path: &std::path::Path) -> Vec<String> {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("read stage config {}", path.display()));
    let doc: toml::Value = raw
        .parse()
        .unwrap_or_else(|_| panic!("parse stage config {}", path.display()));
    doc.get("stages")
        .and_then(toml::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(toml::Value::as_str))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[test]
fn policy__contracts__stage_id_symmetry_policy__ids_follow_domain_dot_verb_pattern() {
    let root = support::workspace_root();
    let pattern =
        regex::Regex::new(r"^(fastq|bam|vcf)\.[a-z0-9]+(?:_[a-z0-9]+)*$").expect("compile regex");
    let mut offenders = Vec::new();
    for path in [
        root.join("configs/ci/stages/stages.toml"),
        root.join("configs/ci/stages/stages_vcf.toml"),
    ] {
        for stage_id in stage_ids_from(&path) {
            if !pattern.is_match(&stage_id) {
                offenders.push(format!("{} invalid stage id `{stage_id}`", path.display()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "stage id naming policy violations:\n{}",
        offenders.join("\n")
    );
}
