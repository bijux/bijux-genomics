use std::fs;
use std::path::Path;

fn cargo_lock_dependencies(package: &str) -> Vec<String> {
    let lock_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("Cargo.lock");
    let contents =
        fs::read_to_string(&lock_path).unwrap_or_else(|err| panic!("read Cargo.lock: {err}"));
    let mut deps = Vec::new();
    let mut in_target = false;
    for line in contents.lines() {
        if line.starts_with("[[package]]") {
            in_target = false;
        }
        if line.starts_with("name = ") {
            let name = line.trim_start_matches("name = ").trim_matches('"');
            in_target = name == package;
        }
        if in_target && line.trim_start().starts_with("dependencies = [") {
            let mut dep_block = String::new();
            dep_block.push_str(line);
            if !line.trim_end().ends_with(']') {
                // read until end of dependency array
                continue;
            }
        }
        if in_target && line.trim_start().starts_with('"') {
            let dep = line.trim().trim_matches(',').trim_matches('"').to_string();
            deps.push(dep);
        }
    }
    deps
}

#[test]
fn domain_fastq_does_not_depend_on_engine() {
    let cargo_toml_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let contents =
        fs::read_to_string(&cargo_toml_path).unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));
    assert!(
        !contents.contains("bijux-dna-engine"),
        "bijux-dna-domain-fastq must not depend on bijux-dna-engine directly"
    );

    let deps = cargo_lock_dependencies("bijux-dna-domain-fastq");
    assert!(
        !deps.iter().any(|dep| dep.starts_with("bijux-dna-engine")),
        "bijux-dna-domain-fastq must not depend on bijux-dna-engine indirectly via Cargo.lock"
    );
}
