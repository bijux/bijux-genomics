use walkdir::WalkDir;

#[test]
fn args_module_uses_explicit_contract_names() {
    let root = crate::support::crate_src("bijux-dna-api")
        .unwrap_or_else(|err| panic!("resolve crate src: {err}"));
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&root) {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().file_name().and_then(|n| n.to_str()) == Some("args.rs") {
            offenders.push(entry.path().display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "args.rs is forbidden; use explicit names such as request_contracts.rs.\nOffenders:\n{}",
        offenders.join("\n")
    );
}
