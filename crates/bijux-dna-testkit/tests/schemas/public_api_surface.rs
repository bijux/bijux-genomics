use std::fs;
use std::path::Path;

#[cfg(unix)]
#[test]
fn workspace_binary_bindings_validate_before_execution() -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    if std::env::var_os("BIJUX_TEST_EXECUTABLE_RESULT").is_some() {
        return workspace_binary_binding_child();
    }

    let directory = bijux_dna_testkit::tempdir_for("workspace binary binding");
    let selected = directory.path().join("bijux-dna");
    let fallback = directory.path().join("compiled-bijux-dna");
    let nonexecutable = directory.path().join("nonexecutable");
    for path in [&selected, &fallback, &nonexecutable] {
        fs::write(path, "#!/bin/sh\nprintf '%s' \"$1\"\n")?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    fs::set_permissions(&nonexecutable, fs::Permissions::from_mode(0o600))?;
    let missing = directory.path().join("missing");
    let relative = Path::new("relative/bijux-dna");
    let cases = [
        (Some(selected.as_path()), fallback.as_path(), "ok", selected.as_path()),
        (None, fallback.as_path(), "ok", fallback.as_path()),
        (Some(missing.as_path()), fallback.as_path(), "NotFound", missing.as_path()),
        (Some(relative), fallback.as_path(), "InvalidInput", relative),
        (Some(Path::new("")), fallback.as_path(), "InvalidInput", Path::new("")),
        (
            Some(nonexecutable.as_path()),
            fallback.as_path(),
            "PermissionDenied",
            nonexecutable.as_path(),
        ),
        (Some(directory.path()), fallback.as_path(), "InvalidInput", directory.path()),
        (None, missing.as_path(), "NotFound", missing.as_path()),
    ];
    for (binding, compiled_path, result, expected) in cases {
        let mut child = Command::new(std::env::current_exe()?);
        child
            .args([
                "workspace_binary_bindings_validate_before_execution",
                "--nocapture",
                "--test-threads=1",
            ])
            .current_dir(directory.path())
            .env("BIJUX_TEST_COMPILED_EXECUTABLE", compiled_path)
            .env("BIJUX_TEST_EXPECTED_EXECUTABLE", expected)
            .env("BIJUX_TEST_EXECUTABLE_RESULT", result);
        if let Some(binding) = binding {
            child.env("NEXTEST_BIN_EXE_bijux-dna", binding);
        } else {
            child.env_remove("NEXTEST_BIN_EXE_bijux-dna");
        }
        let output = child.output()?;
        assert!(
            output.status.success(),
            "binding {binding:?}, expected {result}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    Ok(())
}

#[cfg(unix)]
fn workspace_binary_binding_child() -> std::io::Result<()> {
    let Ok(result) = std::env::var("BIJUX_TEST_EXECUTABLE_RESULT") else {
        return Ok(());
    };
    let compiled_path = std::env::var_os("BIJUX_TEST_COMPILED_EXECUTABLE").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "missing compiled executable fixture path",
        )
    })?;
    let resolved = bijux_dna_testkit::workspace_support::workspace_binary_path(
        "bijux-dna",
        Path::new(&compiled_path),
    );
    if result != "ok" {
        let error = match resolved {
            Err(error) => error,
            Ok(path) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("invalid binding accepted: {}", path.display()),
                ))
            }
        };
        assert_eq!(format!("{:?}", error.kind()), result);
        return Ok(());
    }
    let expected = std::env::var_os("BIJUX_TEST_EXPECTED_EXECUTABLE").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "missing expected executable fixture path",
        )
    })?;
    let executable = resolved?;
    assert_eq!(executable, Path::new(&expected));
    let output =
        std::process::Command::new(&executable).arg("bound executable arguments").output()?;
    assert!(output.status.success());
    assert_eq!(output.stdout, b"bound executable arguments");
    Ok(())
}
