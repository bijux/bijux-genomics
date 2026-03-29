use anyhow::Result;

use super::{
    artifact_env, check_default_settings_docs, check_doc_links, check_domain_index,
    check_domain_layout, check_domain_schema, check_domain_tool_metadata,
    check_external_tool_policy, check_fixture_contracts, check_inventory, check_orphan_files,
    check_planner_fixture_coverage, check_planner_stage_coverage,
    check_reference_bundle_lock, check_rust_stage_catalog_parity, check_shared_tools,
    check_ssot_authority, check_tool_container_parity, command_runner,
};
use crate::model::domain::DomainCommandOutcome;
use crate::runtime::workspace::Workspace;

pub(super) fn validate(workspace: &Workspace, args: &[String]) -> Result<DomainCommandOutcome> {
    let allow_non_artifacts = match args {
        [] => false,
        [single] if single == "--allow-non-artifacts" => true,
        [single] if single == "--help" || single == "-h" => {
            return Ok(DomainCommandOutcome::success(
                "Usage: cargo run -p bijux-dna-dev -- domain run validate -- [--allow-non-artifacts]\n",
            ));
        }
        _ => {
            return Ok(DomainCommandOutcome {
                exit_code: 2,
                stdout: String::new(),
                stderr: "Usage: cargo run -p bijux-dna-dev -- domain run validate -- [--allow-non-artifacts]\n".to_string(),
            });
        }
    };

    let checks = [
        check_domain_layout(workspace)?,
        check_domain_schema(workspace)?,
        check_domain_index(workspace)?,
        check_ssot_authority(workspace)?,
        check_rust_stage_catalog_parity(workspace)?,
        check_shared_tools(workspace)?,
        check_tool_container_parity(workspace)?,
        check_domain_tool_metadata(workspace)?,
        check_planner_stage_coverage(workspace)?,
        check_planner_fixture_coverage(workspace)?,
        check_default_settings_docs(workspace)?,
        check_fixture_contracts(workspace)?,
        check_orphan_files(workspace)?,
        check_doc_links(workspace)?,
        check_external_tool_policy(workspace)?,
        check_reference_bundle_lock(workspace)?,
        check_inventory(workspace)?,
    ];
    let mut stdout = String::new();
    let mut stderr = String::new();
    for outcome in checks {
        stdout.push_str(&outcome.stdout);
        stderr.push_str(&outcome.stderr);
        if !outcome.is_success() {
            return Ok(DomainCommandOutcome {
                exit_code: outcome.exit_code,
                stdout,
                stderr,
            });
        }
    }

    let env = if allow_non_artifacts {
        vec![
            ("TZ".to_string(), "UTC".to_string()),
            ("LC_ALL".to_string(), "C".to_string()),
        ]
    } else {
        artifact_env(workspace)?
    };
    let compiler = command_runner(workspace).run_owned_with_env(
        "cargo",
        &[
            "run".to_string(),
            "-p".to_string(),
            "bijux-dna-domain-compiler".to_string(),
            "--bin".to_string(),
            "domain_validate".to_string(),
            "--".to_string(),
            "--domain-dir".to_string(),
            workspace.path("domain").display().to_string(),
        ],
        &env,
    )?;
    let compiler_outcome = DomainCommandOutcome::from_output(compiler);
    stdout.push_str(&compiler_outcome.stdout);
    stderr.push_str(&compiler_outcome.stderr);
    Ok(DomainCommandOutcome {
        exit_code: compiler_outcome.exit_code,
        stdout,
        stderr,
    })
}
