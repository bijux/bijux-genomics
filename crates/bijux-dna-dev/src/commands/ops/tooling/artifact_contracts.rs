use super::{fs, Context, OpsCommandOutcome, Result, Workspace};

const CATALOG_PATH: &str = "domain/contracts/artifact-contracts.json";
const CONTRACT_SCHEMA_PATH: &str = "domain/contracts/schemas/artifact-contract.schema.json";
const CATALOG_SCHEMA_PATH: &str = "domain/contracts/schemas/artifact-contract-catalog.schema.json";

pub(in super::super) fn tooling_artifact_contracts(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    let check = match args {
        [] => false,
        [flag] if flag == "--check" => true,
        _ => {
            return Ok(OpsCommandOutcome::failure(
                "Usage: cargo run -p bijux-dna-dev -- tooling run artifact-contracts -- [--check]\n",
            ));
        }
    };
    let catalog = bijux_dna_stage_contract::canonical_artifact_contract_catalog();
    catalog.validate().context("validate canonical artifact contract catalog")?;
    let outputs = [
        (CATALOG_PATH, serde_json::to_value(&catalog)?),
        (
            CONTRACT_SCHEMA_PATH,
            serde_json::to_value(bijux_dna_stage_contract::artifact_contract_schema())?,
        ),
        (
            CATALOG_SCHEMA_PATH,
            serde_json::to_value(bijux_dna_stage_contract::artifact_contract_catalog_schema())?,
        ),
    ];
    for (relative, value) in outputs {
        let bytes = format!("{}\n", serde_json::to_string_pretty(&value)?).into_bytes();
        let path = workspace.path(relative);
        if check {
            let current = fs::read(&path)
                .with_context(|| format!("read generated artifact contract {relative}"))?;
            if current != bytes {
                return Ok(OpsCommandOutcome::failure(format!(
                    "generated artifact contract drifted: {relative}\n"
                )));
            }
        } else {
            if let Some(parent) = path.parent() {
                bijux_dna_infra::ensure_dir(parent)?;
            }
            bijux_dna_infra::atomic_write_bytes(&path, &bytes)?;
        }
    }
    Ok(OpsCommandOutcome::success(format!(
        "artifact contracts: {} canonical contracts ({})\n",
        catalog.contracts.len(),
        if check { "checked" } else { "written" }
    )))
}
