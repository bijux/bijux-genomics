use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;
use toml::Value;

const DOMAIN_CRATES: &[&str] =
    &["bijux-dna-domain-bam", "bijux-dna-domain-fastq", "bijux-dna-domain-vcf"];

const PROCESS_EXECUTION_PATTERNS: &[&str] = &[
    concat!("Command", "::new"),
    concat!("std::process::", "Command"),
    concat!("tokio::process::", "Command"),
];
const CONTAINER_EXECUTION_PATTERNS: &[&str] = &["docker", "apptainer", "singularity", "podman"];
const SLURM_EXECUTION_PATTERNS: &[&str] = &["slurm", "sbatch", "srun"];

const RUNNER_DEPENDENCY_PATTERNS: &[&str] = &["runner"];
const CONTAINER_DEPENDENCY_PATTERNS: &[&str] =
    &["docker", "apptainer", "singularity", "podman", "container"];
const SLURM_DEPENDENCY_PATTERNS: &[&str] = &["slurm"];

#[derive(Clone, Debug)]
struct WorkspaceMember {
    crate_name: String,
    manifest_path: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct CrateDependencyMapReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub crate_count: usize,
    pub edge_count: usize,
    pub crates: Vec<CrateDependencyNode>,
    pub edges: Vec<CrateDependencyEdge>,
}

#[derive(Debug, Serialize)]
pub struct CrateDependencyNode {
    pub crate_name: String,
    pub manifest_path: String,
    pub direct_workspace_dependencies: Vec<String>,
    pub direct_workspace_dependents: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DomainNoExecutionReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub audited_crate_count: usize,
    pub ok: bool,
    pub crates: Vec<DomainNoExecutionCrateReport>,
}

#[derive(Debug, Serialize)]
pub struct DomainNoExecutionCrateReport {
    pub crate_name: String,
    pub manifest_path: String,
    pub scanned_rust_files: Vec<String>,
    pub forbidden_direct_dependencies: Vec<ForbiddenDependencyHit>,
    pub process_execution_refs: Vec<SourcePatternHit>,
    pub container_execution_refs: Vec<SourcePatternHit>,
    pub slurm_execution_refs: Vec<SourcePatternHit>,
    pub ok: bool,
}

#[derive(Debug, Serialize)]
pub struct CrateDependencyEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize)]
pub struct ForbiddenDependencyHit {
    pub section: String,
    pub dependency: String,
    pub category: String,
}

#[derive(Debug, Serialize)]
pub struct SourcePatternHit {
    pub path: String,
    pub line: usize,
    pub pattern: String,
}

fn relative_display(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).display().to_string()
}

fn workspace_members(cwd: &Path) -> Result<Vec<WorkspaceMember>> {
    let workspace_manifest_path = cwd.join("Cargo.toml");
    let manifest = std::fs::read_to_string(&workspace_manifest_path)
        .with_context(|| format!("read {}", workspace_manifest_path.display()))?;
    let value: Value = toml::from_str(&manifest)
        .with_context(|| format!("parse {}", workspace_manifest_path.display()))?;
    let members = value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
        .context("workspace.members missing from root Cargo.toml")?;

    let mut resolved = Vec::new();
    for member in members {
        let member = member.as_str().context("workspace.members must contain only string paths")?;
        let manifest_path = cwd.join(member).join("Cargo.toml");
        let crate_manifest = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("read {}", manifest_path.display()))?;
        let crate_value: Value = toml::from_str(&crate_manifest)
            .with_context(|| format!("parse {}", manifest_path.display()))?;
        let crate_name = crate_value
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(Value::as_str)
            .with_context(|| format!("package.name missing from {}", manifest_path.display()))?;
        resolved.push(WorkspaceMember { crate_name: crate_name.to_string(), manifest_path });
    }
    resolved.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));
    Ok(resolved)
}

fn collect_rust_sources(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir).with_context(|| format!("read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_sources(&path, files)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
    Ok(())
}

fn push_source_hits(
    hits: &mut Vec<SourcePatternHit>,
    path: &Path,
    root: &Path,
    content: &str,
    patterns: &[&str],
) {
    for (line_number, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        for pattern in patterns {
            if line.contains(pattern) {
                hits.push(SourcePatternHit {
                    path: relative_display(path, root),
                    line: line_number + 1,
                    pattern: (*pattern).to_string(),
                });
            }
        }
    }
}

fn collect_manifest_dependency_hits(
    manifest: &Value,
    section_name: &str,
    patterns: &[&str],
    category: &str,
) -> Vec<ForbiddenDependencyHit> {
    let Some(table) = manifest.get(section_name).and_then(Value::as_table) else {
        return Vec::new();
    };
    let mut hits = table
        .keys()
        .filter(|dependency| {
            let normalized = dependency.to_ascii_lowercase();
            patterns.iter().any(|pattern| normalized.contains(pattern))
        })
        .map(|dependency| ForbiddenDependencyHit {
            section: section_name.to_string(),
            dependency: dependency.to_string(),
            category: category.to_string(),
        })
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| left.dependency.cmp(&right.dependency));
    hits
}

fn audit_domain_crate(
    cwd: &Path,
    member: &WorkspaceMember,
) -> Result<DomainNoExecutionCrateReport> {
    let crate_root = member
        .manifest_path
        .parent()
        .with_context(|| format!("resolve crate root from {}", member.manifest_path.display()))?;
    let manifest_text = std::fs::read_to_string(&member.manifest_path)
        .with_context(|| format!("read {}", member.manifest_path.display()))?;
    let manifest_value: Value = toml::from_str(&manifest_text)
        .with_context(|| format!("parse {}", member.manifest_path.display()))?;

    let mut rust_files = Vec::new();
    collect_rust_sources(&crate_root.join("src"), &mut rust_files)?;
    let build_rs = crate_root.join("build.rs");
    if build_rs.is_file() {
        rust_files.push(build_rs);
    }
    rust_files.sort();

    let mut process_execution_refs = Vec::new();
    let mut container_execution_refs = Vec::new();
    let mut slurm_execution_refs = Vec::new();
    for rust_file in &rust_files {
        let content = std::fs::read_to_string(rust_file)
            .with_context(|| format!("read {}", rust_file.display()))?;
        push_source_hits(
            &mut process_execution_refs,
            rust_file,
            cwd,
            &content,
            PROCESS_EXECUTION_PATTERNS,
        );
        push_source_hits(
            &mut container_execution_refs,
            rust_file,
            cwd,
            &content,
            CONTAINER_EXECUTION_PATTERNS,
        );
        push_source_hits(
            &mut slurm_execution_refs,
            rust_file,
            cwd,
            &content,
            SLURM_EXECUTION_PATTERNS,
        );
    }

    let mut forbidden_direct_dependencies = Vec::new();
    for (patterns, category) in [
        (RUNNER_DEPENDENCY_PATTERNS, "runner"),
        (CONTAINER_DEPENDENCY_PATTERNS, "container"),
        (SLURM_DEPENDENCY_PATTERNS, "slurm"),
    ] {
        for section_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
            forbidden_direct_dependencies.extend(collect_manifest_dependency_hits(
                &manifest_value,
                section_name,
                patterns,
                category,
            ));
        }
    }
    forbidden_direct_dependencies.sort_by(|left, right| {
        left.category
            .cmp(&right.category)
            .then_with(|| left.section.cmp(&right.section))
            .then_with(|| left.dependency.cmp(&right.dependency))
    });

    let scanned_rust_files =
        rust_files.iter().map(|path| relative_display(path, cwd)).collect::<Vec<_>>();

    let ok = forbidden_direct_dependencies.is_empty()
        && process_execution_refs.is_empty()
        && container_execution_refs.is_empty()
        && slurm_execution_refs.is_empty();

    Ok(DomainNoExecutionCrateReport {
        crate_name: member.crate_name.clone(),
        manifest_path: relative_display(&member.manifest_path, cwd),
        scanned_rust_files,
        forbidden_direct_dependencies,
        process_execution_refs,
        container_execution_refs,
        slurm_execution_refs,
        ok,
    })
}

/// # Errors
/// Returns an error if the workspace crate graph cannot be resolved or written.
pub fn write_dependency_map(cwd: &Path, output_path: &Path) -> Result<CrateDependencyMapReport> {
    let members = workspace_members(cwd)?;
    let member_names =
        members.iter().map(|member| member.crate_name.clone()).collect::<BTreeSet<_>>();
    let edges = bijux_dna_api::v1::api::workspace_edges().context("load workspace edges")?;

    let mut direct_deps = BTreeMap::<String, BTreeSet<String>>::new();
    let mut direct_dependents = BTreeMap::<String, BTreeSet<String>>::new();
    for crate_name in &member_names {
        direct_deps.insert(crate_name.clone(), BTreeSet::new());
        direct_dependents.insert(crate_name.clone(), BTreeSet::new());
    }

    let mut edge_rows = Vec::new();
    for (from, to) in edges {
        if !member_names.contains(&from) || !member_names.contains(&to) {
            continue;
        }
        direct_deps.entry(from.clone()).or_default().insert(to.clone());
        direct_dependents.entry(to.clone()).or_default().insert(from.clone());
        edge_rows.push(CrateDependencyEdge { from, to });
    }
    edge_rows
        .sort_by(|left, right| left.from.cmp(&right.from).then_with(|| left.to.cmp(&right.to)));

    let nodes = members
        .into_iter()
        .map(|member| CrateDependencyNode {
            direct_workspace_dependencies: direct_deps
                .remove(&member.crate_name)
                .unwrap_or_default()
                .into_iter()
                .collect(),
            direct_workspace_dependents: direct_dependents
                .remove(&member.crate_name)
                .unwrap_or_default()
                .into_iter()
                .collect(),
            crate_name: member.crate_name,
            manifest_path: relative_display(&member.manifest_path, cwd),
        })
        .collect::<Vec<_>>();

    let report = CrateDependencyMapReport {
        schema_version: "bijux.crates.dependency_map.v1",
        workspace_manifest: relative_display(&cwd.join("Cargo.toml"), cwd),
        output_path: relative_display(output_path, cwd),
        crate_count: nodes.len(),
        edge_count: edge_rows.len(),
        crates: nodes,
        edges: edge_rows,
    };

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)?;
    Ok(report)
}

/// # Errors
/// Returns an error if the domain crate execution audit cannot be resolved or written.
pub fn write_domain_no_execution_report(
    cwd: &Path,
    output_path: &Path,
) -> Result<DomainNoExecutionReport> {
    let members = workspace_members(cwd)?;
    let mut crates = members
        .iter()
        .filter(|member| DOMAIN_CRATES.contains(&member.crate_name.as_str()))
        .map(|member| audit_domain_crate(cwd, member))
        .collect::<Result<Vec<_>>>()?;
    crates.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));

    let report = DomainNoExecutionReport {
        schema_version: "bijux.crates.domain_no_execution.v1",
        workspace_manifest: relative_display(&cwd.join("Cargo.toml"), cwd),
        output_path: relative_display(output_path, cwd),
        audited_crate_count: crates.len(),
        ok: crates.iter().all(|crate_report| crate_report.ok),
        crates,
    };

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)?;
    Ok(report)
}
