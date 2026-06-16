use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;
use toml::Value;

const DOMAIN_CRATES: &[&str] =
    &["bijux-dna-domain-bam", "bijux-dna-domain-fastq", "bijux-dna-domain-vcf"];
const PLANNER_CRATES: &[&str] =
    &["bijux-dna-planner-bam", "bijux-dna-planner-fastq", "bijux-dna-planner-vcf"];

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
const PARSER_ENTRYPOINT_PATTERNS: &[&str] = &["pub fn parse_", "pub(crate) fn parse_"];
const RAW_INPUT_READ_PATTERNS: &[&str] = &[
    "fs::read_to_string",
    "std::fs::read_to_string",
    "fs::read(",
    "std::fs::read(",
    "File::open(",
    ".read_to_string(",
];
const INPUT_MUTATION_PATTERNS: &[&str] = &[
    "fs::write(",
    "std::fs::write(",
    "File::create(",
    "fs::create_dir(",
    "fs::create_dir_all(",
    "std::fs::create_dir(",
    "std::fs::create_dir_all(",
    "fs::remove_file(",
    "fs::remove_dir(",
    "fs::remove_dir_all(",
    "std::fs::remove_file(",
    "std::fs::remove_dir(",
    "std::fs::remove_dir_all(",
    "fs::rename(",
    "std::fs::rename(",
    "fs::copy(",
    "std::fs::copy(",
    "atomic_write",
    "write_bytes(",
    "write_json(",
];
const PLANNER_PARSER_API_PATTERNS: &[&str] = &[
    "bijux_dna_domain_fastq::observer",
    "bijux_dna_stages_fastq::observer",
    "observer::parse_",
    "metrics::parse_",
    "parsers::parse_",
    "bijux_dna_domain_vcf::parsers",
    "raw_parser_contract",
];

#[derive(Clone, Debug)]
struct WorkspaceMember {
    crate_name: String,
    manifest_path: PathBuf,
}

#[derive(Clone, Copy, Debug)]
struct ParserSurfaceSpec {
    crate_name: &'static str,
    parser_root: &'static str,
    excluded_paths: &'static [ParserSurfaceExclusionSpec],
}

#[derive(Clone, Copy, Debug)]
struct ParserSurfaceExclusionSpec {
    path: &'static str,
    reason: &'static str,
}

#[derive(Clone, Copy, Debug)]
struct PlannerCrateAuditSpec {
    crate_name: &'static str,
    allowed_input_read_files: &'static [AllowedAuditFileSpec],
}

#[derive(Clone, Copy, Debug)]
struct AllowedAuditFileSpec {
    path: &'static str,
    reason: &'static str,
}

const PARSER_SURFACES: &[ParserSurfaceSpec] = &[
    ParserSurfaceSpec {
        crate_name: "bijux-dna-domain-bam",
        parser_root: "src/metrics",
        excluded_paths: &[ParserSurfaceExclusionSpec {
            path: "src/metrics/raw_parser_contract.rs",
            reason: "governance helper for malformed raw-fixture probe generation",
        }],
    },
    ParserSurfaceSpec {
        crate_name: "bijux-dna-domain-fastq",
        parser_root: "src/observer/parse",
        excluded_paths: &[
            ParserSurfaceExclusionSpec {
                path: "src/observer/parse/parser_contracts",
                reason: "cfg(test) parser contract fixture-bank suite",
            },
            ParserSurfaceExclusionSpec {
                path: "src/observer/parse/raw_parser_contract.rs",
                reason: "governance helper for malformed raw-fixture probe generation",
            },
        ],
    },
    ParserSurfaceSpec {
        crate_name: "bijux-dna-domain-vcf",
        parser_root: "src/parsers",
        excluded_paths: &[],
    },
];

const PLANNER_AUDIT_SPECS: &[PlannerCrateAuditSpec] = &[
    PlannerCrateAuditSpec {
        crate_name: "bijux-dna-planner-bam",
        allowed_input_read_files: &[
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/local_readiness.rs",
                reason: "loads governed local-ready planning inputs and runtime defaults",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/local_smoke.rs",
                reason: "loads governed local-smoke planning inputs and fixture manifests",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/selection/domain_tool_specs.rs",
                reason: "loads governed BAM tool planning metadata",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/selection/domain_tool_output_contracts.rs",
                reason: "loads governed BAM output-contract metadata",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/selection/registry.rs",
                reason: "reads the governed workspace tool registry snapshot",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-bam/src/stage_activation.rs",
                reason: "reads governed stage activation policy",
            },
        ],
    },
    PlannerCrateAuditSpec {
        crate_name: "bijux-dna-planner-fastq",
        allowed_input_read_files: &[
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/local_readiness.rs",
                reason: "loads governed local-ready FASTQ planning inputs",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/local_smoke.rs",
                reason: "loads governed local-smoke FASTQ planning inputs and fixture manifests",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/planner/quality_sampling.rs",
                reason: "samples governed input reads for planning-time quality estimation",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-fastq/src/selection/domain_tool_specs.rs",
                reason: "loads governed FASTQ tool planning metadata",
            },
            AllowedAuditFileSpec {
                path:
                    "crates/bijux-dna-planner-fastq/src/selection/domain_tool_output_contracts.rs",
                reason: "loads governed FASTQ output-contract metadata",
            },
        ],
    },
    PlannerCrateAuditSpec {
        crate_name: "bijux-dna-planner-vcf",
        allowed_input_read_files: &[
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-vcf/src/coverage.rs",
                reason: "reads governed coverage-regime planning thresholds",
            },
            AllowedAuditFileSpec {
                path: "crates/bijux-dna-planner-vcf/src/workspace_config.rs",
                reason: "reads governed workspace tool and parameter registries",
            },
        ],
    },
];

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
pub struct ParserNoExecutionReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub audited_surface_count: usize,
    pub ok: bool,
    pub surfaces: Vec<ParserSurfaceAuditReport>,
}

#[derive(Debug, Serialize)]
pub struct PlannerNoParserReport {
    pub schema_version: &'static str,
    pub workspace_manifest: String,
    pub output_path: String,
    pub audited_crate_count: usize,
    pub ok: bool,
    pub crates: Vec<PlannerNoParserCrateReport>,
}

#[derive(Debug, Serialize)]
pub struct PlannerNoParserCrateReport {
    pub crate_name: String,
    pub manifest_path: String,
    pub scanned_rust_files: Vec<String>,
    pub allowed_input_read_paths: Vec<AllowedAuditPath>,
    pub planner_input_read_refs: Vec<SourcePatternHit>,
    pub unexpected_input_read_refs: Vec<SourcePatternHit>,
    pub forbidden_parser_api_refs: Vec<SourcePatternHit>,
    pub ok: bool,
}

#[derive(Debug, Serialize)]
pub struct ParserSurfaceAuditReport {
    pub crate_name: String,
    pub manifest_path: String,
    pub parser_root: String,
    pub scanned_rust_files: Vec<String>,
    pub excluded_governance_paths: Vec<ExcludedAuditPath>,
    pub parser_entrypoints: Vec<SourcePatternHit>,
    pub raw_input_read_refs: Vec<SourcePatternHit>,
    pub input_mutation_refs: Vec<SourcePatternHit>,
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

#[derive(Debug, Serialize)]
pub struct ExcludedAuditPath {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct AllowedAuditPath {
    pub path: String,
    pub reason: String,
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

fn collect_rust_sources_with_exclusions(
    root: &Path,
    current: &Path,
    excluded_paths: &BTreeMap<PathBuf, String>,
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    if !current.exists() {
        return Ok(());
    }
    if excluded_paths.contains_key(&current.to_path_buf()) {
        return Ok(());
    }
    for entry in
        std::fs::read_dir(current).with_context(|| format!("read {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if excluded_paths.contains_key(&path) {
            continue;
        }
        if path.is_dir() {
            collect_rust_sources_with_exclusions(root, &path, excluded_paths, files)?;
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
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

fn parser_surface_spec(crate_name: &str) -> Option<&'static ParserSurfaceSpec> {
    PARSER_SURFACES.iter().find(|spec| spec.crate_name == crate_name)
}

fn planner_audit_spec(crate_name: &str) -> Option<&'static PlannerCrateAuditSpec> {
    PLANNER_AUDIT_SPECS.iter().find(|spec| spec.crate_name == crate_name)
}

fn audit_parser_surface(
    cwd: &Path,
    member: &WorkspaceMember,
    spec: &ParserSurfaceSpec,
) -> Result<ParserSurfaceAuditReport> {
    let crate_root = member
        .manifest_path
        .parent()
        .with_context(|| format!("resolve crate root from {}", member.manifest_path.display()))?;
    let manifest_text = std::fs::read_to_string(&member.manifest_path)
        .with_context(|| format!("read {}", member.manifest_path.display()))?;
    let manifest_value: Value = toml::from_str(&manifest_text)
        .with_context(|| format!("parse {}", member.manifest_path.display()))?;

    let parser_root = crate_root.join(spec.parser_root);
    let excluded_paths = spec
        .excluded_paths
        .iter()
        .map(|excluded| (crate_root.join(excluded.path), excluded.reason.to_string()))
        .collect::<BTreeMap<_, _>>();

    let mut rust_files = Vec::new();
    collect_rust_sources_with_exclusions(
        crate_root,
        &parser_root,
        &excluded_paths,
        &mut rust_files,
    )?;
    rust_files.sort();

    let excluded_governance_paths = spec
        .excluded_paths
        .iter()
        .filter_map(|excluded| {
            let absolute = crate_root.join(excluded.path);
            if absolute.exists() {
                Some(ExcludedAuditPath {
                    path: relative_display(&absolute, cwd),
                    reason: excluded.reason.to_string(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut parser_entrypoints = Vec::new();
    let mut raw_input_read_refs = Vec::new();
    let mut input_mutation_refs = Vec::new();
    let mut process_execution_refs = Vec::new();
    let mut container_execution_refs = Vec::new();
    let mut slurm_execution_refs = Vec::new();
    for rust_file in &rust_files {
        let absolute = crate_root.join(rust_file);
        let content = std::fs::read_to_string(&absolute)
            .with_context(|| format!("read {}", absolute.display()))?;
        push_source_hits(
            &mut parser_entrypoints,
            &absolute,
            cwd,
            &content,
            PARSER_ENTRYPOINT_PATTERNS,
        );
        push_source_hits(
            &mut raw_input_read_refs,
            &absolute,
            cwd,
            &content,
            RAW_INPUT_READ_PATTERNS,
        );
        push_source_hits(
            &mut input_mutation_refs,
            &absolute,
            cwd,
            &content,
            INPUT_MUTATION_PATTERNS,
        );
        push_source_hits(
            &mut process_execution_refs,
            &absolute,
            cwd,
            &content,
            PROCESS_EXECUTION_PATTERNS,
        );
        push_source_hits(
            &mut container_execution_refs,
            &absolute,
            cwd,
            &content,
            CONTAINER_EXECUTION_PATTERNS,
        );
        push_source_hits(
            &mut slurm_execution_refs,
            &absolute,
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

    let scanned_rust_files = rust_files
        .iter()
        .map(|path| relative_display(&crate_root.join(path), cwd))
        .collect::<Vec<_>>();
    let ok = !parser_entrypoints.is_empty()
        && forbidden_direct_dependencies.is_empty()
        && input_mutation_refs.is_empty()
        && process_execution_refs.is_empty()
        && container_execution_refs.is_empty()
        && slurm_execution_refs.is_empty();

    Ok(ParserSurfaceAuditReport {
        crate_name: member.crate_name.clone(),
        manifest_path: relative_display(&member.manifest_path, cwd),
        parser_root: relative_display(&parser_root, cwd),
        scanned_rust_files,
        excluded_governance_paths,
        parser_entrypoints,
        raw_input_read_refs,
        input_mutation_refs,
        forbidden_direct_dependencies,
        process_execution_refs,
        container_execution_refs,
        slurm_execution_refs,
        ok,
    })
}

fn audit_planner_crate(
    cwd: &Path,
    member: &WorkspaceMember,
    spec: &PlannerCrateAuditSpec,
) -> Result<PlannerNoParserCrateReport> {
    let crate_root = member
        .manifest_path
        .parent()
        .with_context(|| format!("resolve crate root from {}", member.manifest_path.display()))?;

    let mut rust_files = Vec::new();
    collect_rust_sources(&crate_root.join("src"), &mut rust_files)?;
    rust_files.sort();

    let allowed_input_read_paths = spec
        .allowed_input_read_files
        .iter()
        .map(|allowed| AllowedAuditPath {
            path: allowed.path.to_string(),
            reason: allowed.reason.to_string(),
        })
        .collect::<Vec<_>>();
    let allowed_input_read_files =
        spec.allowed_input_read_files.iter().map(|allowed| allowed.path).collect::<BTreeSet<_>>();

    let mut planner_input_read_refs = Vec::new();
    let mut unexpected_input_read_refs = Vec::new();
    let mut forbidden_parser_api_refs = Vec::new();
    for rust_file in &rust_files {
        let content = std::fs::read_to_string(rust_file)
            .with_context(|| format!("read {}", rust_file.display()))?;
        let mut read_refs = Vec::new();
        push_source_hits(&mut read_refs, rust_file, cwd, &content, RAW_INPUT_READ_PATTERNS);
        for hit in read_refs {
            if allowed_input_read_files.contains(hit.path.as_str()) {
                planner_input_read_refs.push(hit);
            } else {
                unexpected_input_read_refs.push(hit);
            }
        }
        push_source_hits(
            &mut forbidden_parser_api_refs,
            rust_file,
            cwd,
            &content,
            PLANNER_PARSER_API_PATTERNS,
        );
    }

    let scanned_rust_files =
        rust_files.iter().map(|path| relative_display(path, cwd)).collect::<Vec<_>>();
    let ok = unexpected_input_read_refs.is_empty() && forbidden_parser_api_refs.is_empty();

    Ok(PlannerNoParserCrateReport {
        crate_name: member.crate_name.clone(),
        manifest_path: relative_display(&member.manifest_path, cwd),
        scanned_rust_files,
        allowed_input_read_paths,
        planner_input_read_refs,
        unexpected_input_read_refs,
        forbidden_parser_api_refs,
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

/// # Errors
/// Returns an error if the parser surface execution audit cannot be resolved or written.
pub fn write_parser_no_execution_report(
    cwd: &Path,
    output_path: &Path,
) -> Result<ParserNoExecutionReport> {
    let members = workspace_members(cwd)?;
    let mut surfaces = members
        .iter()
        .filter_map(|member| parser_surface_spec(&member.crate_name).map(|spec| (member, spec)))
        .map(|(member, spec)| audit_parser_surface(cwd, member, spec))
        .collect::<Result<Vec<_>>>()?;
    surfaces.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));

    let report = ParserNoExecutionReport {
        schema_version: "bijux.crates.parser_no_execution.v1",
        workspace_manifest: relative_display(&cwd.join("Cargo.toml"), cwd),
        output_path: relative_display(output_path, cwd),
        audited_surface_count: surfaces.len(),
        ok: surfaces.iter().all(|surface| surface.ok),
        surfaces,
    };

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    bijux_dna_infra::atomic_write_json(output_path, &report)?;
    Ok(report)
}

/// # Errors
/// Returns an error if the planner crate parsing audit cannot be resolved or written.
pub fn write_planner_no_parser_report(
    cwd: &Path,
    output_path: &Path,
) -> Result<PlannerNoParserReport> {
    let members = workspace_members(cwd)?;
    let mut crates = members
        .iter()
        .filter(|member| PLANNER_CRATES.contains(&member.crate_name.as_str()))
        .filter_map(|member| planner_audit_spec(&member.crate_name).map(|spec| (member, spec)))
        .map(|(member, spec)| audit_planner_crate(cwd, member, spec))
        .collect::<Result<Vec<_>>>()?;
    crates.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));

    let report = PlannerNoParserReport {
        schema_version: "bijux.crates.planner_no_parser.v1",
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
