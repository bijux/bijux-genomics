#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn is_allowed_writer_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/crates/bijux-dna-runtime/")
        || path_str.contains("/crates/bijux-dna-engine/")
        || path_str.contains("/crates/bijux-dna-infra/")
        // Developer process logs and immutable source materialization are runtime effects.
        || path_str.ends_with("/crates/bijux-dna-dev/src/runtime/process.rs")
        || path_str.ends_with("/crates/bijux-dna-dev/src/runtime/source_snapshot.rs")
        || path_str.contains("/crates/bijux-dna-runner/src/backend/slurm/qualification/")
        || path_str.contains("/crates/bijux-dna-runner/src/step_runner/external_artifact.rs")
        || path_str.contains("/crates/bijux-dna-containers/src/build/")
        || path_str.ends_with("/crates/bijux-dna-containers/src/site_policy/repository_archive.rs")
        || path_str.contains("/crates/bijux-dna-containers-qa/src/security_verification/")
        || path_str.contains("/crates/bijux-dna/src/commands/policies.rs")
        || path_str.contains("/crates/bijux-dna/src/commands/hpc/hpc_impl.rs")
        || path_str.contains("/crates/bijux-dna/src/commands/vcf/vcf_impl.rs")
        || path_str.contains("/crates/bijux-dna-domain-fastq/src/stages/contract/runtime/")
        || path_str.contains(
            "/crates/bijux-dna-domain-compiler/src/compiler/validation/strict_stage_schemas.rs",
        )
        || path_str.contains("/crates/bijux-dna-db-ref/src/resolution/reference_assets.rs")
        || path_str.contains("/crates/bijux-dna-domain-bam/src/artifacts.rs")
        || path_str.ends_with("/crates/bijux-dna-domain-bam/src/artifacts/transformation/sort_index.rs")
        || path_str.contains("/crates/bijux-dna-domain-vcf/src/artifacts.rs")
        || path_str.contains("/crates/bijux-dna-api/src/v1/run/runtime_support.rs")
        || path_str.contains(
            "/crates/bijux-dna-dev/src/commands/ops/tooling/operator_workflow_maturity.rs",
        )
        || path_str.contains(
            "/crates/bijux-dna-api/src/internal/fastq/stages/preprocess/stage_backend_policy.rs",
        )
        || path_str
            .contains("/crates/bijux-dna-api/src/internal/handlers/cross/bam_exec_contracts.rs")
        || path_str.contains("/crates/bijux-dna-stages-vcf/src/pipeline.rs")
}

fn is_path_policies_test(path: &Path) -> bool {
    path.to_string_lossy().ends_with(
        "/crates/bijux-dna-policies/tests/boundaries/surface/structure_layout/path_policies.rs",
    )
}

#[test]
fn policy__boundaries__path_policies__src_bin_requires_bin_targets() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_dir())
    {
        if entry.file_name() != "src" {
            continue;
        }
        let crate_root = entry.path().parent().unwrap();
        let src_bin = entry.path().join("bin");
        if !src_bin.exists() {
            continue;
        }
        let has_bins = std::fs::read_dir(&src_bin).is_ok_and(|entries| {
            entries
                .filter_map(Result::ok)
                .any(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs"))
        });
        if !has_bins {
            offenders.push(crate_root.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "src/bin must contain at least one .rs binary source:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__path_policies__src_test_paths_are_explicitly_test_only() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() && !entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();
        if !path.to_string_lossy().contains("/src/") {
            continue;
        }
        let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
        let rel = path.strip_prefix(&root).unwrap_or(path).to_string_lossy().replace('\\', "/");
        if rel == "crates/bijux-dna-testkit/src/temp/test_paths.rs" {
            continue;
        }
        if name.contains("test") && !is_explicit_private_test_path(path) {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "src test paths must contain only explicitly test-gated Rust modules:\n{}",
        offenders.join("\n")
    );
}

fn is_explicit_private_test_source(path: &Path) -> bool {
    if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
        return false;
    }
    let Ok(source) = std::fs::read_to_string(path) else { return false };
    bijux_dna_testkit::workspace_support::strip_rust_test_modules(&source).trim().is_empty()
        && !bijux_dna_testkit::workspace_support::select_rust_test_modules(&source)
            .trim()
            .is_empty()
}

fn is_explicit_private_test_path(path: &Path) -> bool {
    if path.is_file() {
        return is_explicit_private_test_source(path);
    }
    let mut sources = 0usize;
    for entry in WalkDir::new(path).into_iter().skip(1) {
        let Ok(entry) = entry else { return false };
        if entry.file_type().is_dir() {
            continue;
        }
        if !entry.file_type().is_file() || !is_explicit_private_test_source(entry.path()) {
            return false;
        }
        sources += 1;
    }
    sources > 0
}

#[test]
fn policy__boundaries__path_policies__private_test_paths_reject_production_and_empty_groups() {
    let root = bijux_dna_testkit::temp::tempdir_for("private-test-path-policy");
    let tests = root.path().join("tests");
    std::fs::create_dir(&tests).expect("create private test group");
    assert!(!is_explicit_private_test_path(&tests));
    let file = tests.join("fixture.rs");
    std::fs::write(&file, "#![cfg(test)]\nfn fixture() {}\n").expect("write test-only source");
    assert!(is_explicit_private_test_path(&file));
    assert!(is_explicit_private_test_path(&tests));
    let production = tests.join("production.rs");
    std::fs::write(&production, "pub fn production() {}\n").expect("write production source");
    assert!(!is_explicit_private_test_path(&production));
    assert!(!is_explicit_private_test_path(&tests));
    std::fs::write(&production, "const MARKER: &str = \"#![cfg(test)]\";\n")
        .expect("write literal cfg marker");
    assert!(!is_explicit_private_test_path(&production));
}

fn run_artifact_literal_offenders(targets: &[PathBuf]) -> anyhow::Result<Vec<PathBuf>> {
    let mut offenders = Vec::new();
    for target in targets {
        anyhow::ensure!(
            target.is_dir(),
            "run artifact policy source root is missing: {}",
            target.display()
        );
        let mut inspected = 0usize;
        for entry in WalkDir::new(target) {
            let entry = entry?;
            if !entry.file_type().is_file()
                || entry.path().extension().and_then(|s| s.to_str()) != Some("rs")
            {
                continue;
            }
            inspected += 1;
            let source = std::fs::read_to_string(entry.path())?;
            let source = bijux_dna_testkit::workspace_support::strip_rust_test_modules(&source);
            if source.contains("\"run_artifacts\"") {
                offenders.push(entry.path().to_path_buf());
            }
        }
        anyhow::ensure!(
            inspected > 0,
            "run artifact policy source root contains no Rust files: {}",
            target.display()
        );
    }
    Ok(offenders)
}

#[test]
fn policy__boundaries__path_policies__run_artifacts_paths_use_runtime_helpers() {
    let root = workspace_root();
    let targets = ["bijux-dna-api", "bijux-dna", "bijux-dna-stages-fastq", "bijux-dna-stages-bam"]
        .map(|name| root.join("crates").join(name).join("src"));
    let offenders =
        run_artifact_literal_offenders(&targets).expect("inspect actual run artifact consumers");
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "run artifact consumers must use the shared path owner: {:?}",
        offenders
    );
}

#[test]
fn policy__boundaries__path_policies__run_artifact_scan_rejects_missing_empty_and_offending_sources(
) -> anyhow::Result<()> {
    let temp = bijux_dna_testkit::temp::tempdir_for("run-artifact-path-policy");
    let root = temp.path().join("src");
    assert!(run_artifact_literal_offenders(std::slice::from_ref(&root)).is_err());
    std::fs::create_dir(&root)?;
    assert!(run_artifact_literal_offenders(std::slice::from_ref(&root)).is_err());
    let source = root.join("consumer.rs");
    std::fs::write(&source, "fn consumer() { out.join(\"run_artifacts\"); }\n")?;
    assert_eq!(run_artifact_literal_offenders(std::slice::from_ref(&root))?, vec![source.clone()]);
    std::fs::write(&source, "fn consumer() { shared_path(out); }\n")?;
    assert!(run_artifact_literal_offenders(std::slice::from_ref(&root))?.is_empty());
    Ok(())
}

#[test]
fn policy__boundaries__path_policies__write_locations_are_confined_to_effect_owners() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let patterns = ["std::fs::OpenOptions", "std::fs::write", "File::create("];

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        if entry.path().to_string_lossy().contains("/tests/") {
            continue;
        }
        if is_allowed_writer_path(entry.path()) {
            continue;
        }
        let content = bijux_dna_testkit::workspace_support::strip_rust_test_modules(
            &std::fs::read_to_string(entry.path()).expect("read source"),
        );
        if patterns.iter().any(|pattern| content.contains(pattern)) {
            offenders.push(entry.path().display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "direct filesystem writes must stay in declared runtime, execution, container, QA, or infrastructure owners:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__path_policies__crates_do_not_reference_removed_fastq_test_paths() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let needles = ["tests/data/fastq/"];
    let exts = ["rs", "toml", "md"];

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if !exts.contains(&ext) {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        if is_path_policies_test(path) {
            continue;
        }
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "crates must not reference tests/data/fastq paths:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__path_policies__developer_effects_have_exact_runtime_owners() {
    for relative in ["runtime/process.rs", "runtime/source_snapshot.rs"] {
        assert!(is_allowed_writer_path(&PathBuf::from(format!(
            "/workspace/crates/bijux-dna-dev/src/{relative}"
        ))));
    }
    for relative in [
        "runtime/workspace.rs",
        "runtime/other.rs",
        "runtime/process.rs.disabled",
        "application/ci/executables.rs",
        "commands/ops/tooling/ci.rs",
    ] {
        assert!(!is_allowed_writer_path(&PathBuf::from(format!(
            "/workspace/crates/bijux-dna-dev/src/{relative}"
        ))));
    }
}
