#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use crate::rust_source_scratch_paths::derives_scratch_from_current_dir;

use regex::Regex;
use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn rs_test_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let path_s = path.to_string_lossy();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let content = bijux_dna_testkit::read_policy_text(path);
        let is_integration_test = path_s.contains("/tests/");
        let is_unit_test_container = path_s.contains("/src/")
            && (content.contains("#[cfg(test)]") || content.contains("mod tests {"));
        if !is_integration_test && !is_unit_test_container {
            continue;
        }
        files.push(path.to_path_buf());
    }
    files.sort();
    files
}

fn test_scoped_content(path: &Path) -> String {
    let content = bijux_dna_testkit::read_policy_text(path);
    let path_s = path.to_string_lossy();
    if path_s.contains("/tests/") {
        return content;
    }
    bijux_dna_testkit::workspace_support::select_rust_test_modules(&content)
}

fn is_policy_file(path: &Path) -> bool {
    path.ends_with(
        "crates/bijux-dna-policies/tests/contracts/tooling/runtime/test_determinism_policy.rs",
    )
}

struct ArtifactWritePolicy {
    direct_write: Regex,
    bound_path: Regex,
    write_argument: Regex,
}

impl ArtifactWritePolicy {
    fn new() -> Self {
        let operations = r"\b(?P<operation>write|write_string|write_bytes|atomic_write_json|create_dir|create_dir_all|ensure_dir|open|create|create_file)\s*\(";
        Self {
            direct_write: Regex::new(&format!(
                r#"{operations}\s*&?\s*(?:(?:std::path::)?(?:PathBuf::from|Path::new)\(\s*)?"artifacts(?:/|")"#
            ))
            .expect("compile direct artifact write regex"),
            bound_path: Regex::new(
                r#"\blet\s+(?:mut\s+)?(?P<name>[A-Za-z_][A-Za-z_0-9]*)\s*(?::\s*(?:std::path::)?PathBuf\s*)?=\s*(?:std::path::)?(?:PathBuf::from|Path::new)\(\s*"artifacts(?:/|")"#,
            )
            .expect("compile bound artifact path regex"),
            write_argument: Regex::new(&format!(
                r"{operations}\s*&?\s*(?P<name>[A-Za-z_][A-Za-z_0-9]*)\b"
            ))
            .expect("compile artifact write argument regex"),
        }
    }

    fn writes_to_shared_artifacts(&self, content: &str) -> bool {
        if self
            .direct_write
            .captures_iter(content)
            .any(|capture| is_filesystem_path_call(content, &capture))
        {
            return true;
        }
        // Path values are also persisted plan metadata. A constructor alone is not a write.
        // Track direct local bindings into filesystem calls without exempting their source files.
        let scopes = bijux_dna_testkit::workspace_support::rust_block_ranges(content);
        let bound_paths = self
            .bound_path
            .captures_iter(content)
            .filter_map(|capture| {
                let name = capture.name("name")?;
                let scope = scopes
                    .iter()
                    .filter(|scope| scope.contains(&name.start()))
                    .min_by_key(|scope| scope.len())
                    .cloned()
                    .unwrap_or(0..content.len());
                Some((name.as_str(), name.end(), scope))
            })
            .collect::<Vec<_>>();
        self.write_argument
            .captures_iter(content)
            .filter(|capture| is_filesystem_path_call(content, capture))
            .any(|capture| {
                capture.name("name").is_some_and(|name| {
                    bound_paths.iter().any(|(bound, declared_at, scope)| {
                        *bound == name.as_str()
                            && *declared_at < name.start()
                            && scope.contains(&name.start())
                    })
                })
            })
    }
}

// A receiver's write method is not a filesystem path constructor. File creation and
// open methods still take paths and remain governed, as do static or imported fs writes.
fn is_filesystem_path_call(content: &str, capture: &regex::Captures<'_>) -> bool {
    let Some(operation) = capture.name("operation") else {
        return true;
    };
    operation.as_str() != "write" || !content[..operation.start()].trim_end().ends_with('.')
}

#[test]
fn policy__contracts__test_determinism_policy__artifact_write_scan_distinguishes_metadata_from_effects(
) {
    let policy = ArtifactWritePolicy::new();
    for source in [
        r#"std::fs::write("artifacts/report.json", data)?;"#,
        r#"write("artifacts/report.json", data)?;"#,
        r#"File::options().open("artifacts/report.json")?;"#,
        r#"fixture.write("artifacts/metadata", data)?; std::fs::write("artifacts/report.json", data)?;"#,
        "std::fs::write(\n    \"artifacts/report.json\", data)?;",
        r#"File::create(PathBuf::from("artifacts/report.json"))?;"#,
        r#"let output = PathBuf::from("artifacts/report.json"); atomic_write_json(&output, &report)?;"#,
        r#"let mut output: PathBuf = PathBuf::from("artifacts"); ensure_dir(&output.join("tests"))?;"#,
        r#"let output = Path::new("artifacts/report.json"); std::fs::write(output, data)?;"#,
        r#"fn owner() { let output = PathBuf::from("artifacts/report.json"); if enabled { std::fs::write(&output, data)?; } }"#,
    ] {
        assert!(policy.writes_to_shared_artifacts(source), "missed write: {source}");
    }
    for source in [
        r#"assert_eq!(plan.out_dir, PathBuf::from("artifacts/runs/bench/local-smoke"));"#,
        r#"fixture.write("artifacts/cache/caller-owned", b"preserve caller cache")?;"#,
        r#"let output = PathBuf::from("artifacts/relative.json"); fixture.write(&output, data)?;"#,
        r#"let output = PathBuf::from("artifacts/report.json"); render(root.path(), output)?;"#,
        r#"let output = paths.root.join("report.json"); std::fs::write(&output, "artifacts/metadata")?;"#,
        r#"std::fs::write(root.path().join("artifacts/report.json"), data)?;"#,
        r#"fn metadata() { let output = PathBuf::from("artifacts/report.json"); render(root.path(), output)?; } fn isolated() { let output = paths.root.join("report.json"); std::fs::write(&output, data)?; }"#,
    ] {
        assert!(
            !policy.writes_to_shared_artifacts(source),
            "mistook metadata for a write: {source}"
        );
    }
}

#[test]
fn policy__contracts__test_determinism_policy__tests_ban_systemtime_now() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for file in rs_test_files(&root) {
        if is_policy_file(&file) {
            continue;
        }
        let content = test_scoped_content(&file);
        if content.contains("SystemTime::now(") || content.contains("std::time::SystemTime::now(") {
            offenders.push(file.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tests must use bijux_dna_testkit::FixedClock instead of SystemTime::now:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__test_determinism_policy__tests_ban_thread_rng() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for file in rs_test_files(&root) {
        if is_policy_file(&file) {
            continue;
        }
        let content = test_scoped_content(&file);
        if content.contains("rand::thread_rng(") || content.contains("thread_rng(") {
            offenders.push(file.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tests must use bijux_dna_testkit::fixed_rng(seed) instead of thread_rng:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__test_determinism_policy__tests_do_not_write_to_artifacts_root() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let policy = ArtifactWritePolicy::new();

    for file in rs_test_files(&root) {
        if is_policy_file(&file) {
            continue;
        }
        let content = test_scoped_content(&file);
        if policy.writes_to_shared_artifacts(&content) {
            offenders.push(file.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tests must write under TEST_TMP_DIR/TestPaths, not hardcoded artifacts/:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__test_determinism_policy__tests_do_not_derive_scratch_from_current_dir() {
    let root = workspace_root();
    for source in [
        r#"let workspace = std::env::current_dir()?; let root = workspace.join("artifacts/tests"); std::fs::create_dir_all(&root)?;"#,
        r#"let root = std::env::current_dir()?.join("artifacts/tests"); fs::write(&root, data)?;"#,
        r#"let cwd = env::current_dir().expect("cwd"); let root = cwd.clone().join("artifacts"); File::create(&root)?;"#,
        r#"let cwd = current_dir()?; let root = cwd.join("nested").join("artifacts/tests"); create_dir_all(&root)?;"#,
        r#"let cwd = current_dir()?; let alias = cwd; std::fs::create_dir_all(alias.join("artifacts/tests"))?;"#,
        r#"let cwd = current_dir()?; { let cwd = paths.root; } std::fs::write(cwd.join("artifacts/tests"), data)?;"#,
        r#"let mut cwd = paths.root; if enabled { cwd = current_dir()?; } File::open(cwd.join("artifacts/tests"))?;"#,
        r#"std::fs::create_dir_all(current_dir()?.join(Path::new("artifacts/tests")))?;"#,
        r#"let mut root = current_dir()?; if enabled { root = paths.root; } fs::create_dir(root.join("artifacts/tests"))?;"#,
    ] {
        assert!(
            derives_scratch_from_current_dir(source),
            "missed cwd-derived filesystem path admission: {source}"
        );
    }
    for source in [
        r#"let root = std::env::current_dir()?.join("artifacts/build/cargo"); let env = bindings.remapped_environment(&root)?; for (name, path) in env { assert_eq!(PathBuf::from(path), root.join("debug").join(name)); }"#,
        r#"let root = current_dir()?.join("artifacts/tests"); assert!(root.is_absolute());"#,
        r#"let root = current_dir()?.join("artifacts/tests"); let write = |path, data| metadata(path, data); write(root, data);"#,
        r#"let root = current_dir()?.join("artifacts/tests"); let create_dir = |path| metadata(path); create_dir(root);"#,
        r#"let root = current_dir()?.join("artifacts/tests"); fixture.write(root, data);"#,
        r#"let root = workspace_paths::repo_root()?; let repo_root = if root.is_absolute() { root } else { std::env::current_dir().expect("cwd").join(root) }; let parent = repo_root.join("artifacts/tests/fixture-commands");"#,
        r#"let cwd = std::env::current_dir()?; let owner = paths.root.join("artifacts/tests");"#,
        r#"let root = current_dir()?; let root = paths.root; std::fs::create_dir_all(root.join("artifacts/tests"))?;"#,
        r#"let mut root = current_dir()?; root = paths.root; std::fs::create_dir_all(root.join("artifacts/tests"))?;"#,
        r#"let root = paths.root; { let root = current_dir()?; } std::fs::create_dir_all(root.join("artifacts/tests"))?;"#,
        r#"let current_dir = || paths.root; std::fs::create_dir_all(current_dir().join("artifacts/tests"))?;"#,
        r#"let root = object.current_dir(); std::fs::create_dir_all(root.join("artifacts/tests"))?;"#,
        r#"let root = std::env::current_dir()?.join("/owned/fixture"); std::fs::create_dir_all(root.join("artifacts/tests"))?;"#,
        r#"let mut root = current_dir()?; { root = paths.root; } std::fs::create_dir_all(root.join("artifacts/tests"))?;"#,
        r#"// current_dir()?.join("artifacts/tests");
let owner = paths.root;"#,
        r#"/* current_dir()?.join("artifacts/tests"); */ let owner = paths.root;"#,
        r##"let payload = r#"let cwd = current_dir()?; cwd.join("artifacts/tests");"#;"##,
        r#"let payload = "current_dir()?.join(\"artifacts/tests\");";"#,
    ] {
        assert!(
            !derives_scratch_from_current_dir(source),
            "metadata or non-cwd filesystem authority rejected: {source}"
        );
    }
    let process_owner = root.join("crates/bijux-dna/tests/support/fixtures/process.rs");
    assert!(
        !derives_scratch_from_current_dir(&bijux_dna_testkit::read_policy_text(&process_owner)),
        "repository admission and exclusive fixture ownership are not cwd scratch authority"
    );
    let mut offenders = Vec::new();
    for file in rs_test_files(&root) {
        if !is_policy_file(&file) && derives_scratch_from_current_dir(&test_scoped_content(&file)) {
            offenders.push(file.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tests must use TEST_TMP_DIR/TestPaths for filesystem paths under cwd-derived artifacts/:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__test_determinism_policy__jsonl_appends_use_locked_writer() {
    let root = workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let path_s = path.to_string_lossy();
        if path_s.contains("/target/") {
            continue;
        }
        if path_s.ends_with("/bijux-dna-runtime/src/run_layout.rs")
            || path_s.ends_with("/bijux-dna-runtime/src/run_layout/journal.rs")
            || path_s.ends_with("/bijux-dna/src/commands/bench_suite/bench_suite_part1.rs")
            || path_s.ends_with("/bijux-dna-core/tests/contracts/identity/run_index.rs")
        {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        let writes_jsonl = content.contains(".jsonl")
            && (content.contains("OpenOptions::new") || content.contains("append(true)"));
        let uses_locked_helpers = content.contains("append_jsonl_line(")
            || content.contains("append_execution_event(")
            || content.contains("append_run_index_record(");
        if writes_jsonl && !uses_locked_helpers {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "jsonl appends must route through locked writer helpers:\n{}",
        offenders.join("\n")
    );
}
