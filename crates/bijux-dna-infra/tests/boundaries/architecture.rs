use std::collections::BTreeSet;
use std::path::Path;

#[test]
#[allow(clippy::too_many_lines)]
fn infra_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_dir_entries(
        root,
        ["Cargo.toml", "README.md", "docs/", "src/", "tests/"],
        "infra crate root must stay minimal and intentional",
    );

    assert_dir_entries(
        &root.join("docs"),
        [
            "ARCHITECTURE.md",
            "BOUNDARY.md",
            "COMMANDS.md",
            "DEPENDENCIES.md",
            "EFFECTS.md",
            "FORMATS.md",
            "INDEX.md",
            "PATHS.md",
            "PUBLIC_API.md",
            "TESTS.md",
        ],
        "infra docs must stay centralized under docs/",
    );

    assert_dir_entries(
        &root.join("src"),
        [
            "formats/",
            "hashing/",
            "io/",
            "lib.rs",
            "locking.rs",
            "logging/",
            "paths/",
            "retry/",
            "run_directories/",
            "stable_surface.rs",
            "temp.rs",
        ],
        "infra src tree must stay partitioned by low-level capability",
    );

    assert_dir_entries(
        &root.join("src/formats"),
        ["json.rs", "mod.rs", "stable_surface.rs", "yaml.rs"],
        "format support must stay split by encoded format",
    );
    assert_dir_entries(
        &root.join("src/hashing"),
        ["file_digest.rs", "mod.rs"],
        "hashing support must keep digest logic isolated",
    );
    assert_dir_entries(
        &root.join("src/io"),
        ["error.rs", "mod.rs", "read.rs", "remove.rs", "stable_surface.rs", "write.rs"],
        "io support must stay split by operation and error boundary",
    );
    assert_dir_entries(
        &root.join("src/logging"),
        ["bootstrap.rs", "mod.rs", "subscriber.rs"],
        "logging support must keep setup and subscriber ownership explicit",
    );
    assert_dir_entries(
        &root.join("src/paths"),
        [
            "bench.rs",
            "config.rs",
            "config_aliases.rs",
            "mod.rs",
            "run_layout.rs",
            "segments.rs",
            "stable_surface.rs",
        ],
        "path support must stay split by path family",
    );
    assert_dir_entries(
        &root.join("src/retry"),
        ["backoff.rs", "clock.rs", "mod.rs", "policy.rs", "runtime.rs", "stable_surface.rs"],
        "retry support must keep policy, timing, and runtime concerns separate",
    );
    assert_dir_entries(
        &root.join("src/run_directories"),
        ["contract.rs", "mod.rs", "operations.rs", "stable_surface.rs"],
        "run-directory support must keep contract and operations separate",
    );

    assert_dir_entries(
        &root.join("tests"),
        [
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "determinism/",
            "determinism.rs",
            "guardrails.rs",
            "schemas/",
            "schemas.rs",
            "snapshots/",
        ],
        "infra test tree must match the documented taxonomy",
    );
    assert_dir_entries(
        &root.join("tests/boundaries"),
        ["architecture.rs", "guardrails/", "guardrails.rs"],
        "boundary tests must stay focused on architecture and guardrails",
    );
    assert_dir_entries(
        &root.join("tests/boundaries/guardrails"),
        [
            "canonical_owner.rs",
            "dependencies.rs",
            "docs_layout.rs",
            "no_generic_helpers.rs",
            "policies.rs",
        ],
        "guardrail tests must keep one file per ownership concern",
    );
    assert_dir_entries(
        &root.join("tests/contracts"),
        ["io.rs", "run_layout.rs"],
        "contract tests must stay split by owned runtime capability",
    );
    assert_dir_entries(
        &root.join("tests/determinism"),
        ["hash.rs", "retry.rs"],
        "determinism tests must stay split by deterministic behavior",
    );
    assert_dir_entries(
        &root.join("tests/schemas"),
        ["public_surface.rs"],
        "schema tests must own snapshot-style public surface locks",
    );
}

fn assert_dir_entries<const N: usize>(path: &Path, expected: [&str; N], message: &str) {
    let actual = dir_entries(path);
    let expected = expected.into_iter().map(str::to_string).collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "{message}");
}

fn dir_entries(path: &Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
        .map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect()
}
