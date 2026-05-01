use super::{
    ensure_generated_header, ensure_generated_header_path, ensure_help_only, failure_lines, fs,
    generate_compatibility_matrix, generate_compatibility_reference_docs, generate_docs_graph,
    generate_domain_coverage_doc, generate_repo_root_map, generate_tool_index, glob_paths,
    read_utf8, rg_lines, run_program, success_line, temp_subdir, trim_quoted, write_utf8, BTreeMap,
    BTreeSet, Context, OpsCommandOutcome, Regex, Result, WalkDir, Workspace,
};
use std::path::Path;

fn docs_graph_exempt(rel: &str) -> bool {
    rel.starts_with("docs/30-operations/benchmark/")
        || matches!(
            rel,
            "docs/30-operations/benchmark"
                | "docs/30-operations/corpus-01.md"
                | "docs/badges.md"
                | "docs/containers/publish-ready-metadata.md"
        )
}

pub(super) fn docs_check_doc_assets(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-assets", args)?;
    let docs_root = workspace.path("docs");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(&docs_root).into_iter().filter_map(std::result::Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
        let lower = rel.to_ascii_lowercase();
        let is_image = [".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp"]
            .iter()
            .any(|ext| lower.ends_with(ext));
        if is_image && !rel.starts_with("docs/assets/") {
            offenders.push(rel);
        }
    }
    if offenders.is_empty() {
        return success_line("doc-assets: OK");
    }
    failure_lines("doc-assets: images must live under docs/assets/", &offenders)
}

pub(super) fn docs_check_doc_depth(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-depth", args)?;
    let docs_root = workspace.path("docs");
    let purpose = Regex::new(r"(?mi)^##\s+(Purpose|What)\s*$")?;
    let scope = Regex::new(r"(?mi)^##\s+(Scope|Why)\s*$")?;
    let non_goals = Regex::new(r"(?mi)^##\s+Non-goals\s*$")?;
    let contracts = Regex::new(r"(?mi)^##\s+Contracts\s*$")?;
    let mut violations = Vec::new();
    for entry in WalkDir::new(&docs_root).into_iter().filter_map(std::result::Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let rel = workspace.rel(path).to_string_lossy().to_string();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if matches!(
            name,
            "index.md" | "command_snapshot.txt" | "release_help_snapshot.txt" | "DOCS_GRAPH.toml"
        ) || name.ends_with(".generated.md")
            || rel == "docs/cli/index.md"
            || rel.starts_with("docs/30-operations/benchmark/")
        {
            continue;
        }
        let raw = read_utf8(path)?;
        let mut missing = Vec::new();
        if !purpose.is_match(&raw) {
            missing.push("purpose");
        }
        if !scope.is_match(&raw) {
            missing.push("scope");
        }
        if !non_goals.is_match(&raw) {
            missing.push("non-goals");
        }
        if !contracts.is_match(&raw) {
            missing.push("contracts");
        }
        if !missing.is_empty() {
            violations.push(format!("{rel}: {}", missing.join(", ")));
        }
    }
    if violations.is_empty() {
        return success_line("doc-depth: OK");
    }
    failure_lines("doc-depth: missing required sections", &violations)
}

pub(super) fn docs_check_doc_links(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-links", args)?;
    let docs_root = workspace.path("docs");
    let link_re = Regex::new(r"\[[^\]]*\]\(([^)]+)\)")?;
    let mut missing = Vec::new();
    let mut publication = Vec::new();
    for entry in WalkDir::new(&docs_root).into_iter().filter_map(std::result::Result::ok) {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("md")
        {
            continue;
        }
        let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
        let raw = read_utf8(entry.path())?;
        for capture in link_re.captures_iter(&raw) {
            let Some(target) = capture.get(1).map(|value| value.as_str().trim()) else {
                continue;
            };
            if target.is_empty()
                || target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
                || target.starts_with('#')
            {
                continue;
            }
            let target = target.split('#').next().unwrap_or_default();
            if target.is_empty() {
                continue;
            }
            let candidate = if target.starts_with('/') {
                workspace.root.join(target.trim_start_matches('/'))
            } else {
                entry
                    .path()
                    .parent()
                    .map_or_else(|| workspace.root.join(target), |parent| parent.join(target))
            };
            if !candidate.exists() {
                missing.push(format!("{rel} -> {target}"));
            }
            if target.contains("assets/publications/")
                && !target.split('#').next().unwrap_or_default().ends_with("/index.md")
            {
                publication.push(format!(
                    "{rel} -> {target} (must link to assets/publications/<pub-id>/index.md)"
                ));
            }
        }
    }
    for target in
        ["make _ci-fast", "make _ci-slow", "make _quick", "make policy-fast", "make policy-full"]
    {
        let matches = rg_lines(workspace, "docs", target)?;
        missing.extend(
            matches
                .into_iter()
                .map(|line| format!("stale make target reference `{target}`: {line}")),
        );
    }
    if missing.is_empty() && publication.is_empty() {
        return success_line("docs links: OK");
    }
    let mut errors = Vec::new();
    errors.extend(missing);
    errors.extend(publication);
    failure_lines("docs link check failed:", &errors)
}

pub(super) fn docs_check_doc_root_layout(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-root-layout", args)?;
    let allowed_dirs = BTreeSet::from([
        "00-intro",
        "10-architecture",
        "20-science",
        "30-operations",
        "40-policies",
        "50-reference",
        "assets",
        "cli",
        "containers",
        "decisions",
        "overrides",
    ]);
    let required_dirs = BTreeSet::from([
        "00-intro",
        "10-architecture",
        "20-science",
        "30-operations",
        "40-policies",
        "50-reference",
        "assets",
    ]);
    let docs_root = workspace.path("docs");
    let mut violations = Vec::new();
    for entry in
        fs::read_dir(&docs_root).with_context(|| format!("read {}", docs_root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let base =
            path.file_name().and_then(|value| value.to_str()).unwrap_or_default().to_string();
        if path.is_dir() {
            if !allowed_dirs.contains(base.as_str()) {
                violations.push(format!("unsupported docs root directory: docs/{base}"));
            }
        } else if path.is_file()
            && base != "index.md"
            && base != "DOCS_GRAPH.toml"
            && base != "badges.md"
        {
            violations.push(format!("unsupported docs root file: docs/{base}"));
        }
    }
    for required in required_dirs {
        if !docs_root.join(required).is_dir() {
            violations.push(format!("missing required docs root directory: docs/{required}"));
        }
    }
    if violations.is_empty() {
        return success_line("doc-root-layout: OK");
    }
    failure_lines("doc-root-layout: FAILED", &violations)
}

pub(super) fn docs_check_docs_graph(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-docs-graph", args)?;
    let docs_root = workspace.path("docs");
    let graph_path = docs_root.join("DOCS_GRAPH.toml");
    if !graph_path.is_file() {
        return Ok(OpsCommandOutcome::failure("docs-graph: missing docs/DOCS_GRAPH.toml\n"));
    }
    let graph = read_utf8(&graph_path)?;
    let mut edges = BTreeMap::<String, Vec<String>>::new();
    let mut graph_nodes = BTreeSet::new();
    let mut current = None::<String>;
    let mut in_children = false;
    for raw in graph.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[edge]]" {
            current = None;
            in_children = false;
            continue;
        }
        if let Some(value) = line.strip_prefix("from = ") {
            let from = trim_quoted(value);
            graph_nodes.insert(from.clone());
            current = Some(from);
            continue;
        }
        if line == "children = [" {
            in_children = true;
            continue;
        }
        if in_children {
            if line == "]" {
                in_children = false;
                continue;
            }
            let child = trim_quoted(line.trim_end_matches(','));
            if let Some(from) = &current {
                edges.entry(from.clone()).or_default().push(child.clone());
                graph_nodes.insert(child);
            }
        }
    }
    let mut errors = Vec::new();
    if !edges.contains_key("docs/index.md") {
        errors.push("docs/index.md missing from graph roots".to_string());
    }
    for node in graph_nodes {
        if !workspace.path(&node).exists() {
            errors.push(format!("missing graph node target: {node}"));
        }
    }
    let link_re = Regex::new(r"\[[^\]]*\]\(([^)]+)\)")?;
    for entry in WalkDir::new(&docs_root).into_iter().filter_map(std::result::Result::ok) {
        if !entry.file_type().is_file()
            || entry.path().extension().and_then(|ext| ext.to_str()) != Some("md")
        {
            continue;
        }
        let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
        let raw = read_utf8(entry.path())?;
        for capture in link_re.captures_iter(&raw) {
            let Some(target) = capture.get(1).map(|value| value.as_str().trim()) else {
                continue;
            };
            if target.is_empty()
                || target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
                || target.starts_with('#')
            {
                continue;
            }
            let target = target.split('#').next().unwrap_or_default();
            if target.is_empty() {
                continue;
            }
            let candidate = if target.starts_with('/') {
                workspace.root.join(target.trim_start_matches('/'))
            } else {
                entry
                    .path()
                    .parent()
                    .map_or_else(|| workspace.root.join(target), |parent| parent.join(target))
            };
            if !candidate.exists() {
                errors.push(format!("{rel} -> {target}"));
            }
        }
    }
    for dir in std::iter::once(docs_root.clone()).chain(
        WalkDir::new(&docs_root)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_dir())
            .map(|entry| entry.path().to_path_buf()),
    ) {
        let markdowns = fs::read_dir(&dir)
            .ok()
            .into_iter()
            .flat_map(|entries| entries.filter_map(std::result::Result::ok))
            .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("md"))
            .count();
        let rel_dir = workspace.rel(&dir).to_string_lossy().to_string();
        if markdowns > 0 && !dir.join("index.md").exists() && !docs_graph_exempt(&rel_dir) {
            errors
                .push(format!("section folder lacks index.md: {}", workspace.rel(&dir).display()));
        }
    }
    let all_docs = WalkDir::new(&docs_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().and_then(|ext| ext.to_str()) == Some("md")
        })
        .map(|entry| workspace.rel(entry.path()).to_string_lossy().to_string())
        .filter(|rel| rel != "docs/DOCS_GRAPH.toml")
        .filter(|rel| !docs_graph_exempt(rel))
        .collect::<BTreeSet<_>>();
    let mut reachable = BTreeSet::new();
    let mut queue = vec!["docs/index.md".to_string()];
    while let Some(node) = queue.pop() {
        if !reachable.insert(node.clone()) {
            continue;
        }
        if let Some(children) = edges.get(&node) {
            queue.extend(children.iter().cloned());
        }
    }
    for rel in all_docs.difference(&reachable) {
        errors
            .push(format!("docs not reachable from docs/index.md via docs/DOCS_GRAPH.toml: {rel}"));
    }
    if errors.is_empty() {
        return success_line("docs-graph: OK");
    }
    failure_lines("docs-graph: FAILED", &errors)
}

pub(super) fn docs_check_domain_doc_references(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-domain-doc-references", args)?;
    let stage_id_re = Regex::new(r#"^\s*id\s*=\s*"([^"]+)""#)?;
    let domain_stage_re = Regex::new(r"^\s*-\s*((?:fastq|bam|vcf)\.[a-z0-9_]+)")?;
    let tool_id_re = Regex::new(r#"^\s*(?:id|tool_id)\s*=\s*"([^"]+)""#)?;
    let docs_stage_re = Regex::new(r"`((?:fastq|bam)\.[a-z0-9_]+)`")?;
    let docs_tool_re = Regex::new(r"`tool:([a-z0-9][a-z0-9._-]*)`")?;
    let mut stage_ids = BTreeSet::new();
    for rel in ["configs/ci/stages/stages.toml", "configs/ci/stages/stages_vcf.toml"] {
        for line in read_utf8(&workspace.path(rel))?.lines() {
            if let Some(capture) = stage_id_re.captures(line) {
                if let Some(value) = capture.get(1) {
                    stage_ids.insert(value.as_str().to_string());
                }
            }
        }
    }
    for domain_index in glob_paths(workspace, "domain/*/index.yaml")? {
        for line in read_utf8(&domain_index)?.lines() {
            if let Some(capture) = domain_stage_re.captures(line) {
                if let Some(value) = capture.get(1) {
                    stage_ids.insert(value.as_str().to_string());
                }
            }
        }
    }
    let mut tool_ids = BTreeSet::new();
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
    ] {
        for line in read_utf8(&workspace.path(rel))?.lines() {
            if let Some(capture) = tool_id_re.captures(line) {
                if let Some(value) = capture.get(1) {
                    tool_ids.insert(value.as_str().to_string());
                }
            }
        }
    }
    let mut errors = Vec::new();
    for entry in WalkDir::new(workspace.path("docs"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| is_docs_reference_text_file(entry.path()))
    {
        let raw = read_utf8(entry.path())?;
        for capture in docs_stage_re.captures_iter(&raw) {
            let token = capture.get(1).map(|value| value.as_str()).unwrap_or_default();
            if !token.is_empty() && !stage_ids.contains(token) {
                errors.push(format!("unknown stage: {token}"));
            }
        }
        for capture in docs_tool_re.captures_iter(&raw) {
            let token = capture.get(1).map(|value| value.as_str()).unwrap_or_default();
            if !token.is_empty() && !token.contains('*') && !tool_ids.contains(token) {
                errors.push(format!("unknown tool: {token}"));
            }
        }
    }
    if errors.is_empty() {
        return success_line("docs stage/tool references validated");
    }
    failure_lines("docs reference unknown stage/tool ids", &errors)
}

fn is_docs_reference_text_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("csv" | "json" | "md" | "toml" | "txt")
    )
}

pub(super) fn docs_check_generated_docs(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-generated-docs", args)?;
    let required = [
        "docs/30-operations/SCOPE_CLOSURE_CHECKLIST.generated.md",
        "docs/20-science/TOOL_INDEX.md",
        "docs/20-science/DOMAIN_COVERAGE.generated.md",
        "docs/30-operations/APPTAINER_QA_MATRIX.md",
        "docs/00-intro/REPO_ROOT_MAP.generated.md",
        "docs/50-reference/COMPATIBILITY_MATRIX.md",
    ];
    let mut errors = Vec::new();
    for rel in required {
        ensure_generated_header(workspace, rel, &mut errors)?;
    }
    for entry in WalkDir::new(workspace.path("docs"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry
            .path()
            .file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| name.ends_with(".generated.md"))
        {
            ensure_generated_header_path(workspace, entry.path(), &mut errors)?;
        }
    }
    let temp_root = temp_subdir(workspace, "generated-docs")?;
    bijux_dna_infra::ensure_dir(temp_root.join("00-intro"))?;
    bijux_dna_infra::ensure_dir(temp_root.join("20-science"))?;
    bijux_dna_infra::ensure_dir(temp_root.join("30-operations"))?;
    bijux_dna_infra::ensure_dir(temp_root.join("50-reference"))?;
    write_utf8(
        &temp_root.join("30-operations/SCOPE_CLOSURE_CHECKLIST.generated.md"),
        &read_utf8(&workspace.path("docs/30-operations/SCOPE_CLOSURE_CHECKLIST.generated.md"))?,
    )?;
    generate_tool_index(workspace, &temp_root.join("20-science/TOOL_INDEX.md"))?;
    generate_domain_coverage_doc(
        workspace,
        &temp_root.join("20-science/DOMAIN_COVERAGE.generated.md"),
    )?;
    let qa_matrix = run_program(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "-q".to_string(),
            "-p".to_string(),
            "bijux-dna-dev".to_string(),
            "--".to_string(),
            "containers".to_string(),
            "run".to_string(),
            "generate-qa-matrix".to_string(),
            "--".to_string(),
            temp_root.join("30-operations/APPTAINER_QA_MATRIX.md").display().to_string(),
        ],
    )?;
    if !qa_matrix.is_success() {
        return Ok(qa_matrix);
    }
    generate_repo_root_map(workspace, &temp_root.join("00-intro/REPO_ROOT_MAP.generated.md"))?;
    generate_compatibility_matrix(
        workspace,
        &temp_root.join("50-reference/COMPATIBILITY_MATRIX.md"),
    )?;
    generate_compatibility_reference_docs(workspace, &temp_root.join("50-reference"))?;
    generate_docs_graph(workspace, &temp_root.join("DOCS_GRAPH.toml"))?;
    for (actual, expected) in [
        (
            workspace.path("docs/20-science/TOOL_INDEX.md"),
            temp_root.join("20-science/TOOL_INDEX.md"),
        ),
        (
            workspace.path("docs/20-science/DOMAIN_COVERAGE.generated.md"),
            temp_root.join("20-science/DOMAIN_COVERAGE.generated.md"),
        ),
        (
            workspace.path("docs/30-operations/APPTAINER_QA_MATRIX.md"),
            temp_root.join("30-operations/APPTAINER_QA_MATRIX.md"),
        ),
        (
            workspace.path("docs/00-intro/REPO_ROOT_MAP.generated.md"),
            temp_root.join("00-intro/REPO_ROOT_MAP.generated.md"),
        ),
        (
            workspace.path("docs/50-reference/COMPATIBILITY_MATRIX.md"),
            temp_root.join("50-reference/COMPATIBILITY_MATRIX.md"),
        ),
        (
            workspace.path("docs/50-reference/SCHEMA_REGISTRY.md"),
            temp_root.join("50-reference/SCHEMA_REGISTRY.md"),
        ),
        (
            workspace.path("docs/50-reference/API_VERSIONING.md"),
            temp_root.join("50-reference/API_VERSIONING.md"),
        ),
        (
            workspace.path("docs/50-reference/DEPRECATION_DASHBOARD.md"),
            temp_root.join("50-reference/DEPRECATION_DASHBOARD.md"),
        ),
        (
            workspace.path("docs/50-reference/UPGRADE_GUIDE.md"),
            temp_root.join("50-reference/UPGRADE_GUIDE.md"),
        ),
        (workspace.path("docs/DOCS_GRAPH.toml"), temp_root.join("DOCS_GRAPH.toml")),
    ] {
        if read_utf8(&actual)? != read_utf8(&expected)? {
            errors.push(format!(
                "{} drifted from generated output",
                workspace.rel(&actual).display()
            ));
        }
    }
    if errors.is_empty() {
        return success_line("generated docs headers: OK");
    }
    failure_lines("generated-docs: FAILED", &errors)
}

pub(super) fn docs_check_no_placeholder_language(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-no-placeholder-language", args)?;
    let placeholder_pattern = concat!(r"\b(?:", "TO", "DO|TB", "D|WI", "P|placeholder", r")\b");
    let re = Regex::new(placeholder_pattern)?;
    let mut violations = Vec::new();
    for entry in WalkDir::new(workspace.path("docs"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
        if rel.starts_with("docs/overrides/") {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let raw = read_utf8(entry.path())?;
        if re.is_match(&raw) {
            violations.push(rel);
        }
    }
    if violations.is_empty() {
        return success_line("docs-placeholder-policy: OK");
    }
    failure_lines(
        "docs-placeholder-policy: forbidden placeholder language found outside docs/overrides/",
        &violations,
    )
}

pub(super) fn docs_check_root_pollution(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-root-pollution", args)?;
    let mut offenders = Vec::new();
    for rel in ["coverage", "target-docs"] {
        if workspace.path(rel).exists() {
            offenders.push(rel.to_string());
        }
    }
    for entry in fs::read_dir(&workspace.root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("target-") {
            offenders.push(name);
        }
    }
    if offenders.is_empty() {
        return success_line("root-pollution: OK");
    }
    failure_lines("root-pollution: forbidden repo-root outputs detected", &offenders)
}

pub(super) fn docs_check_doc_major_depth(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("check-doc-major-depth", args)?;
    let required = BTreeMap::from([
        ("purpose", Regex::new(r"(?mi)^##\s+Purpose:?\s*$")?),
        ("scope", Regex::new(r"(?mi)^##\s+Scope:?\s*$")?),
        ("contracts", Regex::new(r"(?mi)^##\s+Contracts:?\s*$")?),
        ("examples", Regex::new(r"(?mi)^##\s+Examples:?\s*$")?),
        ("failure modes", Regex::new(r"(?mi)^##\s+Failure modes:?\s*$")?),
    ]);
    let mut errors = Vec::new();
    for rel in [
        "docs/10-architecture/ARCHITECTURE.md",
        "docs/10-architecture/SSOT.md",
        "docs/20-science/SCIENTIFIC_DEFAULTS.md",
        "docs/30-operations/CONTAINERS.md",
        "docs/30-operations/REPRODUCIBILITY.md",
    ] {
        let path = workspace.path(rel);
        if !path.exists() {
            errors.push(format!("{rel}: missing major doc file"));
            continue;
        }
        let raw = read_utf8(&path)?;
        let missing = required
            .iter()
            .filter_map(|(name, re)| (!re.is_match(&raw)).then_some(*name))
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            errors.push(format!("{rel}: missing sections: {}", missing.join(", ")));
        }
    }
    if errors.is_empty() {
        return success_line("doc-major-depth: OK");
    }
    failure_lines("doc-major-depth: FAILED", &errors)
}
