#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn allowed_test_command_arg(relative_path: &Path, arg: &str) -> bool {
    let normalized = arg.trim();
    let prebuilt = normalized
        .strip_prefix("env!(\"CARGO_BIN_EXE_")
        .and_then(|name| name.strip_suffix("\")"))
        .is_some_and(|name| {
            !name.is_empty()
                && name.chars().all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
                })
        });
    prebuilt
        || normalized == "\"cargo\""
        || normalized == "\"bash\""
        || normalized == "\"git\""
        || normalized == "\"awk\""
        || owned_harness_command(relative_path, normalized)
}

fn owned_harness_command(relative_path: &Path, arg: &str) -> bool {
    // These owners exercise governed native and cross-language entrypoints with owned fixtures.
    // Grant each constructor expression at its exact source path, never to the entire crate.
    match relative_path.to_str() {
        Some("crates/bijux-dna-science/tests/contracts/archive_sync.rs") => arg == "\"python3\"",
        Some("crates/bijux-dna-dev/tests/bootstrap.rs") => arg == "\"sh\"" || arg == "&binary",
        Some("crates/bijux-dna-process/tests/contracts.rs") => {
            matches!(arg, "\"sh\"" | "\"unavailable-capture-contract-executable\"")
        }
        Some("crates/bijux-dna-testkit/tests/schemas/public_api_surface.rs") => {
            matches!(arg, "std::env::current_exe()?" | "&executable")
        }
        Some(
            "crates/bijux-dna-db-ref/tests/contracts/runtime_root.rs"
            | "crates/bijux-dna-planner-bam/tests/contracts/plan/registry_root.rs",
        ) => arg == "std::env::current_exe()?",
        Some("crates/bijux-dna/tests/contracts/support/repository_sandbox/mod.rs") => {
            arg == "executable"
        }
        _ => false,
    }
}

fn content_spawns_forbidden_test_command(relative_path: &Path, content: &str) -> bool {
    let Some(tokens) = source_tokens(content) else { return true };
    let mut index = 0;
    while index + 2 < tokens.len() {
        let constructor = ["Command", "::", "new"];
        if tokens[index..index + 3]
            .iter()
            .zip(constructor)
            .all(|(token, expected)| !token.literal && token.text == expected)
        {
            if tokens.get(index + 3).is_none_or(|token| token.literal || token.text != "(") {
                return true;
            }
            let Some((argument, after)) = constructor_argument(&tokens, index + 3) else {
                return true;
            };
            if !allowed_test_command_arg(relative_path, &argument) {
                return true;
            }
            index = after;
        } else {
            index += 1;
        }
    }
    false
}

struct SourceToken<'a> {
    text: &'a str,
    literal: bool,
}

fn quoted_end(bytes: &[u8], start: usize, quote: u8) -> Option<usize> {
    let mut index = start + 1;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index += 2;
        } else if bytes[index] == quote {
            return Some(index + 1);
        } else {
            index += 1;
        }
    }
    None
}

enum RawLiteralEnd {
    NotRaw,
    Closed(usize),
    Unterminated,
}

fn raw_literal_end(bytes: &[u8], start: usize) -> RawLiteralEnd {
    let mut quote = start + 1;
    while bytes.get(quote) == Some(&b'#') {
        quote += 1;
    }
    if bytes.get(quote) != Some(&b'"') {
        return RawLiteralEnd::NotRaw;
    }
    let hashes = quote - start - 1;
    let mut index = quote + 1;
    while index < bytes.len() {
        if bytes[index] == b'"'
            && (0..hashes).all(|offset| bytes.get(index + 1 + offset) == Some(&b'#'))
        {
            return RawLiteralEnd::Closed(index + hashes + 1);
        }
        index += 1;
    }
    RawLiteralEnd::Unterminated
}

fn character_literal_end(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut end = start + 1;
    if bytes.get(end) == Some(&b'\\') {
        end += 1;
        match bytes.get(end)? {
            b'x' => end += 3,
            b'u' if bytes.get(end + 1) == Some(&b'{') => {
                end += 2;
                while bytes.get(end).is_some_and(|byte| byte.is_ascii_hexdigit() || *byte == b'_') {
                    end += 1;
                }
                if bytes.get(end) != Some(&b'}') {
                    return None;
                }
                end += 1;
            }
            _ => end += 1,
        }
    } else {
        end += content.get(end..)?.chars().next()?.len_utf8();
    }
    (bytes.get(end) == Some(&b'\'')).then_some(end + 1)
}

// This bounded source scan preserves literal bytes and complete argument boundaries;
// runtime resolver contracts remain responsible for executable provenance.
fn source_tokens(content: &str) -> Option<Vec<SourceToken<'_>>> {
    let bytes = content.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let start = index;
        let character = content.get(index..)?.chars().next()?;
        if character.is_whitespace() {
            index += character.len_utf8();
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"//") {
            while bytes.get(index).is_some_and(|byte| *byte != b'\n') {
                index += 1;
            }
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"/*") {
            let mut depth = 1_usize;
            index += 2;
            while depth > 0 {
                match bytes.get(index..index + 2)? {
                    b"/*" => {
                        depth = depth.checked_add(1)?;
                        index += 2;
                    }
                    b"*/" => {
                        depth -= 1;
                        index += 2;
                    }
                    _ => index += 1,
                }
            }
            continue;
        }
        let prefix = if matches!(bytes[index], b'b' | b'c') { index + 1 } else { index };
        let raw = if bytes.get(prefix) == Some(&b'r') {
            match raw_literal_end(bytes, prefix) {
                RawLiteralEnd::NotRaw => None,
                RawLiteralEnd::Closed(end) => Some(end),
                RawLiteralEnd::Unterminated => return None,
            }
        } else {
            None
        };
        let literal = raw.or_else(|| match bytes.get(prefix) {
            Some(b'"') => quoted_end(bytes, prefix, b'"'),
            Some(b'\'') => character_literal_end(content, prefix),
            _ => None,
        });
        if let Some(end) = literal {
            index = end;
            tokens.push(SourceToken { text: content.get(start..end)?, literal: true });
            continue;
        }
        if bytes[index] == b'"' || bytes.get(prefix) == Some(&b'"') {
            return None;
        }
        if bytes.get(index..index + 2) == Some(b"::") {
            index += 2;
        } else if character.is_alphanumeric() || character == '_' {
            index += character.len_utf8();
            while let Some(character) = content.get(index..)?.chars().next() {
                if !character.is_alphanumeric() && character != '_' {
                    break;
                }
                index += character.len_utf8();
            }
        } else {
            index += character.len_utf8();
        }
        tokens.push(SourceToken { text: content.get(start..index)?, literal: false });
    }
    Some(tokens)
}

fn constructor_argument(tokens: &[SourceToken<'_>], opening: usize) -> Option<(String, usize)> {
    let mut delimiters = vec!["("];
    let mut argument = String::new();
    for (index, token) in tokens.iter().enumerate().skip(opening + 1) {
        if !token.literal {
            match token.text {
                "(" | "[" | "{" => delimiters.push(token.text),
                ")" | "]" | "}" => {
                    let expected = match token.text {
                        ")" => "(",
                        "]" => "[",
                        _ => "{",
                    };
                    if delimiters.pop()? != expected {
                        return None;
                    }
                    if delimiters.is_empty() {
                        return (!argument.is_empty()).then_some((argument, index + 1));
                    }
                }
                "," if delimiters.len() == 1 => return None,
                _ => {}
            }
        }
        if !token.literal
            && argument
                .chars()
                .last()
                .is_some_and(|character| character.is_alphanumeric() || character == '_')
            && token
                .text
                .chars()
                .next()
                .is_some_and(|character| character.is_alphanumeric() || character == '_')
        {
            argument.push(' ');
        }
        argument.push_str(token.text);
    }
    None
}

fn is_allowed_command_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path.ends_with("crates/bijux-dna-process/src/capture.rs")
        || path_str.contains("/crates/bijux-dna-runner/")
        || path_str.contains("/crates/bijux-dna-containers/")
        || path_str.contains("/crates/bijux-dna-containers-qa/")
        || path_str.contains("/crates/bijux-dna-dev/")
        || path_str.contains("/crates/bijux-dna-stages-vcf/")
}

#[test]
fn policy__contracts__command_spawn_policy__command_spawning_is_confined_to_runner_and_env_tooling()
{
    let root = workspace_root();
    let mut offenders = Vec::new();
    let needles = ["std::process::Command", "Command::new"];

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if path.to_string_lossy().contains("/tests/") {
            continue;
        }
        if is_allowed_command_path(path) {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        let production = bijux_dna_testkit::workspace_support::strip_rust_test_modules(&content);
        if needles.iter().any(|needle| production.contains(needle)) {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "process construction must remain in the explicit lifecycle and execution owners:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__command_spawn_policy__crate_tests_do_not_spawn_external_commands() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let forbidden_runtime_needles = [
        "tokio::process::Command",
        "assert_cmd",
        "duct::cmd",
        "DockerRunner",
        "docker::",
        "bollard::",
    ];

    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let path_str = path.to_string_lossy();
        if !path_str.contains("/tests/") {
            continue;
        }
        if path_str.contains("/tests/boundaries/") {
            continue;
        }
        if path_str.contains("/crates/bijux-dna-runner/tests/")
            || path_str.contains("/crates/bijux-dna-containers/tests/")
            || path_str.contains("/crates/bijux-dna-containers-qa/tests/")
        {
            continue;
        }
        if path_str.contains("/crates/bijux-dna-policies/tests/")
            || path_str.contains("/crates/bijux-dna-stages-fastq/tests/architecture.rs")
            || path_str
                .contains("/crates/bijux-dna-api/tests/contracts/v1_fastq_small_integration.rs")
            || path_str.contains("/crates/bijux-dna-runner/tests/boundaries/architecture.rs")
            || path_str.contains("/crates/bijux-dna-containers-qa/tests/boundaries/architecture.rs")
            || path_str.contains("/crates/bijux-dna-containers/tests/contracts/resolve_runtime.rs")
            || path_str.contains("/crates/bijux-dna-runtime/tests/contracts/run_layout_contract.rs")
            || path_str.contains(
                "/crates/bijux-dna-runtime/tests/schemas/schema/runtime_schema_snapshots.rs",
            )
            || path_str.contains("/crates/bijux-dna-domain-compiler/tests/domain_registry_cli.rs")
            || path_str.contains("/crates/bijux-dna/tests/contracts/cli_library_parity.rs")
        {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        let relative_path = path.strip_prefix(&root).expect("source belongs to workspace");
        if content_spawns_forbidden_test_command(relative_path, &content)
            || forbidden_runtime_needles.iter().any(|needle| content.contains(needle))
        {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "crate tests must not spawn arbitrary external commands; only governed workspace harness commands are allowed:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__command_spawn_policy__harness_commands_stay_with_their_exact_owners() {
    assert!(is_allowed_command_path(Path::new(
        "/workspace/crates/bijux-dna-process/src/capture.rs"
    )));
    assert!(!is_allowed_command_path(Path::new(
        "/workspace/crates/bijux-dna-process/src/projection.rs"
    )));
    for (owner, command) in [
        ("crates/bijux-dna-science/tests/contracts/archive_sync.rs", "\"python3\""),
        ("crates/bijux-dna-dev/tests/bootstrap.rs", "\"sh\""),
        ("crates/bijux-dna-dev/tests/bootstrap.rs", "&binary"),
        ("crates/bijux-dna-process/tests/contracts.rs", "\"sh\""),
        (
            "crates/bijux-dna-process/tests/contracts.rs",
            "\"unavailable-capture-contract-executable\"",
        ),
    ] {
        let invocation = format!("Command::new({command}).output()?");
        assert!(!content_spawns_forbidden_test_command(Path::new(owner), &invocation));
        let adjacent = Path::new(owner).with_file_name("unrelated.rs");
        assert!(content_spawns_forbidden_test_command(&adjacent, &invocation));
        assert!(content_spawns_forbidden_test_command(
            Path::new(owner),
            "Command::new(\"curl\").output()?"
        ));
        assert!(content_spawns_forbidden_test_command(
            &Path::new("foreign-checkout").join(owner),
            &invocation
        ));
    }
}

#[test]
fn policy__contracts__command_spawn_policy__native_harnesses_require_exact_complete_bindings() {
    for (owner, expression) in [
        (
            "crates/bijux-dna-testkit/tests/schemas/public_api_surface.rs",
            "std::env::current_exe()?",
        ),
        ("crates/bijux-dna-testkit/tests/schemas/public_api_surface.rs", "&executable"),
        ("crates/bijux-dna-db-ref/tests/contracts/runtime_root.rs", "std::env::current_exe()?"),
        (
            "crates/bijux-dna-planner-bam/tests/contracts/plan/registry_root.rs",
            "std::env::current_exe()?",
        ),
        ("crates/bijux-dna/tests/contracts/support/repository_sandbox/mod.rs", "executable"),
    ] {
        let invocation = format!("std::process::Command::new({expression}).output()?");
        assert!(!content_spawns_forbidden_test_command(Path::new(owner), &invocation), "{owner}");
        let adjacent = Path::new(owner).with_file_name("unrelated.rs");
        assert!(content_spawns_forbidden_test_command(&adjacent, &invocation));
        assert!(content_spawns_forbidden_test_command(
            &Path::new("foreign-checkout").join(owner),
            &invocation
        ));
        for command in ["curl", "rustc", "arbitrary-unbound-command"] {
            assert!(content_spawns_forbidden_test_command(
                Path::new(owner),
                &format!("Command::new(\"{command}\")")
            ));
        }
    }
}

#[test]
fn policy__contracts__command_spawn_policy__constructor_literals_do_not_change_call_boundaries() {
    let owner = Path::new("crates/bijux-dna-db-ref/tests/contracts/runtime_root.rs");
    for source in [
        "// Command::new(\"curl\")\nCommand::new(std::env::current_exe()?)",
        "/* outer /* Command::new(\"curl\") */ nested */ Command::new(std::env::current_exe()?)",
        r#"let quoted = "Command::new(\"curl\")"; Command::new(std::env::current_exe()?)"#,
        r###"let raw = r##"Command::new("curl")"##; Command::new(std::env::current_exe()?)"###,
        "let character = ')'; let byte = b'('; Command :: new(\nstd::env::current_exe(/* close ) */)?\n)",
        "let character = '\\u{29}'; Command::new(std::env::current_exe()?)",
        "let character = 'é';\u{200e}Command::new(std::env::current_exe()?)",
    ] {
        assert!(!content_spawns_forbidden_test_command(owner, source), "{source}");
    }
    for source in [
        "Command::new(std::env::current_exe()?).output()?; Command::new(\"curl\")",
        r###"Command::new(r##"std::env::current_exe()?)"##)"###,
        "Command::new(\"cargo)\")",
        "Command::new(b'(')",
    ] {
        assert!(content_spawns_forbidden_test_command(owner, source), "{source}");
    }
}

#[test]
fn policy__contracts__command_spawn_policy__redirected_or_malformed_constructor_arguments_refuse() {
    let owner = Path::new("crates/bijux-dna-db-ref/tests/contracts/runtime_root.rs");
    for source in [
        "Command::new(std::env::current_exe()?.with_file_name(\"curl\"))",
        "Command::new(std::env::current_exe().with_file_name(\"curl\")?)",
        "Command::new(std::env::current_exe().unwrap())",
        "Command::new(std::env::current_exe()?, \"curl\")",
        "Command::new(std::env::current_exe()?,)",
        "Command::new()",
        "Command::new(std::env::current_exe()?",
        "Command::new(std::env::current_exe(?))",
        "Command::new({std::env::current_exe()?])",
        "Command::new(\"unterminated)",
        "Command::new(r##\"unterminated)",
        "Command::new(std::env::current_exe()? /* unterminated)",
        "Command::new::<&str>(\"curl\")",
        "let constructor = Command::new; constructor(\"curl\")",
        "Command::new(env!(\"CARGO_BIN_EXE_bijux-dna\").with_file_name(\"curl\"))",
    ] {
        assert!(content_spawns_forbidden_test_command(owner, source), "{source}");
    }
    let sandbox = Path::new("crates/bijux-dna/tests/contracts/support/repository_sandbox/mod.rs");
    assert!(content_spawns_forbidden_test_command(sandbox, "Command::new(exe cutable)"));
    assert!(content_spawns_forbidden_test_command(sandbox, "Command::new(executable, foreign)"));
    assert!(!content_spawns_forbidden_test_command(
        sandbox,
        "Command::new(env!(\"CARGO_BIN_EXE_bijux-dna\"))"
    ));
}

#[test]
fn policy__contracts__command_spawn_policy__bijux_dna_tests_use_prebuilt_cli() {
    let tests_root = workspace_root().join("crates/bijux-dna/tests");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(tests_root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs")
        {
            continue;
        }
        let path = entry.path();
        let content = std::fs::read_to_string(path).expect("read integration test");
        if content.contains("Command::new(\"cargo\")") {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna integration tests must invoke env!(\"CARGO_BIN_EXE_bijux-dna\") instead of spawning nested Cargo builds:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__command_spawn_policy__run_controls_never_poll_for_progress() {
    let root = workspace_root();
    let operations = std::fs::read_to_string(
        root.join("crates/bijux-dna-api/src/runtime/run/reporting/operations.rs"),
    )
    .expect("read run control operations");
    let failure_injection = std::fs::read_to_string(
        root.join("crates/bijux-dna-api/src/runtime/run/reporting/failure_injection.rs"),
    )
    .expect("read failure injection controls");

    for forbidden in
        ["thread::sleep", "while started.elapsed()", ".join(\"run_control.json\").exists()"]
    {
        bijux_dna_policies::policy_assert!(
            !operations.contains(forbidden) && !failure_injection.contains(forbidden),
            "run controls must persist one observation and return; forbidden polling token: {forbidden}"
        );
    }
}
