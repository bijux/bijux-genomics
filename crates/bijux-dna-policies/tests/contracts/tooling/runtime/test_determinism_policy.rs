#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ScratchAuthority {
    Other,
    CurrentDirectory,
    CurrentDirectoryArtifacts,
    Repository,
}

impl ScratchAuthority {
    fn merged(self, other: Self) -> Self {
        if self == Self::CurrentDirectoryArtifacts || other == Self::CurrentDirectoryArtifacts {
            Self::CurrentDirectoryArtifacts
        } else if self == Self::CurrentDirectory || other == Self::CurrentDirectory {
            Self::CurrentDirectory
        } else if self == Self::Repository || other == Self::Repository {
            Self::Repository
        } else {
            Self::Other
        }
    }
}

struct ScratchBinding<'a> {
    name: &'a str,
    authority: ScratchAuthority,
    scope: usize,
}

struct ScratchUpdate<'a> {
    at: usize,
    name: &'a str,
    authority: ScratchAuthority,
    scope: usize,
    declaration: bool,
    conditional: bool,
}

struct ScratchSource<'a> {
    tokens: Vec<crate::rust_source_tokens::SourceToken<'a>>,
    closing: Vec<Option<usize>>,
    bindings: Vec<ScratchBinding<'a>>,
    scopes: Vec<(usize, bool)>,
    updates: Vec<ScratchUpdate<'a>>,
}

impl<'a> ScratchSource<'a> {
    fn new(content: &'a str) -> Option<Self> {
        let tokens = crate::rust_source_tokens::source_tokens(content)?;
        let mut closing = vec![None; tokens.len()];
        let mut openings = Vec::new();
        for (index, token) in tokens.iter().enumerate().filter(|(_, token)| !token.literal) {
            match token.text {
                "(" | "[" | "{" => openings.push(index),
                ")" | "]" | "}" => {
                    let opening = openings.pop()?;
                    let expected = match token.text {
                        ")" => "(",
                        "]" => "[",
                        _ => "{",
                    };
                    if tokens[opening].text != expected {
                        return None;
                    }
                    closing[opening] = Some(index);
                }
                _ => {}
            }
        }
        if !openings.is_empty() {
            return None;
        }
        Some(Self {
            tokens,
            closing,
            bindings: Vec::new(),
            scopes: vec![(usize::MAX, false)],
            updates: Vec::new(),
        })
    }

    fn text(&self, index: usize) -> Option<&'a str> {
        self.tokens.get(index).map(|token| token.text)
    }

    fn binding(&self, name: &str) -> ScratchAuthority {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.name == name)
            .map_or(ScratchAuthority::Other, |binding| binding.authority)
    }

    fn statement_end(&self, mut index: usize) -> Option<usize> {
        while index < self.tokens.len() {
            match self.text(index)? {
                ";" => return Some(index),
                "}" => return None,
                "(" | "[" | "{" => index = self.closing[index]? + 1,
                _ => index += 1,
            }
        }
        None
    }

    fn path_argument(&self, start: usize, end: usize) -> Option<&'a str> {
        let token = self.tokens.get(start).filter(|_| start < end)?;
        if token.literal {
            let text = token.text;
            let quoted = if text.starts_with('"') && text.ends_with('"') {
                Some(&text[1..text.len() - 1])
            } else if text.starts_with('r') {
                text.find('"')
                    .zip(text.rfind('"'))
                    .filter(|(first, last)| first < last)
                    .map(|(first, last)| &text[first + 1..last])
            } else {
                None
            };
            return quoted;
        }
        for prefix in [["Path", "::", "new", "("], ["PathBuf", "::", "from", "("]] {
            if self.tokens.get(start..start + 4).is_some_and(|tokens| {
                tokens
                    .iter()
                    .zip(prefix)
                    .all(|(token, expected)| !token.literal && token.text == expected)
            }) {
                return self.path_argument(start + 4, end);
            }
        }
        None
    }

    // Only recognized filesystem path arguments establish an effect boundary.
    // Bare imported functions lose that authority when a local callable shadows them.
    fn filesystem_path_call(&self, path: &[&str]) -> bool {
        let unbound =
            path.len() == 1 && !self.bindings.iter().any(|binding| binding.name == path[0]);
        (unbound && matches!(path[0], "write" | "create_dir" | "create_dir_all"))
            || matches!(
                path,
                ["fs", "write" | "create_dir" | "create_dir_all"]
                    | ["std", "fs", "write" | "create_dir" | "create_dir_all"]
                    | ["File", "create" | "open"]
                    | ["fs", "File", "create" | "open"]
                    | ["std", "fs", "File", "create" | "open"]
            )
    }

    fn first_argument_end(&self, mut cursor: usize, end: usize) -> usize {
        while cursor < end {
            match self.text(cursor) {
                Some(",") => break,
                Some("(" | "[" | "{") => {
                    let Some(close) = self.closing[cursor] else { return end };
                    cursor = close + 1;
                }
                _ => cursor += 1,
            }
        }
        cursor
    }

    fn chain(&self, start: usize, end: usize) -> (ScratchAuthority, usize, bool) {
        let mut cursor = start;
        let mut authority = ScratchAuthority::Other;
        let mut forbidden = false;
        if self.text(cursor) == Some("(") {
            if let Some(close) = self.closing[cursor].filter(|&close| close < end) {
                let result = self.expression(cursor + 1, close);
                authority = result.0;
                forbidden |= result.1;
                cursor = close + 1;
            }
        } else if self.tokens.get(cursor).is_some_and(|token| !token.literal) {
            let mut path = vec![self.tokens[cursor].text];
            cursor += 1;
            while cursor + 1 < end && self.text(cursor) == Some("::") {
                path.push(self.tokens[cursor + 1].text);
                cursor += 2;
            }
            if self.text(cursor) == Some("(") {
                if let Some(close) = self.closing[cursor].filter(|&close| close < end) {
                    let unbound_current_dir = path.as_slice() == ["current_dir"]
                        && !self.bindings.iter().any(|binding| binding.name == "current_dir");
                    if close == cursor + 1
                        && (unbound_current_dir
                            || matches!(
                                path.as_slice(),
                                ["env", "current_dir"] | ["std", "env", "current_dir"]
                            ))
                    {
                        authority = ScratchAuthority::CurrentDirectory;
                    } else if close == cursor + 1
                        && path.as_slice() == ["workspace_paths", "repo_root"]
                    {
                        authority = ScratchAuthority::Repository;
                    }
                    if self.filesystem_path_call(&path) {
                        let argument_end = self.first_argument_end(cursor + 1, close);
                        let argument = self.expression(cursor + 1, argument_end);
                        forbidden |=
                            argument.0 == ScratchAuthority::CurrentDirectoryArtifacts || argument.1;
                    }
                    cursor = close + 1;
                }
            } else if path.len() == 1 {
                authority = self.binding(path[0]);
            }
        } else {
            cursor += 1;
        }
        while cursor < end {
            if self.text(cursor) == Some("?") {
                cursor += 1;
                continue;
            }
            if self.text(cursor) != Some(".") {
                break;
            }
            let Some(method) = self.text(cursor + 1) else { break };
            if self.text(cursor + 2) != Some("(") {
                authority = ScratchAuthority::Other;
                cursor += 2;
                continue;
            }
            let Some(close) = self.closing[cursor + 2].filter(|&close| close < end) else { break };
            if method == "join" {
                if authority == ScratchAuthority::CurrentDirectory
                    && self
                        .path_argument(cursor + 3, close)
                        .is_some_and(|path| path == "artifacts" || path.starts_with("artifacts/"))
                {
                    authority = ScratchAuthority::CurrentDirectoryArtifacts;
                }
                let argument = self.expression(cursor + 3, close);
                forbidden |= argument.1;
                if argument.0 == ScratchAuthority::Repository {
                    authority = ScratchAuthority::Repository;
                } else if self
                    .path_argument(cursor + 3, close)
                    .is_some_and(|path| Path::new(path).is_absolute())
                {
                    authority = ScratchAuthority::Other;
                }
            } else if matches!(
                method,
                "is_absolute"
                    | "is_relative"
                    | "is_file"
                    | "is_dir"
                    | "exists"
                    | "starts_with"
                    | "ends_with"
            ) {
                authority = ScratchAuthority::Other;
            }
            cursor = close + 1;
        }
        (authority, cursor.max(start + 1), forbidden)
    }

    fn expression(&self, mut index: usize, end: usize) -> (ScratchAuthority, bool) {
        let mut authority = ScratchAuthority::Other;
        let mut forbidden = false;
        while index < end {
            if !self.tokens[index].literal
                && !matches!(self.text(index.saturating_sub(1)), Some("." | "::"))
            {
                let result = self.chain(index, end);
                authority = authority.merged(result.0);
                forbidden |= result.2;
                index = result.1;
            } else {
                index += 1;
            }
        }
        (authority, forbidden)
    }

    fn apply_updates(&mut self, index: usize) {
        for update in std::mem::take(&mut self.updates) {
            if update.at != index {
                self.updates.push(update);
                continue;
            }
            if update.declaration {
                self.bindings.push(ScratchBinding {
                    name: update.name,
                    authority: update.authority,
                    scope: update.scope,
                });
            } else if let Some(binding) =
                self.bindings.iter_mut().rev().find(|binding| binding.name == update.name)
            {
                binding.authority = if update.conditional {
                    binding.authority.merged(update.authority)
                } else {
                    update.authority
                };
            }
        }
    }

    fn schedule(&mut self, name: usize, equals: usize, declaration: bool) {
        let Some(end) = self.statement_end(equals + 1) else { return };
        let value = self.expression(equals + 1, end).0;
        let scope = self.scopes.last().expect("scratch source root scope").0;
        self.updates.push(ScratchUpdate {
            at: end,
            name: self.tokens[name].text,
            authority: value,
            scope,
            declaration,
            conditional: !declaration && self.scopes.iter().any(|(_, conditional)| *conditional),
        });
    }

    fn derives_scratch(mut self) -> bool {
        let mut declarations = Vec::new();
        for index in 0..self.tokens.len() {
            self.apply_updates(index);
            if self.tokens[index].literal {
                continue;
            }
            let text = self.tokens[index].text;
            if text == "{" {
                let from = (0..index)
                    .rev()
                    .find(|&at| matches!(self.text(at), Some(";" | "{" | "}")))
                    .map_or(0, |at| at + 1);
                let conditional = self.tokens[from..index].iter().any(|token| {
                    !token.literal
                        && matches!(token.text, "if" | "else" | "match" | "while" | "for" | "loop")
                });
                self.scopes.push((index, conditional));
            } else if text == "}" {
                if let Some((scope, _)) = self.scopes.pop() {
                    self.bindings.retain(|binding| binding.scope != scope);
                }
            } else if text == "let" {
                let mut name = index + 1;
                while matches!(self.text(name), Some("mut" | "ref")) {
                    name += 1;
                }
                if let Some(end) = self.statement_end(name + 1) {
                    if let Some(equals) = (name + 1..end)
                        .find(|&at| self.text(at) == Some("=") && self.text(at + 1) != Some("="))
                    {
                        declarations.push(name);
                        self.schedule(name, equals, true);
                    }
                }
            } else if self.text(index + 1) == Some("=")
                && self.text(index + 2) != Some("=")
                && !declarations.contains(&index)
                && !matches!(self.text(index.saturating_sub(1)), Some("." | "::"))
            {
                self.schedule(index, index + 1, false);
            }
            if !matches!(self.text(index.saturating_sub(1)), Some("." | "::"))
                && self.chain(index, self.tokens.len()).2
            {
                return true;
            }
        }
        false
    }
}

fn derives_scratch_from_current_dir(content: &str) -> bool {
    ScratchSource::new(content).is_none_or(ScratchSource::derives_scratch)
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
