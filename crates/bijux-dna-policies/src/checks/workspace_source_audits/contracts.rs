#![cfg(test)]

use super::*;

#[test]
fn source_matching_preserves_comment_lines_strings_and_overlapping_needles() {
    let source = concat!(
        "  // Command::new(\"ignored\");\n",
        "/* Command::new(\"ignored\");\n",
        " * Command::new(\"ignored\");\n",
        "let child = std::process::Command::new(\"tool\");\n",
        "let note = \"Command::new(\";\n",
        "block_comment_continuation Command::new(\"counted\");\n",
    );
    assert_eq!(
        workspace_source_matches(source, WorkspaceSourceRule::ProcessExecution),
        vec![
            WorkspaceSourceMatch { line: 4, pattern: "Command::new" },
            WorkspaceSourceMatch { line: 4, pattern: "std::process::Command" },
            WorkspaceSourceMatch { line: 5, pattern: "Command::new" },
            WorkspaceSourceMatch { line: 6, pattern: "Command::new" },
        ]
    );
    assert!(workspace_source_matches("// only a comment", WorkspaceSourceRule::ProcessExecution)
        .is_empty());
}

#[test]
fn source_rules_keep_parser_reads_mutation_and_scheduler_evidence_distinct() {
    let source = "pub(crate) fn parse_input() { std::fs::read(\"input\"); std::fs::write(\"output\", b\"x\"); }";
    assert_eq!(
        workspace_source_matches(source, WorkspaceSourceRule::ParserEntrypoint),
        vec![WorkspaceSourceMatch { line: 1, pattern: "pub(crate) fn parse_" },]
    );
    assert_eq!(
        workspace_source_matches(source, WorkspaceSourceRule::RawInputRead),
        vec![
            WorkspaceSourceMatch { line: 1, pattern: "fs::read(" },
            WorkspaceSourceMatch { line: 1, pattern: "std::fs::read(" },
        ]
    );
    assert_eq!(
        workspace_source_matches(source, WorkspaceSourceRule::InputMutation),
        vec![
            WorkspaceSourceMatch { line: 1, pattern: "fs::write(" },
            WorkspaceSourceMatch { line: 1, pattern: "std::fs::write(" },
        ]
    );
    let commands = "Command::new(\"docker\"); Command::new(\"sbatch\"); observer::parse_rows();";
    assert_eq!(
        workspace_source_matches(commands, WorkspaceSourceRule::ContainerExecution),
        vec![WorkspaceSourceMatch { line: 1, pattern: "Command::new(\"docker\")" },]
    );
    assert_eq!(
        workspace_source_matches(commands, WorkspaceSourceRule::SlurmExecution),
        vec![WorkspaceSourceMatch { line: 1, pattern: "Command::new(\"sbatch\")" },]
    );
    assert_eq!(
        workspace_source_matches(commands, WorkspaceSourceRule::PlannerParserApi),
        vec![WorkspaceSourceMatch { line: 1, pattern: "observer::parse_" },]
    );
}

#[test]
fn forbidden_dependencies_preserve_ascii_categories_sections_and_order() -> anyhow::Result<()> {
    let manifest: Value = toml::from_str(
        r#"
[dependencies]
ZRunner = "1"
container_slurm_runner = "1"
plain = "1"
[dev-dependencies]
DockerKit = "1"
[build-dependencies]
SlurmKit = "1"
[target.'cfg(unix)'.dependencies]
foreign_runner = "1"
"#,
    )?;
    let hits = forbidden_workspace_execution_dependencies(&manifest);
    let projection = hits
        .iter()
        .map(|hit| (hit.category.as_str(), hit.section.as_str(), hit.dependency.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        projection,
        [
            ("container", "dependencies", "container_slurm_runner"),
            ("container", "dev-dependencies", "DockerKit"),
            ("runner", "dependencies", "ZRunner"),
            ("runner", "dependencies", "container_slurm_runner"),
            ("slurm", "build-dependencies", "SlurmKit"),
            ("slurm", "dependencies", "container_slurm_runner"),
        ]
    );
    let malformed_shape: Value = toml::from_str("dependencies = []\n")?;
    assert!(forbidden_workspace_execution_dependencies(&malformed_shape).is_empty());
    Ok(())
}

#[test]
fn execution_edges_keep_exact_names_and_informational_status() -> anyhow::Result<()> {
    let manifest: Value = toml::from_str(
        r#"
[dependencies]
bijux-dna-runtime = "1"
bijux-dna-runner = "1"
bijux-dna-runtime-extra = "1"
BIJUX-DNA-RUNNER = "1"
[dev-dependencies]
bijux-dna-runner = "1"
"#,
    )?;
    let edges = workspace_execution_owner_dependencies(&manifest, "dependencies");
    assert_eq!(
        edges.iter().map(|hit| (hit.section.as_str(), hit.dependency.as_str())).collect::<Vec<_>>(),
        [("dependencies", "bijux-dna-runner"), ("dependencies", "bijux-dna-runtime"),]
    );
    assert_eq!(workspace_execution_owner_dependencies(&manifest, "dev-dependencies").len(), 1);
    assert!(workspace_execution_owner_dependencies(&manifest, "build-dependencies").is_empty());
    // Owner edges are observable even when there is no forbidden process evidence.
    assert!(!edges.is_empty());
    assert!(evaluate_workspace_source_audit(WorkspaceSourceAuditInput::ExecutionOwner {
        process_execution: 0,
        container_execution: 0,
        slurm_execution: 0,
    }));
    assert!(!evaluate_workspace_source_audit(WorkspaceSourceAuditInput::ExecutionOwner {
        process_execution: 1,
        container_execution: 0,
        slurm_execution: 0,
    }));
    Ok(())
}

#[test]
fn verdicts_preserve_required_parser_entrypoints_and_planner_read_exceptions() {
    assert!(evaluate_workspace_source_audit(WorkspaceSourceAuditInput::Domain {
        forbidden_dependencies: 0,
        process_execution: 0,
        container_execution: 0,
        slurm_execution: 0,
    }));
    for rejected in [
        WorkspaceSourceAuditInput::Domain {
            forbidden_dependencies: 1,
            process_execution: 0,
            container_execution: 0,
            slurm_execution: 0,
        },
        WorkspaceSourceAuditInput::Domain {
            forbidden_dependencies: 0,
            process_execution: 0,
            container_execution: 1,
            slurm_execution: 0,
        },
        WorkspaceSourceAuditInput::Domain {
            forbidden_dependencies: 0,
            process_execution: 0,
            container_execution: 0,
            slurm_execution: 1,
        },
        WorkspaceSourceAuditInput::Parser {
            parser_entrypoints: 0,
            forbidden_dependencies: 0,
            input_mutations: 0,
            process_execution: 0,
            container_execution: 0,
            slurm_execution: 0,
        },
        WorkspaceSourceAuditInput::Parser {
            parser_entrypoints: 1,
            forbidden_dependencies: 0,
            input_mutations: 1,
            process_execution: 0,
            container_execution: 0,
            slurm_execution: 0,
        },
    ] {
        assert!(!evaluate_workspace_source_audit(rejected));
    }
    assert!(evaluate_workspace_source_audit(WorkspaceSourceAuditInput::Parser {
        parser_entrypoints: 1,
        forbidden_dependencies: 0,
        input_mutations: 0,
        process_execution: 0,
        container_execution: 0,
        slurm_execution: 0,
    }));
    // The caller has already separated admitted planner reads from unexpected reads.
    assert!(evaluate_workspace_source_audit(WorkspaceSourceAuditInput::Planner {
        unexpected_input_reads: 0,
        forbidden_parser_api: 0,
    }));
    assert!(!evaluate_workspace_source_audit(WorkspaceSourceAuditInput::Planner {
        unexpected_input_reads: 1,
        forbidden_parser_api: 0,
    }));
    assert!(!evaluate_workspace_source_audit(WorkspaceSourceAuditInput::Planner {
        unexpected_input_reads: 0,
        forbidden_parser_api: usize::MAX,
    }));
}
