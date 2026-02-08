use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn support_helpers_are_named_by_purpose() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("support");
    let allowed: BTreeSet<&str> = BTreeSet::from([
        "mod.rs",
        "README.md",
        "execution_setup.rs",
        "manifest_fixture.rs",
        "plan_factory.rs",
        "runner_stub.rs",
    ]);
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&root).expect("read tests/support") {
        let entry = entry.expect("support entry");
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "helpers.rs" {
            offenders.push(name);
            continue;
        }
        if entry.path().is_file() && !allowed.contains(name.as_str()) {
            offenders.push(name);
        }
    }
    assert!(
        offenders.is_empty(),
        "support helpers must be purpose-named and never helpers.rs: {offenders:?}"
    );
}
