use cargo_metadata::MetadataCommand;

const BUDGETS: &[(&str, usize)] = &[
    ("bijux-core", 20),
    ("bijux-runtime", 25),
    ("bijux-analyze", 35),
    ("bijux-benchmark-model", 20),
    ("bijux-infra", 20),
    ("bijux-environment-qa", 80),
];

#[test]
fn dependency_budgets() {
    let metadata = MetadataCommand::new()
        .no_deps()
        .exec()
        .expect("cargo metadata");
    let mut offenders = Vec::new();
    for (crate_name, limit) in BUDGETS {
        if let Some(pkg) = metadata.packages.iter().find(|pkg| pkg.name == *crate_name) {
            let count = pkg.dependencies.len();
            if count > *limit {
                offenders.push(format!("{crate_name}: {count} > {limit}"));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "dependency budgets exceeded.\n\
Fix by removing unused deps or feature-gating heavy deps.\n\
See docs/40-policies/STYLE.md for budgeting rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
