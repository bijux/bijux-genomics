#![allow(non_snake_case)]
use std::collections::BTreeSet;
use std::env;
use std::path::PathBuf;

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

// Interpret actual run directives, never workflow names, comments or notice prose.
// This lexer recognizes shell command boundaries and quoted arguments; it does
// not execute shell text or claim to validate arbitrary shell programming.
fn shell_commands(raw: &str) -> Vec<Vec<String>> {
    let mut commands = Vec::new();
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'(') && quote != Some('\'') {
            chars.next();
            let mut depth = 1;
            let mut substitution = String::new();
            for next in chars.by_ref() {
                match next {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                if depth == 0 {
                    break;
                }
                substitution.push(next);
            }
            commands.extend(shell_commands(&substitution));
            word.push_str("${substitution}");
            continue;
        }
        if ch == '`' && quote != Some('\'') {
            let substitution = chars.by_ref().take_while(|next| *next != '`').collect::<String>();
            commands.extend(shell_commands(&substitution));
            word.push_str("${substitution}");
            continue;
        }
        if ch == '\\' && quote != Some('\'') {
            if let Some(next) = chars.next() {
                if next != '\n' {
                    word.push(next);
                }
            }
            continue;
        }
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            } else {
                word.push(ch);
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            '{' if word.is_empty() && words.is_empty() => commands.push(vec!["{".to_string()]),
            '#' if word.is_empty() => {
                for rest in chars.by_ref() {
                    if rest == '\n' {
                        break;
                    }
                }
                if !words.is_empty() {
                    commands.push(std::mem::take(&mut words));
                }
            }
            '\n' | ';' | '&' | '|' => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
                if !words.is_empty() {
                    commands.push(std::mem::take(&mut words));
                }
            }
            ch if ch.is_whitespace() => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            _ => word.push(ch),
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    if !words.is_empty() {
        commands.push(words);
    }
    commands
}

fn command_start(words: &[String]) -> &[String] {
    let mut start = 0;
    while let Some(word) = words.get(start) {
        if matches!(word.as_str(), "if" | "then" | "elif" | "do" | "!" | "exec" | "env" | "time")
            || word.contains('=')
        {
            start += 1;
        } else {
            break;
        }
    }
    &words[start..]
}

fn has_direct_cargo_verification(commands: &[Vec<String>]) -> bool {
    commands.iter().any(|words| {
        let words = command_start(words);
        let Some(command) = words.first() else { return false };
        if command == "cargo" {
            words
                .iter()
                .skip(1)
                .any(|word| matches!(word.as_str(), "clippy" | "fmt" | "test" | "nextest" | "make"))
        } else if matches!(command.as_str(), "sh" | "bash") {
            words.windows(2).any(|pair| {
                pair[0] == "-c" && has_direct_cargo_verification(&shell_commands(&pair[1]))
            })
        } else {
            false
        }
    })
}

fn is_notice_only(commands: &[Vec<String>]) -> bool {
    !commands.is_empty()
        && commands.iter().all(|words| {
            let words = command_start(words);
            words.first().is_some_and(|word| {
                matches!(word.as_str(), "set" | "echo" | "printf" | "{")
                    || (word == "}" && words.get(1).is_some_and(|next| next.starts_with('>')))
            })
        })
}

fn workflow_uses_make_entrypoints(raw: &str) -> Result<bool, serde_yaml::Error> {
    let workflow: serde_yaml::Value = serde_yaml::from_str(raw)?;
    let runs = workflow["jobs"]
        .as_mapping()
        .into_iter()
        .flat_map(|jobs| jobs.values())
        .filter_map(|job| job["steps"].as_sequence())
        .flatten()
        .filter_map(|step| step["run"].as_str())
        .map(shell_commands)
        .collect::<Vec<_>>();
    if runs.iter().any(|commands| has_direct_cargo_verification(commands)) {
        return Ok(false);
    }
    let uses_make = runs
        .iter()
        .flatten()
        .any(|words| command_start(words).first().is_some_and(|command| command == "make"));
    Ok(uses_make || (!runs.is_empty() && runs.iter().all(|commands| is_notice_only(commands))))
}

#[test]
fn policy__contracts__ci_tools_policy__workflows_use_make_only() {
    let root = workspace_root();
    let workflows_dir = root.join(".github").join("workflows");
    let allowlist = [
        ".github/workflows/automerge-pr.yml",
        ".github/workflows/codecov.yml",
        ".github/workflows/github-policy.yml",
        ".github/workflows/labeler.yml",
        ".github/workflows/pr-approval-policy.yml",
    ];
    let mut offenders = Vec::new();
    for entry in WalkDir::new(workflows_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("yml") {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(&root)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");
        if allowlist.iter().any(|allowed| rel == *allowed) {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read workflow");
        if !workflow_uses_make_entrypoints(&content).expect("parse workflow run directives") {
            offenders.push(entry.path().display().to_string());
        }
    }
    offenders.sort();
    offenders.dedup();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "CI workflows must use Makefile entrypoints only: {:?}",
        offenders
    );
}

#[test]
fn policy__contracts__ci_tools_policy__notice_prose_does_not_select_verification_commands() {
    let workflow = |run: &str| {
        format!(
            "jobs:\n  check:\n    steps:\n      - run: {}\n",
            serde_json::to_string(run).unwrap()
        )
    };
    for run in [
        "make test",
        "# cargo test is governed by Make\nmake lint",
        "set -euo pipefail\n{\necho 'cargo test and make lint did not run'\nprintf '%s\\n' 'owner verification required'\n} >> \"$GITHUB_STEP_SUMMARY\"",
    ] {
        assert!(workflow_uses_make_entrypoints(&workflow(run)).unwrap(), "accepted: {run}");
    }
    for run in [
        "cargo test",
        "make lint\ncargo +stable -q test --workspace",
        "env CARGO_NET_OFFLINE=true cargo clippy",
        "bash -c 'cargo nextest run'",
        "make lint\necho \"$(cargo test)\"",
        "make lint\nprintf '%s' `cargo clippy`",
        "{ cargo fmt; }",
        "echo 'make test'\npython3 check.py",
    ] {
        assert!(!workflow_uses_make_entrypoints(&workflow(run)).unwrap(), "rejected: {run}");
    }
    assert!(!workflow_uses_make_entrypoints("name: make test\njobs: {}\n").unwrap());
    assert!(workflow_uses_make_entrypoints("jobs: [").is_err());
}

#[test]
fn policy__contracts__ci_tools_policy__serde_yaml_is_scoped() {
    let root = workspace_root();
    let expected = [
        "bijux-dna",
        "bijux-dna-api",
        "bijux-dna-bench",
        "bijux-dna-containers",
        "bijux-dna-containers-qa",
        "bijux-dna-dev",
        "bijux-dna-domain-compiler",
        "bijux-dna-infra",
        "bijux-dna-policies",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    let development_only = ["bijux-dna-bench-population", "bijux-dna-bench-vcf"]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    let mut observed = BTreeSet::new();
    let mut observed_development = BTreeSet::new();
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if entry.file_name() != "Cargo.toml" {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read Cargo.toml");
        if !content.contains("serde_yaml") && !content.contains("serde-yaml") {
            continue;
        }
        let name = entry
            .path()
            .parent()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if development_only.contains(name) {
            let mut manifest: toml::Value = toml::from_str(&content).expect("parse Cargo.toml");
            let development = manifest
                .as_table_mut()
                .expect("manifest table")
                .remove("dev-dependencies")
                .expect("development-only YAML owner declares dev-dependencies");
            assert!(
                development.get("serde_yaml").is_some(),
                "{name} must keep YAML under dev-dependencies"
            );
            let production = manifest.to_string();
            assert!(
                !production.contains("serde_yaml") && !production.contains("serde-yaml"),
                "{name} must not introduce YAML into normal, build or target dependencies"
            );
            observed_development.insert(name.to_string());
        } else {
            observed.insert(name.to_string());
        }
    }
    assert_eq!(observed_development, development_only, "exact development-only YAML owners");
    bijux_dna_policies::policy_assert!(
        observed == expected,
        "serde_yaml owners must match the exact product boundary: expected {:?}, observed {:?}",
        expected,
        observed
    );
}

#[test]
fn policy__contracts__ci_tools_policy__coverage_command_policy_is_stable() {
    let root = workspace_root();
    let cargo_mk = root.join("makes").join("cargo.mk");
    let cargo_mk_content = std::fs::read_to_string(&cargo_mk).expect("read cargo.mk");
    let rust_gate = root
        .join(".bijux")
        .join("shared")
        .join("bijux-makes-rs")
        .join("scripts")
        .join("rust_gate.sh");
    let rust_gate_content = std::fs::read_to_string(&rust_gate).expect("read shared rust_gate.sh");

    let cargo_mk_required = [
        "coverage-workspace: ## Run the governed coverage control-plane lane.",
        "_coverage:",
        "NEXTEST_CONFIG=\"$(NEXTEST_CONFIG)\"",
        "TEST_FEATURES=\"$(TEST_FEATURES)\"",
        "NEXTEST_PROFILE=\"$(NEXTEST_PROFILE)\"",
        "NEXTEST_TEST_THREADS=\"$(NEXTEST_TEST_THREADS)\"",
        "RUN_IGNORED=\"$(RUN_IGNORED)\"",
        "COVERAGE_OUT=\"$(COVERAGE_OUT)\"",
        "COVERAGE_BASELINE=\"$(COVERAGE_BASELINE)\"",
        "COVERAGE_THRESHOLDS=\"$(COVERAGE_THRESHOLDS)\"",
        "cargo run -q -p bijux-dna-dev -- tooling run ci-coverage",
    ];
    let rust_gate_required = [
        "cargo llvm-cov nextest",
        "--workspace",
        "--all-features",
        "--run-ignored all",
        "--config-file \"${NEXTEST_CONFIG_FILE}\"",
        "--profile \"${NEXTEST_PROFILE_ALL:-default}\"",
        "cargo llvm-cov report --summary-only",
    ];

    let mut missing = Vec::new();
    for needle in cargo_mk_required {
        if !cargo_mk_content.contains(needle) {
            missing.push(format!("cargo.mk missing `{needle}`"));
        }
    }
    for needle in rust_gate_required {
        if !rust_gate_content.contains(needle) {
            missing.push(format!("rust_gate.sh missing `{needle}`"));
        }
    }
    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "coverage command policy drift: {missing:?}"
    );
}

#[test]
fn policy__contracts__ci_tools_policy__test_and_coverage_dirs_are_isolated() {
    let test_target = env::var("TEST_TARGET_DIR").unwrap_or_default();
    let cov_target = env::var("COV_TARGET_DIR").unwrap_or_default();
    let test_tmp = env::var("TEST_TMP_DIR").unwrap_or_default();
    let cov_tmp = env::var("COV_TMP_DIR").unwrap_or_default();
    let test_profraw = env::var("TEST_PROFRAW_DIR").unwrap_or_default();
    let cov_profraw = env::var("COV_PROFRAW_DIR").unwrap_or_default();

    let missing = [
        test_target.is_empty(),
        cov_target.is_empty(),
        test_tmp.is_empty(),
        cov_tmp.is_empty(),
        test_profraw.is_empty(),
        cov_profraw.is_empty(),
    ]
    .iter()
    .any(|missing| *missing);

    if missing {
        return;
    }

    bijux_dna_policies::policy_assert!(
        !test_target.is_empty() && !cov_target.is_empty(),
        "TEST_TARGET_DIR and COV_TARGET_DIR must be set."
    );
    bijux_dna_policies::policy_assert!(
        test_tmp != cov_tmp,
        "TEST_TMP_DIR and COV_TMP_DIR must be distinct."
    );
    bijux_dna_policies::policy_assert!(
        test_profraw != cov_profraw,
        "TEST_PROFRAW_DIR and COV_PROFRAW_DIR must be distinct."
    );
}

#[test]
fn policy__contracts__ci_tools_policy__no_bijux_namespace_in_docs_or_scripts() {
    let root = workspace_root();
    let scan_roots = [root.join("docs"), root.join("scripts"), root.join(".github")];
    let mut offenders = Vec::new();

    for scan_root in scan_roots {
        if !scan_root.exists() {
            continue;
        }
        for entry in WalkDir::new(scan_root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            if content.contains("bijux::") {
                offenders.push(entry.path().display().to_string());
            }
        }
    }

    offenders.sort();
    offenders.dedup();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Docs, automation, and CI must not reference legacy bijux:: namespace: {:?}",
        offenders
    );
}

#[test]
fn policy__contracts__ci_tools_policy__ci_environment_contract_is_stable() {
    if env::var_os("CI").is_none() {
        return;
    }
    let tz = env::var("TZ").unwrap_or_default();
    let lc_all = env::var("LC_ALL").unwrap_or_default();
    bijux_dna_policies::policy_assert!(
        tz == "UTC",
        "CI must set TZ=UTC for deterministic tests. Observed: {tz}"
    );
    bijux_dna_policies::policy_assert!(
        lc_all == "C",
        "CI must set LC_ALL=C for deterministic tests. Observed: {lc_all}"
    );
}
