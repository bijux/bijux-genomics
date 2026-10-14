use std::fs;
use std::path::Path;

fn collect_rs_files(root: &Path, files: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let path = entry.path();
        if file_type.is_symlink() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("CLI source entry must not redirect: {}", path.display()),
            ));
        }
        if file_type.is_dir() {
            collect_rs_files(&path, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn production_process_offenders(src: &Path) -> std::io::Result<Vec<String>> {
    let mut files = Vec::new();
    collect_rs_files(src, &mut files)?;
    if files.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("CLI source contains no Rust files: {}", src.display()),
        ));
    }
    files.sort();
    let mut offenders = Vec::new();
    let needles = [
        concat!("std::process::", "Command"),
        concat!("Command::", "new("),
        concat!("tokio::process::", "Command"),
    ];
    for file in files {
        let raw = fs::read_to_string(&file)?;
        let contents = bijux_dna_testkit::workspace_support::strip_rust_test_modules(&raw);
        for needle in &needles {
            if contents.contains(needle) {
                offenders.push(format!("{} contains {}", file.display(), needle));
            }
        }
    }
    Ok(offenders)
}

#[test]
fn cli_does_not_spawn_processes() {
    let src = super::support::crate_src("bijux-dna")
        .unwrap_or_else(|err| panic!("resolve bijux-dna src: {err}"));
    let offenders = production_process_offenders(&src)
        .unwrap_or_else(|err| panic!("scan CLI production source {}: {err}", src.display()));
    assert!(
        offenders.is_empty(),
        "cli must not spawn processes directly:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn cli_process_scan_preserves_production_refusal_and_test_gating() -> std::io::Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("CLI production source classification");
    let constructor = "pub fn entry() { let _ = std::process::Command::new(\"true\"); }";
    fs::write(owner.path().join("production.rs"), constructor)?;
    fs::write(owner.path().join("test_support.rs"), format!("#![cfg(test)]\n{constructor}"))?;
    fs::write(
        owner.path().join("inline.rs"),
        format!("pub fn entry() {{}}\n#[cfg(test)]\nmod tests {{ {constructor} }}"),
    )?;
    fs::write(
        owner.path().join("literal.rs"),
        format!("const EXAMPLE: &str = \"#![cfg(test)]\";\n{constructor}"),
    )?;
    let offenders = production_process_offenders(owner.path())?;
    assert_eq!(offenders.len(), 4);
    assert!(offenders
        .iter()
        .all(|message| { message.contains("production.rs") || message.contains("literal.rs") }));
    for name in ["production.rs", "literal.rs"] {
        assert_eq!(offenders.iter().filter(|message| message.contains(name)).count(), 2);
    }
    Ok(())
}

#[test]
fn cli_process_scan_refuses_missing_empty_and_unreadable_source() -> std::io::Result<()> {
    let owner = bijux_dna_testkit::tempdir_for("CLI source admission");
    assert!(production_process_offenders(&owner.path().join("missing")).is_err());
    assert!(production_process_offenders(owner.path()).is_err());
    fs::write(owner.path().join("invalid.rs"), [0xff])?;
    assert!(production_process_offenders(owner.path()).is_err());
    Ok(())
}
