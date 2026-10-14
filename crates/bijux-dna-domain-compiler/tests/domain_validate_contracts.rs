use bijux_dna_domain_compiler::{validate_domain, ValidateOptions, DEFAULT_DOMAIN_DIR};

#[path = "support/mod.rs"]
mod support;

#[test]
fn validate_domain_accepts_governed_workspace_contracts() -> anyhow::Result<()> {
    let root = support::repo_root();
    validate_domain(&ValidateOptions { domain_dir: root.join(DEFAULT_DOMAIN_DIR) })?;
    Ok(())
}

#[path = "contracts/resource_quantities.rs"]
mod resource_quantities;

#[path = "support/domain_workspace.rs"]
mod domain_workspace;

#[path = "contracts/deprecations.rs"]
mod deprecations;
