use std::fs;

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

use super::domain_workflow::{read_utf8, success_line, write_utf8};
use super::REGISTRY_LOCK_GENERATED_BY;
use crate::model::domain::DomainCommandOutcome;
use crate::runtime::workspace::Workspace;

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(super) fn check_reference_bundle_lock(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let catalog = workspace.path("configs/runtime/reference_bundles.toml");
    let lock = workspace.path("configs/runtime/reference_bundles_lock.sha256");
    let materialization_lock_json = workspace.path("configs/runtime/references/locks/lock.json");
    let materialization_lock_sha =
        workspace.path("configs/runtime/references/locks/lock.json.sha256");

    if !catalog.is_file() {
        return Ok(DomainCommandOutcome::failure(format!(
            "reference bundle lock check: missing {}\n",
            catalog.display()
        )));
    }
    if !lock.is_file() {
        return Ok(DomainCommandOutcome::failure(format!(
            "reference bundle lock check: missing {}\n",
            lock.display()
        )));
    }
    let expected =
        sha256_hex(&fs::read(&catalog).with_context(|| format!("read {}", catalog.display()))?);
    let actual = read_utf8(&lock)?.trim().to_string();
    if expected != actual {
        return Ok(DomainCommandOutcome::failure(format!(
            "reference bundle lock drift: {} is stale; update it after bundle changes\nexpected={expected}\nactual={actual}\n",
            lock.display()
        )));
    }

    let mut stdout = String::from("reference bundle lock: OK\n");
    if materialization_lock_json.is_file() || materialization_lock_sha.is_file() {
        if !materialization_lock_json.is_file() {
            return Ok(DomainCommandOutcome::failure(format!(
                "reference materialization lock check: missing {}\n",
                materialization_lock_json.display()
            )));
        }
        if !materialization_lock_sha.is_file() {
            return Ok(DomainCommandOutcome::failure(format!(
                "reference materialization lock check: missing {}\n",
                materialization_lock_sha.display()
            )));
        }
        let expected = sha256_hex(
            &fs::read(&materialization_lock_json)
                .with_context(|| format!("read {}", materialization_lock_json.display()))?,
        );
        let actual = read_utf8(&materialization_lock_sha)?
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        if expected != actual {
            return Ok(DomainCommandOutcome::failure(format!(
                "reference materialization lock drift: {} is stale\nexpected={expected}\nactual={actual}\n",
                materialization_lock_sha.display()
            )));
        }
        stdout.push_str("reference materialization lock: OK\n");
    }
    Ok(DomainCommandOutcome::success(stdout))
}

pub(super) fn lock_registry(
    workspace: &Workspace,
    args: &[String],
) -> Result<DomainCommandOutcome> {
    let print_only = match args {
        [] => false,
        [single] if single == "--print" => true,
        [single] if single == "--help" || single == "-h" => {
            return Ok(DomainCommandOutcome::success(
                "Usage: cargo run -p bijux-dna-dev -- domain run lock-registry -- [--print]\n",
            ));
        }
        _ => {
            return Ok(DomainCommandOutcome {
                exit_code: 2,
                stdout: String::new(),
                stderr: "unknown arg\n".to_string(),
            });
        }
    };

    let lock_doc = workspace.path("configs/ci/registry/LOCK_RULES.md");
    if !lock_doc.is_file() {
        bail!("missing {}", lock_doc.display());
    }
    let inputs = [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
        "configs/ci/registry/domains.toml",
        "configs/ci/registry/deprecations.toml",
    ];
    let mut payload = String::new();
    for rel in inputs {
        let path = workspace.path(rel);
        let sha = sha256_hex(&fs::read(&path).with_context(|| format!("read {}", path.display()))?);
        payload.push_str(rel);
        payload.push(' ');
        payload.push_str(&sha);
        payload.push('\n');
    }
    let lock_sha = sha256_hex(payload.as_bytes());
    if print_only {
        return Ok(DomainCommandOutcome::success(format!("{lock_sha}\n")));
    }

    let lock_file = workspace.path("configs/ci/registry/tool_registry_lock.sha256");
    let marker_file = workspace.path("artifacts/configs/tool_registry_lock.marker");
    write_utf8(&lock_file, &format!("{lock_sha}\n"))?;
    write_utf8(
        &marker_file,
        &format!("{REGISTRY_LOCK_GENERATED_BY}\nlock_sha256={lock_sha}\n"),
    )?;
    success_line(format!(
        "updated {} (rules: configs/ci/registry/LOCK_RULES.md)",
        lock_file.display()
    ))
}
