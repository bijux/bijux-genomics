#![allow(non_snake_case)]
use walkdir::WalkDir;

use crate::cargo_metadata_authority::CargoPackageSnapshot;
use crate::rust_source_module_registration::registered_private_helper_bucket;

fn workspace_root() -> std::path::PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__determinism__workspace_determinism_suite__tests_layout_by_intent_is_complete() {
    let root = workspace_root();
    let suite_buckets = ["boundaries", "contracts", "determinism", "schemas"];
    let mut offenders = Vec::new();
    let mut authority = None;

    let crates_dir = root.join("crates");
    for entry in std::fs::read_dir(&crates_dir)
        .unwrap_or_else(|err| panic!("read {}: {err}", crates_dir.display()))
    {
        let Ok(entry) = entry else { continue };
        let member_dir = entry.path();
        if !member_dir.is_dir() {
            continue;
        }
        let tests = member_dir.join("tests");
        if !tests.exists() {
            continue;
        }
        if !member_dir.join("docs/TESTS.md").exists() {
            offenders.push(format!("{}: missing docs/TESTS.md", member_dir.display()));
        }
        if tests.join("README.md").exists() {
            offenders.push(format!(
                "{}: test taxonomy belongs in docs/TESTS.md, not tests/README.md",
                member_dir.display()
            ));
        }
        for bucket in suite_buckets {
            let suite_dir = tests.join(bucket);
            if !suite_dir.exists() {
                continue;
            }
            let entrypoint = tests.join(format!("{bucket}.rs"));
            if !entrypoint.exists() {
                let observation = authority
                    .get_or_insert_with(|| CargoPackageSnapshot::capture(&root.join("Cargo.toml")));
                let registered = registered_private_helper_bucket(&member_dir, bucket, observation)
                    .unwrap_or_else(|error| {
                        panic!("read registered test helpers {}: {error:#}", suite_dir.display())
                    });
                if !registered {
                    offenders.push(format!(
                        "{}: tests/{bucket}/ has neither tests/{bucket}.rs nor complete declared target registration",
                        member_dir.display()
                    ));
                }
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "workspace test taxonomy must use docs/TESTS.md and materialized suite entrypoints:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__determinism__workspace_determinism_suite__policy_tests_use_repo_relative_paths() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let tests_root = root.join("crates/bijux-dna-policies/tests");
    let isolate_path_marker = ["artifacts", "isolates", ""].join("/");
    let iso_tag_marker = format!("{}_{}", "ISO", "TAG");
    for entry in WalkDir::new(&tests_root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if raw.contains(&isolate_path_marker) || raw.contains(&iso_tag_marker) {
            offenders.push(path.strip_prefix(&root).unwrap_or(path).display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "policy tests must not depend on isolate-tag-specific paths:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__determinism__workspace_determinism_suite__six_flake_snapshot_tests_remain_explicit() {
    let root = workspace_root();
    let file = root.join("crates/bijux-dna-analyze/tests/contracts/pipeline/pipeline_e2e.rs");
    let raw = std::fs::read_to_string(&file)
        .unwrap_or_else(|err| panic!("read {}: {err}", file.display()));
    let required = [
        "pipeline_fastq_to_bam_default_report_snapshot",
        "pipeline_bam_shotgun_report_snapshot",
        "pipeline_bam_capture_report_snapshot",
        "Nondeterminism vectors eliminated:",
        "Per-test unique temp roots via `TestPaths`",
        "Explicit stage sorting before report assembly",
    ];
    let mut missing = Vec::new();
    for needle in required {
        if !raw.contains(needle) {
            missing.push(needle);
        }
    }
    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "flake lock contract drifted; missing markers: {missing:?}"
    );
}

#[test]
fn policy__determinism__workspace_determinism_suite__helper_buckets_require_declared_targets_and_direct_registration(
) -> anyhow::Result<()> {
    let owned = bijux_dna_testkit::tempdir_for("test-helper-target-authority");
    let root = owned.path();
    std::fs::create_dir_all(root.join("src"))?;
    std::fs::create_dir_all(root.join("tests/contracts"))?;
    std::fs::write(root.join("src/lib.rs"), "pub fn library() {}\n")?;
    let manifest = root.join("Cargo.toml");
    let package =
        "[package]\nname = \"test-helper-authority\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
    std::fs::write(&manifest, format!("{package}[workspace]\n"))?;
    let target = root.join("tests/domain_contracts.rs");
    let helper = root.join("tests/contracts/values.rs");
    let registration = "#[path = \"contracts/values.rs\"]\nmod values;\n";
    let private = "#![cfg(test)]\n\n#[test]\nfn useful_contract() {}\n";
    std::fs::write(&target, registration)?;
    std::fs::write(&helper, private)?;
    let observe = || {
        let authority = CargoPackageSnapshot::capture(&manifest);
        registered_private_helper_bucket(root, "contracts", &authority)
    };
    let conventional = CargoPackageSnapshot::capture(&manifest);
    assert!(conventional.package(&manifest).targets.iter().any(|candidate| {
        candidate.kind.iter().any(|kind| kind == "test")
            && candidate.src_path.as_std_path() == target.as_path()
    }));
    assert!(registered_private_helper_bucket(root, "contracts", &conventional)?);

    let explicit = format!("{package}autotests = false\n[workspace]\n[[test]]\nname = \"declared-contracts\"\npath = \"tests/domain_contracts.rs\"\n");
    std::fs::write(&manifest, &explicit)?;
    assert!(observe()?, "explicit Cargo test target must own its directly registered helper");
    std::fs::write(&manifest, format!("{package}autotests = false\n[workspace]\n"))?;
    assert!(!observe()?, "an existing source file is not authority when Cargo does not declare it");
    std::fs::write(&manifest, &explicit)?;
    for source in [
        "// #[path = \"contracts/values.rs\"] mod values;\n",
        "const TEXT: &str = r#\"#[path = \"contracts/values.rs\"] mod values;\"#;\n",
        "#[cfg(any())]\n#[path = \"contracts/values.rs\"]\nmod values;\n",
        "#![cfg(any())]\n#[path = \"contracts/values.rs\"]\nmod values;\n",
        "mod unrelated {}\n",
    ] {
        std::fs::write(&target, source)?;
        assert!(
            !observe()?,
            "source text without unconditional registration was admitted: {source}"
        );
    }
    std::fs::write(&target, registration)?;
    assert!(observe()?, "restored direct registration must be observed fresh");
    let orphan = root.join("tests/contracts/orphan.rs");
    std::fs::write(&orphan, private)?;
    assert!(!observe()?, "an unregistered sibling helper must remain an offender");
    std::fs::remove_file(&orphan)?;
    std::fs::write(&helper, "#[test]\nfn useful_contract() {}\n")?;
    assert!(!observe()?, "direct registration alone does not establish explicit private ownership");
    std::fs::write(&helper, private)?;

    for source in [
        "#[path = \"../foreign.rs\"]\nmod values;\n",
        "#[path = \"/foreign.rs\"]\nmod values;\n",
        "#[path = \"contracts/missing.rs\"]\nmod values;\n",
        "const TEXT: &str = r#\"unterminated\n",
        "#[path = \"contracts/values.rs\"]\nmod values {\n",
    ] {
        std::fs::write(&target, source)?;
        assert!(observe().is_err(), "invalid source acquisition was silently accepted: {source}");
    }
    std::fs::write(&target, [0xff_u8])?;
    assert!(observe().is_err(), "unreadable UTF8 target must fail closed");
    std::fs::write(&target, registration)?;
    let before_missing = CargoPackageSnapshot::capture(&manifest);
    std::fs::remove_file(&target)?;
    assert!(registered_private_helper_bucket(root, "contracts", &before_missing).is_err());
    std::fs::write(&target, registration)?;
    std::fs::remove_file(&helper)?;
    assert!(observe().is_err(), "an empty helper bucket must not become an admitted empty scan");
    std::fs::write(&helper, private)?;
    assert!(observe()?, "restoring the same physical target/helper must restore authority");
    Ok(())
}
