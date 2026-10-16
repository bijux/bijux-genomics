//! Pure governance for caller-acquired workspace source audit observations.
use toml::Value;

#[cfg(test)]
mod contracts;

const PROCESS_EXECUTION_PATTERNS: &[&str] = &[
    concat!("Command", "::new"),
    concat!("std::process::", "Command"),
    concat!("tokio::process::", "Command"),
];
const CONTAINER_EXECUTION_PATTERNS: &[&str] = &[
    concat!("Command", "::new(\"docker\")"),
    concat!("Command", "::new(\"apptainer\")"),
    concat!("Command", "::new(\"singularity\")"),
    concat!("Command", "::new(\"podman\")"),
    concat!("std::process::", "Command", "::new(\"docker\")"),
    concat!("std::process::", "Command", "::new(\"apptainer\")"),
    concat!("std::process::", "Command", "::new(\"singularity\")"),
    concat!("std::process::", "Command", "::new(\"podman\")"),
    concat!("tokio::process::", "Command", "::new(\"docker\")"),
    concat!("tokio::process::", "Command", "::new(\"apptainer\")"),
    concat!("tokio::process::", "Command", "::new(\"singularity\")"),
    concat!("tokio::process::", "Command", "::new(\"podman\")"),
];
const SLURM_EXECUTION_PATTERNS: &[&str] = &[
    concat!("Command", "::new(\"sbatch\")"),
    concat!("Command", "::new(\"srun\")"),
    concat!("Command", "::new(\"salloc\")"),
    concat!("std::process::", "Command", "::new(\"sbatch\")"),
    concat!("std::process::", "Command", "::new(\"srun\")"),
    concat!("std::process::", "Command", "::new(\"salloc\")"),
    concat!("tokio::process::", "Command", "::new(\"sbatch\")"),
    concat!("tokio::process::", "Command", "::new(\"srun\")"),
    concat!("tokio::process::", "Command", "::new(\"salloc\")"),
];

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
    concat!("fs", "::write("),
    concat!("std::fs", "::write("),
    concat!("File", "::create("),
    concat!("fs", "::create_dir("),
    concat!("fs", "::create_dir_all("),
    concat!("std::fs", "::create_dir("),
    concat!("std::fs", "::create_dir_all("),
    concat!("fs", "::remove_file("),
    concat!("fs", "::remove_dir("),
    concat!("fs", "::remove_dir_all("),
    concat!("std::fs", "::remove_file("),
    concat!("std::fs", "::remove_dir("),
    concat!("std::fs", "::remove_dir_all("),
    concat!("fs", "::rename("),
    concat!("std::fs", "::rename("),
    concat!("fs", "::copy("),
    concat!("std::fs", "::copy("),
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

/// One closed source vocabulary used by the four workspace audit commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSourceRule {
    ProcessExecution,
    ContainerExecution,
    SlurmExecution,
    ParserEntrypoint,
    RawInputRead,
    InputMutation,
    PlannerParserApi,
}

impl WorkspaceSourceRule {
    fn patterns(self) -> &'static [&'static str] {
        match self {
            Self::ProcessExecution => PROCESS_EXECUTION_PATTERNS,
            Self::ContainerExecution => CONTAINER_EXECUTION_PATTERNS,
            Self::SlurmExecution => SLURM_EXECUTION_PATTERNS,
            Self::ParserEntrypoint => PARSER_ENTRYPOINT_PATTERNS,
            Self::RawInputRead => RAW_INPUT_READ_PATTERNS,
            Self::InputMutation => INPUT_MUTATION_PATTERNS,
            Self::PlannerParserApi => PLANNER_PARSER_API_PATTERNS,
        }
    }
}

/// One original one-based line and matching literal; source paths remain caller-owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSourceMatch {
    pub line: usize,
    pub pattern: &'static str,
}

/// Observe the existing line-oriented vocabularies without parsing or acquiring Rust.
/// Whole lines beginning with comment markers are skipped; strings, overlapping
/// literals and later block-comment lines retain the existing matching behavior.
#[must_use]
pub fn workspace_source_matches(
    content: &str,
    rule: WorkspaceSourceRule,
) -> Vec<WorkspaceSourceMatch> {
    let mut hits = Vec::new();
    for (line_number, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        for pattern in rule.patterns() {
            if line.contains(pattern) {
                hits.push(WorkspaceSourceMatch { line: line_number + 1, pattern });
            }
        }
    }
    hits
}

/// A forbidden direct dependency with its original category and manifest section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceForbiddenDependency {
    pub section: String,
    pub dependency: String,
    pub category: String,
}

fn matching_dependencies(
    manifest: &Value,
    section_name: &str,
    patterns: &[&str],
    category: &str,
) -> Vec<WorkspaceForbiddenDependency> {
    let Some(table) = manifest.get(section_name).and_then(Value::as_table) else {
        return Vec::new();
    };
    let mut hits = table
        .keys()
        .filter(|dependency| {
            let normalized = dependency.to_ascii_lowercase();
            patterns.iter().any(|pattern| normalized.contains(pattern))
        })
        .map(|dependency| WorkspaceForbiddenDependency {
            section: section_name.to_string(),
            dependency: dependency.clone(),
            category: category.to_string(),
        })
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| left.dependency.cmp(&right.dependency));
    hits
}

/// Classify the three direct manifest sections in original category/section/name order.
/// Missing or non-table sections retain the original empty-observation behavior.
#[must_use]
pub fn forbidden_workspace_execution_dependencies(
    manifest_value: &Value,
) -> Vec<WorkspaceForbiddenDependency> {
    let mut forbidden_direct_dependencies = Vec::new();
    for (patterns, category) in [
        (RUNNER_DEPENDENCY_PATTERNS, "runner"),
        (CONTAINER_DEPENDENCY_PATTERNS, "container"),
        (SLURM_DEPENDENCY_PATTERNS, "slurm"),
    ] {
        for section_name in ["dependencies", "dev-dependencies", "build-dependencies"] {
            forbidden_direct_dependencies.extend(matching_dependencies(
                manifest_value,
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
    forbidden_direct_dependencies
}

/// An informational direct edge to an admitted execution owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceExecutionDependency {
    pub section: String,
    pub dependency: String,
}

/// Observe only exact runner/runtime dependency names in the supplied section.
/// These edges are report evidence; their presence alone does not refuse an audit.
pub fn workspace_execution_owner_dependencies(
    manifest: &Value,
    section_name: &str,
) -> Vec<WorkspaceExecutionDependency> {
    let Some(table) = manifest.get(section_name).and_then(Value::as_table) else {
        return Vec::new();
    };
    let mut hits = table
        .keys()
        .filter(|dependency| {
            matches!(dependency.as_str(), "bijux-dna-runner" | "bijux-dna-runtime")
        })
        .map(|dependency| WorkspaceExecutionDependency {
            section: section_name.to_string(),
            dependency: dependency.clone(),
        })
        .collect::<Vec<_>>();
    hits.sort_by(|left, right| {
        left.section.cmp(&right.section).then_with(|| left.dependency.cmp(&right.dependency))
    });
    hits
}

/// Exact observations required by each audit's existing verdict.
/// The caller retains path admission, parser exclusions and planner allowed-read
/// membership; raw reads allowed by those contracts are not forbidden by inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceSourceAuditInput {
    Domain {
        forbidden_dependencies: usize,
        process_execution: usize,
        container_execution: usize,
        slurm_execution: usize,
    },
    Parser {
        parser_entrypoints: usize,
        forbidden_dependencies: usize,
        input_mutations: usize,
        process_execution: usize,
        container_execution: usize,
        slurm_execution: usize,
    },
    Planner {
        unexpected_input_reads: usize,
        forbidden_parser_api: usize,
    },
    ExecutionOwner {
        process_execution: usize,
        container_execution: usize,
        slurm_execution: usize,
    },
}

/// Preserve the four existing semantic decisions without acquiring or publishing evidence.
#[must_use]
pub fn evaluate_workspace_source_audit(input: WorkspaceSourceAuditInput) -> bool {
    match input {
        WorkspaceSourceAuditInput::Domain {
            forbidden_dependencies,
            process_execution,
            container_execution,
            slurm_execution,
        } => {
            forbidden_dependencies == 0
                && process_execution == 0
                && container_execution == 0
                && slurm_execution == 0
        }
        WorkspaceSourceAuditInput::Parser {
            parser_entrypoints,
            forbidden_dependencies,
            input_mutations,
            process_execution,
            container_execution,
            slurm_execution,
        } => {
            parser_entrypoints != 0
                && forbidden_dependencies == 0
                && input_mutations == 0
                && process_execution == 0
                && container_execution == 0
                && slurm_execution == 0
        }
        WorkspaceSourceAuditInput::Planner { unexpected_input_reads, forbidden_parser_api } => {
            unexpected_input_reads == 0 && forbidden_parser_api == 0
        }
        WorkspaceSourceAuditInput::ExecutionOwner {
            process_execution,
            container_execution,
            slurm_execution,
        } => process_execution == 0 && container_execution == 0 && slurm_execution == 0,
    }
}
