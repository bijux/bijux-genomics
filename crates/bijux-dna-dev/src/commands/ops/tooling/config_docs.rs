use super::{
    artifact_root_path, config_snapshot_inputs_changed, config_tree_snapshot_text,
    ensure_help_only, failure_lines, fs, generate_tool_index, merge_outcomes, read_utf8,
    resolve_optional_output_arg, resolve_workspace_path, run_check_ids, run_native_ops_command,
    run_program, sha256_hex, success_line, tooling_cargo_targets, tooling_ci_clippy_executors,
    tooling_ci_fmt, write_utf8, BTreeSet, Context, NativeOpsCommandKey, OpsCommandOutcome, Regex,
    Result, WalkDir, Workspace,
};

pub(in super::super) fn tooling_flake_hunt(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let mut expr = None;
    let mut runs = 20usize;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--expr" => {
                expr = args.get(index + 1).cloned();
                index += 2;
            }
            "--runs" => {
                runs = args
                    .get(index + 1)
                    .context("missing value for --runs")?
                    .parse::<usize>()
                    .context("parse --runs")?;
                index += 2;
            }
            "--help" | "-h" => {
                return success_line(
                    "Usage: cargo run -p bijux-dna-dev -- tooling run flake-hunt -- --expr '<nextest-filter>' [--runs N]",
                )
            }
            other => return Ok(OpsCommandOutcome::failure(format!("unknown arg: {other}\n"))),
        }
    }
    let expr = expr.context("--expr is required")?;
    let log_dir = artifact_root_path(workspace)?.join("flake-hunt");
    bijux_dna_infra::ensure_dir(&log_dir)
        .with_context(|| format!("create {}", log_dir.display()))?;
    let mut stdout = String::new();
    let mut failures = 0usize;
    for run_index in 1..=runs {
        stdout.push_str(&format!("[{run_index}/{runs}] nextest run -E {expr}\n"));
        let outcome = tooling_cargo_targets(
            workspace,
            &[
                "nextest-run".to_string(),
                "--config-file".to_string(),
                "configs/rust/nextest.toml".to_string(),
                "--profile".to_string(),
                "flake".to_string(),
                "-E".to_string(),
                expr.clone(),
            ],
        )?;
        bijux_dna_infra::write_bytes(
            log_dir.join("last.log"),
            format!("{}{}", outcome.stdout, outcome.stderr),
        )
        .with_context(|| format!("write {}", log_dir.join("last.log").display()))?;
        if outcome.is_success() {
            stdout.push_str("  PASS\n");
        } else {
            failures += 1;
            stdout.push_str("  FAIL\n");
            stdout.push_str(&outcome.stdout);
            stdout.push_str(&outcome.stderr);
        }
    }
    stdout.push_str(&format!(
        "Expression: {expr}\nRuns: {runs}\nPassed: {}\nFailed: {failures}\n",
        runs - failures
    ));
    if failures == 0 {
        return Ok(OpsCommandOutcome::success(stdout));
    }
    Ok(OpsCommandOutcome::failure(stdout))
}

pub(in super::super) fn tooling_lint_fast(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("lint-fast", args)?;
    let base_ref = std::env::var("LINT_FAST_BASE_REF")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            let head_prev = run_program(
                workspace,
                "git",
                &["rev-parse".to_string(), "--verify".to_string(), "HEAD~1".to_string()],
            );
            match head_prev {
                Ok(outcome) if outcome.is_success() => "HEAD~1".to_string(),
                _ => "HEAD".to_string(),
            }
        });
    let diff = run_program(
        workspace,
        "git",
        &["diff".to_string(), "--name-only".to_string(), format!("{base_ref}..HEAD")],
    )?;
    let changed = diff.stdout.lines().filter(|line| !line.trim().is_empty()).collect::<Vec<_>>();
    let mut stdout = String::new();
    if changed.is_empty() {
        run_check_ids(&mut stdout, &["check-config-schema", "check-automation-interface"])?;
        stdout.push_str("lint-fast: no changed files; running config+automation lint baseline\n");
        return Ok(OpsCommandOutcome::success(stdout));
    }
    let mut need_fmt = false;
    let mut need_clippy = false;
    let mut need_docs = false;
    let mut need_configs = false;
    let mut need_automation = false;
    for file in &changed {
        if file.ends_with(".rs")
            || *file == "Cargo.toml"
            || *file == "Cargo.lock"
            || file.starts_with("crates/")
        {
            need_fmt = true;
            need_clippy = true;
        }
        if file.starts_with("docs/") || file.ends_with("README.md") {
            need_docs = true;
        }
        if file.starts_with("configs/") || file.starts_with("assets/reference/") {
            need_configs = true;
        }
        if file.starts_with("makes/") || *file == "Makefile" {
            need_automation = true;
        }
    }
    if need_fmt {
        stdout.push_str("lint-fast: running rustfmt\n");
        let outcome = tooling_ci_fmt(workspace, &[])?;
        if !outcome.is_success() {
            return Ok(merge_outcomes(OpsCommandOutcome::success(stdout), outcome));
        }
    }
    if need_clippy {
        stdout.push_str("lint-fast: running clippy for executor/runtime subset\n");
        let outcome = tooling_ci_clippy_executors(workspace, &[])?;
        if !outcome.is_success() {
            return Ok(merge_outcomes(OpsCommandOutcome::success(stdout), outcome));
        }
    }
    if need_docs {
        stdout.push_str("lint-fast: running docs checks\n");
        let docs_outcome =
            run_native_ops_command(NativeOpsCommandKey::DocsCheckDocLinks, workspace, &[])?;
        if !docs_outcome.is_success() {
            return Ok(merge_outcomes(OpsCommandOutcome::success(stdout), docs_outcome));
        }
        stdout.push_str(&docs_outcome.stdout);
        run_check_ids(&mut stdout, &["check-docs-build-contract"])?;
    }
    if need_configs {
        stdout.push_str("lint-fast: running config checks\n");
        run_check_ids(&mut stdout, &["check-config-schema", "check-config-layout"])?;
    }
    if need_automation {
        stdout.push_str("lint-fast: running automation interface checks\n");
        run_check_ids(
            &mut stdout,
            &[
                "check-automation-interface",
                "check-clippy-allowlist-growth",
                "check-rustflags-consistency",
            ],
        )?;
    }
    stdout.push_str("lint-fast: OK\n");
    Ok(OpsCommandOutcome::success(stdout))
}

pub(in super::super) fn tooling_generate_tool_index(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let out = resolve_optional_output_arg(
        workspace,
        "generate-tool-index",
        args,
        "docs/20-science/TOOL_INDEX.md",
    )?;
    generate_tool_index(workspace, &out)?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(in super::super) fn tooling_check_config_snapshot(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let only_if_changed = match args {
        [] => false,
        [flag] if flag == "--if-config-changed" => true,
        [flag] if flag == "--help" || flag == "-h" => {
            return success_line(
                "Usage: cargo run -p bijux-dna-dev -- tooling run check-config-snapshot -- [--if-config-changed]",
            )
        }
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run check-config-snapshot -- [--if-config-changed]\n",
            ))
        }
    };

    if only_if_changed && !config_snapshot_inputs_changed(workspace)? {
        return success_line("config snapshot: SKIP (no config or generator changes)");
    }

    let baseline = workspace.path("configs/schema/config_tree.snapshot");
    let actual = workspace.path("artifacts/tmp/config_tree.snapshot.actual");
    if let Some(parent) = actual.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    write_utf8(&actual, &config_tree_snapshot_text(workspace)?)?;

    if read_utf8(&baseline)? != read_utf8(&actual)? {
        return Ok(OpsCommandOutcome::failure(
            "config snapshot drift detected; regenerate via cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot\n",
        ));
    }
    if !read_utf8(&baseline)?
        .lines()
        .any(|line| {
            line.trim()
                == "# generator = cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot"
        })
    {
        return Ok(OpsCommandOutcome::failure(
            "config snapshot header missing generator contract\n",
        ));
    }
    let marker_file = workspace.path("artifacts/configs/config_tree_snapshot.marker");
    if marker_file.is_file() {
        let marker = read_utf8(&marker_file)?;
        let marker_sha = marker
            .lines()
            .find_map(|line| line.strip_prefix("snapshot_sha256="))
            .unwrap_or_default()
            .trim()
            .to_string();
        let actual_sha = sha256_hex(&baseline)?;
        if marker_sha.is_empty() || marker_sha != actual_sha {
            return Ok(OpsCommandOutcome::failure(
                "config snapshot marker is stale: run cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot\n",
            ));
        }
    }
    success_line("config snapshot: OK")
}

pub(in super::super) fn tooling_generate_config_tree_snapshot(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("generate-config-tree-snapshot", args)?;
    let out = workspace.path("configs/schema/config_tree.snapshot");
    let marker_dir = workspace.path("artifacts/configs");
    let marker_file = marker_dir.join("config_tree_snapshot.marker");
    bijux_dna_infra::ensure_dir(&marker_dir)
        .with_context(|| format!("create {}", marker_dir.display()))?;
    if let Some(parent) = out.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    write_utf8(&out, &config_tree_snapshot_text(workspace)?)?;
    write_utf8(
        &marker_file,
        &format!(
            "generator=cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot\nsnapshot_sha256={}\n",
            sha256_hex(&out)?
        ),
    )?;
    success_line(format!("generated {}", workspace.rel(&out).display()))
}

pub(in super::super) fn tooling_check_config_paths(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-config-paths", args)?;
    let refs = repository_config_references(workspace)?;
    let allow_missing = BTreeSet::from([
        "configs/runtime/profiles/hpc.toml",
        "configs/tools.toml",
        "configs/lab/config.toml",
    ]);
    let missing = refs
        .into_iter()
        .filter(|rel| !allow_missing.contains(rel.as_str()) && !workspace.path(rel).exists())
        .map(|rel| format!("missing config reference: {rel}"))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return success_line("config path references: OK");
    }
    failure_lines("config path references: FAILED", &missing)
}

fn repository_config_references(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let pattern = Regex::new(r"(?:benchmarks/)?configs/[A-Za-z0-9_./-]+\.(toml|md|sha256)")?;
    let mut refs = BTreeSet::new();
    let mut scan_roots = vec![workspace.path("Makefile")];
    scan_roots.extend([
        workspace.path("makes"),
        workspace.path("crates"),
        workspace.path("docs"),
        workspace.path(".github"),
    ]);
    for root in scan_roots {
        if root.is_file() {
            collect_config_file_references(workspace, &root, &pattern, &mut refs)?;
            continue;
        }
        if !root.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&root) {
            let entry = entry.with_context(|| format!("scan {}", root.display()))?;
            if !entry.file_type().is_file() {
                continue;
            }
            collect_config_file_references(workspace, entry.path(), &pattern, &mut refs)?;
        }
    }
    Ok(refs)
}

fn collect_config_file_references(
    workspace: &Workspace,
    path: &std::path::Path,
    pattern: &Regex,
    refs: &mut BTreeSet<String>,
) -> Result<()> {
    let rel = workspace.rel(path);
    let components = rel.components().map(std::path::Component::as_os_str).collect::<Vec<_>>();
    if components.first().is_some_and(|part| *part == "crates")
        && components.iter().skip(2).any(|part| *part == "tests")
        || components.first().is_some_and(|part| *part == "makes")
            && components.get(1).is_some_and(|part| *part == "tests")
    {
        return Ok(());
    }
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let raw = match String::from_utf8(bytes) {
        Ok(raw) => raw,
        Err(error)
            if matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("rs" | "md" | "sh" | "mk" | "py" | "toml" | "yaml" | "yml" | "json")
            ) =>
        {
            return Err(error).with_context(|| format!("decode {}", path.display()));
        }
        Err(_) => return Ok(()),
    };
    if path.extension().is_some_and(|extension| extension == "rs") {
        let mut visitor = ConfigReferenceVisitor { pattern, refs, error: None };
        match syn::parse_file(&raw) {
            Ok(source) => {
                if !visitor.test_only(&source.attrs) {
                    syn::visit::Visit::visit_file(&mut visitor, &source);
                }
            }
            Err(file_error) => {
                // include! also consumes complete expressions, including block fragments.
                let expression = syn::parse_str::<syn::Expr>(&raw).map_err(|expression_error| {
                    anyhow::anyhow!(
                        "parse production Rust {} as file ({file_error}) or expression ({expression_error})",
                        path.display()
                    )
                })?;
                syn::visit::Visit::visit_expr(&mut visitor, &expression);
            }
        }
        if let Some(error) = visitor.error {
            return Err(error)
                .with_context(|| format!("inspect Rust attributes {}", path.display()));
        }
    } else {
        refs.extend(pattern.find_iter(&raw).map(|capture| capture.as_str().to_string()));
    }
    Ok(())
}

struct ConfigReferenceVisitor<'a> {
    pattern: &'a Regex,
    refs: &'a mut BTreeSet<String>,
    error: Option<anyhow::Error>,
}

impl ConfigReferenceVisitor<'_> {
    fn test_only(&mut self, attrs: &[syn::Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("test") {
                return true;
            }
            if attr.path().is_ident("cfg") {
                match attr.parse_args::<syn::Meta>().and_then(|meta| cfg_requires_test(&meta)) {
                    Ok(true) => return true,
                    Ok(false) => {}
                    Err(error) => self.error = Some(error.into()),
                }
            }
        }
        false
    }
}

fn cfg_requires_test(meta: &syn::Meta) -> syn::Result<bool> {
    match meta {
        syn::Meta::Path(path) => Ok(path.is_ident("test")),
        syn::Meta::List(list) if list.path.is_ident("all") || list.path.is_ident("any") => {
            let conditions = list.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
            )?;
            let requirements =
                conditions.iter().map(cfg_requires_test).collect::<syn::Result<Vec<_>>>()?;
            Ok(if list.path.is_ident("all") {
                requirements.contains(&true)
            } else {
                !requirements.is_empty() && requirements.iter().all(|required| *required)
            })
        }
        _ => Ok(false),
    }
}

impl<'ast> syn::visit::Visit<'ast> for ConfigReferenceVisitor<'_> {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let attrs: &[syn::Attribute] = match item {
            syn::Item::Const(item) => &item.attrs,
            syn::Item::Enum(item) => &item.attrs,
            syn::Item::ExternCrate(item) => &item.attrs,
            syn::Item::Fn(item) => &item.attrs,
            syn::Item::ForeignMod(item) => &item.attrs,
            syn::Item::Impl(item) => &item.attrs,
            syn::Item::Macro(item) => &item.attrs,
            syn::Item::Mod(item) => &item.attrs,
            syn::Item::Static(item) => &item.attrs,
            syn::Item::Struct(item) => &item.attrs,
            syn::Item::Trait(item) => &item.attrs,
            syn::Item::TraitAlias(item) => &item.attrs,
            syn::Item::Type(item) => &item.attrs,
            syn::Item::Union(item) => &item.attrs,
            syn::Item::Use(item) => &item.attrs,
            _ => &[],
        };
        if !self.test_only(attrs) {
            syn::visit::visit_item(self, item);
        }
    }

    fn visit_lit_str(&mut self, literal: &'ast syn::LitStr) {
        self.refs.extend(
            self.pattern.find_iter(&literal.value()).map(|capture| capture.as_str().to_string()),
        );
    }

    fn visit_macro(&mut self, invocation: &'ast syn::Macro) {
        self.refs.extend(
            self.pattern
                .find_iter(&invocation.tokens.to_string())
                .map(|capture| capture.as_str().to_string()),
        );
    }
}

pub(in super::super) fn tooling_clean_docs(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let docs_root = match args {
        [] => workspace.path("artifacts/docs"),
        [flag] if flag == "--help" || flag == "-h" => {
            return success_line(
                "Usage: cargo run -p bijux-dna-dev -- tooling run clean-docs -- [artifacts/docs-root]",
            )
        }
        [path] => resolve_workspace_path(workspace, path),
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run clean-docs -- [artifacts/docs-root]\n",
            ))
        }
    };
    let docs_root_rel = workspace.rel(&docs_root).to_string_lossy().to_string();
    if !docs_root_rel.starts_with("artifacts/") {
        return Ok(OpsCommandOutcome::failure(
            "clean-docs refuses to remove paths outside artifacts/\n",
        ));
    }
    if docs_root.exists() {
        fs::remove_dir_all(&docs_root)
            .with_context(|| format!("remove {}", docs_root.display()))?;
    }
    success_line(format!("removed {}", docs_root.display()))
}

#[cfg(test)]
mod config_reference_tests {
    use super::*;

    fn scan(files: &[(&str, &str)]) -> Result<BTreeSet<String>> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        for (rel, text) in files {
            let path = workspace.path(rel);
            fs::create_dir_all(path.parent().context("fixture parent")?)?;
            fs::write(path, text)?;
        }
        repository_config_references(&workspace)
    }

    #[test]
    fn production_config_references_include_literals_macros_and_documentation() -> Result<()> {
        let refs = scan(&[(
            "crates/example/src/lib.rs",
            r##"
                /// Read configs/documented.toml before using this API.
                const PATH: &str = "configs/production.toml";
                const RAW: &str = r#"benchmarks/configs/raw.toml"#;
                const CONTENTS: &str = include_str!("configs/included.toml");
                fn read() { custom_macro!("configs/macro.toml"); }
            "##,
        )])?;
        assert_eq!(
            refs,
            BTreeSet::from([
                "configs/documented.toml".into(),
                "configs/production.toml".into(),
                "benchmarks/configs/raw.toml".into(),
                "configs/included.toml".into(),
                "configs/macro.toml".into(),
            ])
        );
        Ok(())
    }

    #[test]
    fn expression_include_fragments_retain_required_config_paths() -> Result<()> {
        let refs = scan(&[(
            "crates/example/src/execution_outputs.rs",
            r#"{
                let path = "configs/expression.toml";
                consume(include_str!("configs/fragment.toml"));
                path
            }"#,
        )])?;
        assert_eq!(
            refs,
            BTreeSet::from(["configs/expression.toml".into(), "configs/fragment.toml".into()])
        );
        assert!(scan(&[("crates/example/src/execution_outputs.rs", "{ let path =")]).is_err());
        Ok(())
    }

    #[test]
    fn rust_test_boundaries_do_not_hide_following_production_items() -> Result<()> {
        let refs = scan(&[(
            "crates/example/src/lib.rs",
            r##"
                #[cfg(test)] mod fixtures {
                    const PATH: &str = r#"configs/fixture.toml } #[cfg(test)]"#;
                    mod nested { const PATH: &str = "configs/nested-fixture.toml"; }
                }
                #[test] fn direct_test() { consume("configs/test.toml"); }
                #[cfg(all(test, feature = "extra"))]
                const TEST: &str = "configs/test-feature.toml";
                #[cfg(any(test, feature = "extra"))]
                const SHARED: &str = "configs/feature.toml";
                const AFTER: &str = "configs/after.toml";
            "##,
        )])?;
        assert_eq!(
            refs,
            BTreeSet::from(["configs/feature.toml".into(), "configs/after.toml".into()])
        );
        assert!(scan(&[(
            "crates/example/src/testing.rs",
            "#![cfg(test)]\nconst PATH: &str = \"configs/fixture.toml\";"
        )])?
        .is_empty());
        Ok(())
    }

    #[test]
    fn fixture_roots_are_separate_from_governed_make_and_docs_references() -> Result<()> {
        let refs = scan(&[
            (
                "crates/example/tests/contracts.rs",
                "fn fixture() { use_path(\"configs/fixture.toml\"); }",
            ),
            ("crates/example/tests/fixtures/input.md", "configs/fixture-data.toml"),
            ("makes/tests/example.py", "value = 'configs/fixture.toml'"),
            ("crates/example/examples/main.rs", "fn main() { read(\"configs/example.toml\"); }"),
            ("Makefile", "CONFIG = configs/make.toml"),
            ("docs/configuration.md", "Read `configs/documented.toml`."),
            (".github/workflows/ci.yml", "config: configs/workflow.toml"),
        ])?;
        assert_eq!(
            refs,
            BTreeSet::from([
                "configs/example.toml".into(),
                "configs/make.toml".into(),
                "configs/documented.toml".into(),
                "configs/workflow.toml".into(),
            ])
        );
        Ok(())
    }

    #[test]
    fn missing_production_reference_fails_while_test_fixture_remains_private() -> Result<()> {
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let source = workspace.path("crates/example/src/lib.rs");
        fs::create_dir_all(source.parent().context("source parent")?)?;
        fs::write(&source, "const PATH: &str = \"configs/production.toml\";\n#[cfg(test)] mod tests { const PATH: &str = \"configs/missing.toml\"; }")?;
        let failed = tooling_check_config_paths(&workspace, &[])?;
        assert!(!failed.is_success());
        assert!(failed.stderr.contains("missing config reference: configs/production.toml"));
        assert!(!failed.stderr.contains("configs/missing.toml"));
        fs::create_dir(workspace.path("configs"))?;
        fs::write(workspace.path("configs/production.toml"), "enabled = true")?;
        assert!(tooling_check_config_paths(&workspace, &[])?.is_success());
        Ok(())
    }

    #[test]
    fn malformed_or_unreadable_production_source_fails_closed() -> Result<()> {
        assert!(scan(&[("crates/example/src/lib.rs", "fn broken( {")]).is_err());
        let root = tempfile::tempdir()?;
        let workspace = Workspace { root: root.path().to_path_buf() };
        let pattern = Regex::new("configs/[a-z]+[.]toml")?;
        let mut refs = BTreeSet::new();
        assert!(collect_config_file_references(
            &workspace,
            &workspace.path("missing.rs"),
            &pattern,
            &mut refs
        )
        .is_err());
        fs::write(workspace.path("invalid.rs"), [0xff])?;
        assert!(collect_config_file_references(
            &workspace,
            &workspace.path("invalid.rs"),
            &pattern,
            &mut refs
        )
        .is_err());
        Ok(())
    }
}
