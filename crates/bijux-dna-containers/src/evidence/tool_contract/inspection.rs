use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};
use serde_json::Value;
use sha2::{Digest, Sha256};

use super::labels::{validate_tool_contract_labels, ToolContractLabels};
use super::model::{
    ContainerToolContract, ToolImageIdentityEvidence, CONTAINER_TOOL_CONTRACT_PATH,
    TOOL_IMAGE_IDENTITY_EVIDENCE_SCHEMA_V2,
};
use super::validation::{tool_contract_is_compatible, validate_sha256, validate_tool_contract};

/// Read and verify the contract inside one immutable Apptainer image.
///
/// # Errors
/// Returns an error when execution, inspection, image digest, JSON, labels, or expected identity
/// do not match exactly.
pub fn verify_apptainer_tool_contract(
    apptainer: &Path,
    image: &Path,
    expected_image_sha256: &str,
    expected_contract: &ContainerToolContract,
) -> Result<ToolImageIdentityEvidence> {
    validate_sha256(expected_image_sha256, "image")?;
    validate_tool_contract(expected_contract)?;
    let image_sha256 = sha256_file(image)?;
    if image_sha256 != expected_image_sha256 {
        bail!(
            "Apptainer image digest mismatch: expected {expected_image_sha256}, observed {image_sha256}"
        );
    }
    let contract_output = Command::new(apptainer)
        .args(["exec", "--cleanenv", "--containall"])
        .arg(image)
        .args(["cat", CONTAINER_TOOL_CONTRACT_PATH])
        .output()
        .with_context(|| format!("execute embedded contract inspection for {}", image.display()))?;
    if !contract_output.status.success() {
        bail!(
            "embedded tool-contract inspection failed for {}: {}",
            image.display(),
            String::from_utf8_lossy(&contract_output.stderr).trim()
        );
    }
    let observed: ContainerToolContract = serde_json::from_slice(&contract_output.stdout)
        .with_context(|| format!("parse embedded tool contract from {}", image.display()))?;
    validate_tool_contract(&observed)?;
    if !tool_contract_is_compatible(&observed, expected_contract) {
        bail!("embedded tool contract differs from expected build identity");
    }

    let inspect_output = Command::new(apptainer)
        .args(["inspect", "--json"])
        .arg(image)
        .output()
        .with_context(|| format!("inspect Apptainer labels for {}", image.display()))?;
    if !inspect_output.status.success() {
        bail!(
            "Apptainer label inspection failed for {}: {}",
            image.display(),
            String::from_utf8_lossy(&inspect_output.stderr).trim()
        );
    }
    let inspection: Value = serde_json::from_slice(&inspect_output.stdout)
        .with_context(|| format!("parse Apptainer inspection for {}", image.display()))?;
    let labels = collect_contract_labels(&inspection);
    validate_tool_contract_labels(&observed, &labels)?;
    Ok(ToolImageIdentityEvidence {
        schema_version: TOOL_IMAGE_IDENTITY_EVIDENCE_SCHEMA_V2.to_string(),
        image_sha256,
        contract: observed,
        labels,
    })
}

fn collect_contract_labels(value: &Value) -> ToolContractLabels {
    let mut labels = BTreeMap::new();
    collect_contract_labels_inner(value, &mut labels);
    labels
}

fn collect_contract_labels_inner(value: &Value, labels: &mut ToolContractLabels) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key.starts_with("io.bijux.tool-contract.") {
                    if let Some(text) = value.as_str() {
                        labels.insert(key.clone(), text.to_string());
                    }
                } else {
                    collect_contract_labels_inner(value, labels);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_contract_labels_inner(item, labels);
            }
        }
        _ => {}
    }
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("open image {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count =
            file.read(&mut buffer).with_context(|| format!("read image {}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let digest = hasher.finalize();
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(output, "{byte:02x}");
    }
    Ok(output)
}
