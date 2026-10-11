use super::{syntax::validate_published_commands, ShellSyntaxObservation};
use anyhow::Result;
use std::{fs, path::Path};
#[allow(
    clippy::unnecessary_wraps,
    reason = "Test double implements the fallible syntax inspection port."
)]
fn accepted(_: &Path, _: &Path) -> Result<ShellSyntaxObservation> {
    Ok(ShellSyntaxObservation { exit_code: 0, stderr: String::new() })
}
#[allow(
    clippy::unnecessary_wraps,
    reason = "Test double implements the fallible syntax inspection port."
)]
fn refused(_: &Path, _: &Path) -> Result<ShellSyntaxObservation> {
    Ok(ShellSyntaxObservation { exit_code: 2, stderr: "unclosed syntax".to_owned() })
}
#[test]
fn syntax_failure_is_preserved_before_reading_argv_evidence() -> Result<()> {
    let root = tempfile::tempdir()?;
    let script = root.path().join("script.sh");
    let absent = root.path().join("absent.jsonl");
    let result = validate_published_commands(root.path(), &script, &absent, 1, refused);
    assert!(result
        .as_ref()
        .err()
        .is_some_and(|error| error.to_string().contains("unclosed syntax")));
    assert!(!absent.exists());
    Ok(())
}
#[test]
fn accepted_syntax_still_requires_exact_nonblank_argv_coverage() -> Result<()> {
    let root = tempfile::tempdir()?;
    let script = root.path().join("script.sh");
    let argv = root.path().join("argv.jsonl");
    fs::write(&argv, b"{}\n\n")?;
    assert!(validate_published_commands(root.path(), &script, &argv, 2, accepted).is_err());
    assert!(validate_published_commands(root.path(), &script, &argv, 1, accepted).is_ok());
    fs::write(&argv, b"{}\n{}\n")?;
    assert!(validate_published_commands(root.path(), &script, &argv, 1, accepted).is_err());
    Ok(())
}
