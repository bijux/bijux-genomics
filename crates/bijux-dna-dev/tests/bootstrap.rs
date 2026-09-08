#![cfg(unix)]

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Workspace {
    directory: tempfile::TempDir,
    bootstrap: PathBuf,
}

impl Workspace {
    fn create() -> Result<Self, Box<dyn Error>> {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let scratch = repository.join("artifacts/tests/bootstrap");
        fs::create_dir_all(&scratch)?;
        let directory = tempfile::Builder::new().prefix("dependency-").tempdir_in(scratch)?;
        let root = directory.path();
        fs::create_dir_all(root.join("crates/bijux-dna-dev/src"))?;
        fs::create_dir_all(root.join("crates/domain/src"))?;
        fs::create_dir_all(root.join(".cargo"))?;
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nmembers = [\"crates/bijux-dna-dev\", \"crates/domain\"]\nresolver = \"2\"\n",
        )?;
        fs::write(root.join(".cargo/config.toml"), "[build]\ntarget-dir = \"other-target\"\n")?;
        fs::write(
            root.join("crates/bijux-dna-dev/Cargo.toml"),
            "[package]\nname = \"bijux-dna-dev\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[dependencies]\ndomain = { path = \"../domain\" }\n",
        )?;
        fs::write(
            root.join("crates/bijux-dna-dev/src/main.rs"),
            "fn main() { println!(\"{}\", domain::value()); }\n",
        )?;
        fs::write(
            root.join("crates/domain/Cargo.toml"),
            "[package]\nname = \"domain\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )?;
        fs::write(root.join("crates/domain/src/lib.rs"), "pub fn value() -> u8 { 1 }\n")?;
        let lock = Command::new("cargo")
            .args(["generate-lockfile", "--offline"])
            .current_dir(root)
            .output()?;
        assert!(lock.status.success(), "{}", String::from_utf8_lossy(&lock.stderr));
        Ok(Self { directory, bootstrap: repository.join("makes/bin/dev_dna_bootstrap.sh") })
    }

    fn run(&self, binary: &Path) -> Result<Output, Box<dyn Error>> {
        Ok(Command::new("sh")
            .arg(&self.bootstrap)
            .arg(binary)
            .current_dir(self.directory.path())
            .env_remove("CARGO_BUILD_TARGET")
            .env_remove("RUSTC_WRAPPER")
            .output()?)
    }
}

#[test]
fn bootstrap_rebuilds_transitive_sources_and_rejects_stale_success() -> Result<(), Box<dyn Error>> {
    let workspace = Workspace::create()?;
    let binary = workspace.directory.path().join("compiled artifacts/debug/bijux-dna-dev");
    let first = workspace.run(&binary)?;
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    assert_eq!(Command::new(&binary).output()?.stdout, b"1\n");

    let source = workspace.directory.path().join("crates/domain/src/lib.rs");
    fs::write(&source, "pub fn value() -> u8 { 2 }\n")?;
    let second = workspace.run(&binary)?;
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    assert_eq!(Command::new(&binary).output()?.stdout, b"2\n");

    let modified = fs::metadata(&binary)?.modified()?;
    assert!(workspace.run(&binary)?.status.success());
    assert_eq!(fs::metadata(&binary)?.modified()?, modified, "no-op bootstrap should reuse output");

    fs::write(source, "this is not valid Rust\n")?;
    assert!(
        !workspace.run(&binary)?.status.success(),
        "an old executable cannot hide build failure"
    );
    Ok(())
}

#[test]
fn bootstrap_rejects_an_unrelated_output_path() -> Result<(), Box<dyn Error>> {
    let workspace = Workspace::create()?;
    let output = workspace.run(&workspace.directory.path().join("unrelated"))?;
    assert_eq!(output.status.code(), Some(2));
    assert!(!workspace.directory.path().join("other-target").exists());
    Ok(())
}
