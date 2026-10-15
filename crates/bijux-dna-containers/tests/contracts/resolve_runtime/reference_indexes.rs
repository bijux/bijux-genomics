//! Returned reference indices and reuse checks must agree with actual provider output paths.
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{ensure, Context as _, Result};
use bijux_dna_containers::resolve::{ReferenceBuildRequest, ReferenceRegistry};

const ROOT: &str = "BIJUX_CONTAINER_REFERENCE_FIXTURE";
const CASE: &str = "mod_contracts_resolve_runtime_rs::reference_indexes::prepared_index_paths_and_reuse_follow_provider_outputs";

#[test]
fn prepared_index_paths_and_reuse_follow_provider_outputs() -> Result<()> {
    if let Some(root) = std::env::var_os(ROOT) {
        ensure!(
            std::env::args().skip(1).collect::<Vec<_>>() == [CASE, "--exact", "--nocapture"],
            "exact reference child case"
        );
        let root = PathBuf::from(root);
        ensure!(root.is_absolute() && root.canonicalize()? == root, "private physical fixture");
        return exercise_registry(&root);
    }
    let temporary = tempfile::tempdir()?;
    let root = temporary.path().canonicalize()?;
    let tools = root.join("providers");
    std::fs::create_dir(&tools)?;
    std::fs::write(root.join("reference.fa"), b">reference\nACGT\n")?;
    for (name, script) in [
        ("samtools", "#!/bin/sh\n[ \"$1\" = faidx ] || exit 9\nprintf '%s\\n' \"samtools $1 $2\" >> \"$BIJUX_REFERENCE_LOG\"\nprintf index > \"$2.fai\"\n"),
        ("bwa", "#!/bin/sh\n[ \"$1\" = index ] || exit 9\nprintf '%s\\n' \"bwa $1 $2\" >> \"$BIJUX_REFERENCE_LOG\"\nprintf index > \"$2.bwt\"\n"),
        ("bowtie2-build", "#!/bin/sh\nprintf '%s\\n' \"bowtie2-build $1 $2\" >> \"$BIJUX_REFERENCE_LOG\"\nprintf index > \"$2.1.bt2\"\n"),
    ] {
        let path = tools.join(name);
        std::fs::write(&path, script)?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    let executable = std::env::current_exe()?.canonicalize()?;
    let executable_digest = bijux_dna_infra::hash_file_sha256(&executable)?;
    let mut command = Command::new(&executable);
    command
        .args([CASE, "--exact", "--nocapture"])
        .env(ROOT, &root)
        .env("BIJUX_CACHE_ROOT", root.join("cache"))
        .env("BIJUX_REFERENCE_LOG", root.join("provider-calls.log"))
        .env("PATH", &tools)
        .env("HOME", &root)
        .env("TEST_TMP_DIR", &root)
        .env("TMPDIR", &root)
        .env("TMP", &root)
        .env("TEMP", &root);
    let result = bijux_dna_process::capture(
        &mut command,
        bijux_dna_process::CaptureLimits {
            timeout: Duration::from_secs(20),
            max_output_bytes: 65_536,
        },
        "private reference-index contract",
    )?;
    ensure!(
        !result.timed_out && !result.output_limit_exceeded,
        "bounded reference child: {result:?}"
    );
    ensure!(
        bijux_dna_infra::hash_file_sha256(&executable)? == executable_digest,
        "test executable drift"
    );
    ensure!(
        result.status.success(),
        "reference contract child failed: stdout={} stderr={}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    ensure!(
        String::from_utf8_lossy(&result.stdout).contains("running 1 test"),
        "exact reference case executes"
    );
    Ok(())
}

fn exercise_registry(root: &Path) -> Result<()> {
    let fasta = root.join("reference.fa");
    let original = std::fs::read(&fasta)?;
    let registry = ReferenceRegistry::new();
    let no_indices = registry.prepare_reference(&fasta, &ReferenceBuildRequest::default())?;
    ensure!(std::fs::read(&no_indices.fasta)? == original, "actual no-index copy control");
    ensure!(!root.join("provider-calls.log").exists(), "disabled index flags invoke no providers");
    let request = ReferenceBuildRequest {
        build_fai: true,
        build_dict: false,
        build_bwa_index: true,
        build_bowtie2_index: true,
    };
    let first = registry.prepare_reference(&fasta, &request)?;
    let calls_path = root.join("provider-calls.log");
    let first_calls = std::fs::read_to_string(&calls_path)?;
    let expected_calls = format!(
        "samtools faidx {}\nbwa index {}\nbowtie2-build {} {}\n",
        first.fasta.display(),
        first.fasta.display(),
        first.fasta.display(),
        first.fasta.display()
    );
    ensure!(first_calls == expected_calls, "actual provider argv contract: {first_calls:?}");
    for suffix in [".fai", ".bwt", ".1.bt2"] {
        ensure!(
            PathBuf::from(format!("{}{suffix}", first.fasta.display())).is_file(),
            "provider emitted full FASTA suffix"
        );
    }
    let second = registry.prepare_reference(&fasta, &request)?;
    let second_calls = std::fs::read_to_string(&calls_path)?;
    ensure!(
        std::fs::read(&fasta)? == original && std::fs::read(&first.fasta)? == original,
        "reference input and cache copy preserved"
    );
    ensure!(
        first.fasta == second.fasta && first.root == second.root,
        "same content-addressed reference"
    );
    let returned_fai_exists = first.fai.is_file();
    eprintln!("REFERENCE_PROVIDER_CONTROLS_PASS: flags-off, actual argv and all full-suffix artifacts; returned_fai_exists={returned_fai_exists}; repeated_calls={second_calls:?}");
    let mut failures = Vec::new();
    if !returned_fai_exists {
        failures.push("returned FAI path");
    }
    if second_calls != first_calls {
        failures.push("BWA/Bowtie2/FAI index reuse");
    }
    ensure!(
        failures.is_empty(),
        "reference index paths disagree with provider outputs: {failures:?}"
    );
    ensure!(first.fai == second.fai, "stable returned index identity");
    std::fs::read(&first.fai).context("returned index must be readable")?;
    Ok(())
}
