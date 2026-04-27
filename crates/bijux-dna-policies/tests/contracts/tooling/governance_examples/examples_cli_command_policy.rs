#![allow(non_snake_case)]
use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn policy__contracts__examples_cli_command_policy__examples_use_existing_cli_commands_only() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let examples = root.join("examples");
    let allowed =
        ["run", "plan", "explain", "analyze", "bench", "env", "registry", "corpus", "status"];

    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&examples).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or_default();
        if !matches!(ext, "sh" | "md" | "txt") {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.contains("bijux ") && !trimmed.contains("bijux-dna ") {
                continue;
            }
            if trimmed.contains("bijux example")
                || trimmed.contains("bijux dna example")
                || trimmed.contains("bijux-dna example")
            {
                offenders.push(format!(
                    "{}: legacy example command is forbidden: {}",
                    path.display(),
                    trimmed
                ));
                continue;
            }
            if let Some(idx) = trimmed.find("bijux-dna ") {
                let tail = &trimmed[idx + "bijux-dna ".len()..];
                let verb = tail.split_whitespace().next().unwrap_or_default();
                if !allowed.contains(&verb) {
                    offenders.push(format!(
                        "{}: unknown/non-allowlisted verb `{}` in line: {}",
                        path.display(),
                        verb,
                        trimmed
                    ));
                }
            } else if let Some(idx) = trimmed.find("bijux dna ") {
                let tail = &trimmed[idx + "bijux dna ".len()..];
                let verb = tail.split_whitespace().next().unwrap_or_default();
                if !allowed.contains(&verb) {
                    offenders.push(format!(
                        "{}: unknown/non-allowlisted verb `{}` in line: {}",
                        path.display(),
                        verb,
                        trimmed
                    ));
                }
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "examples must use existing CLI commands only:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__examples_cli_command_policy__recipe_only_dirs_are_allowlisted_and_readme_only(
) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let allowlist = root.join("examples/RECIPE_ONLY.txt");
    let allowlisted = std::fs::read_to_string(&allowlist)
        .unwrap_or_else(|_| panic!("read {}", allowlist.display()))
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect::<BTreeSet<_>>();

    let mut observed = BTreeSet::new();
    let mut offenders = Vec::new();
    for domain in ["fastq", "vcf"] {
        let domain_root = root.join("examples").join(domain);
        for entry in std::fs::read_dir(&domain_root)
            .unwrap_or_else(|_| panic!("read {}", domain_root.display()))
            .flatten()
        {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .unwrap_or_else(|_| panic!("strip prefix {}", path.display()))
                .display()
                .to_string();
            let readme = path.join("README.md");
            let example_toml = path.join("example.toml");
            if !readme.is_file() || example_toml.is_file() {
                continue;
            }
            observed.insert(rel.clone());
            if !allowlisted.contains(&rel) {
                offenders.push(format!("{rel}: missing examples/RECIPE_ONLY.txt entry"));
            }
            let entries = std::fs::read_dir(&path)
                .unwrap_or_else(|_| panic!("read {}", path.display()))
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect::<BTreeSet<_>>();
            let expected = BTreeSet::from(["README.md".to_string()]);
            if entries != expected {
                offenders.push(format!(
                    "{rel}: recipe-only directories must stay README-only (found {entries:?})"
                ));
            }
        }
    }

    for rel in allowlisted.difference(&observed) {
        offenders.push(format!("{rel}: listed in examples/RECIPE_ONLY.txt but not observed"));
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "recipe-only example directory policy violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__examples_cli_command_policy__example_navigation_docs_use_index_as_ssot() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let index =
        std::fs::read_to_string(root.join("examples/index.yaml")).expect("read examples index");
    let example_ids = index
        .lines()
        .filter_map(|line| line.trim().strip_prefix("- id: ").map(str::to_string))
        .collect::<BTreeSet<_>>();

    let mut offenders = Vec::new();
    for rel in ["examples/README.md", "docs/50-reference/EXAMPLES.md"] {
        let path = root.join(rel);
        let raw =
            std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("read {}", path.display()));
        if !raw.contains("examples/index.yaml") {
            offenders.push(format!("{rel}: must point to examples/index.yaml as runnable SSOT"));
        }
        for forbidden in [
            "`template`",
            "`data_corpus_01`",
            "`data_corpus_01_mini`",
            "`corpus_01`",
            "`corpus_01_mini`",
        ] {
            if raw.contains(forbidden) {
                offenders.push(format!(
                    "{rel}: must not advertise non-runnable example ids such as {forbidden}"
                ));
            }
        }
        for token in raw.split('`').skip(1).step_by(2) {
            if (token.starts_with("fastq_") || token.starts_with("vcf_"))
                && !example_ids.contains(token)
            {
                offenders.push(format!(
                    "{rel}: references runnable example id `{token}` not present in examples/index.yaml"
                ));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "example navigation doc SSOT violations:\n{}",
        offenders.join("\n")
    );
}
