use super::ParseKind;
use crate::values::{ensure_governed_keys_present, read_json_document};
use anyhow::Result;
use serde_json::Value;
use std::path::Path;

pub(super) fn parse(proof_path: &Path, kind: ParseKind) -> Result<Value> {
    match kind {
        ParseKind::VcfCallMetrics => {
            let value = read_json_document(proof_path)?;
            ensure_governed_keys_present(
                &value,
                proof_path,
                &["schema_version", "stage_id", "tool_id", "variant_count", "sample_count"],
            )?;
        }
        ParseKind::VcfDemographyReport => {
            let value = read_json_document(proof_path)?;
            ensure_governed_keys_present(
                &value,
                proof_path,
                &[
                    "schema_version",
                    "stage_id",
                    "tool_id",
                    "method",
                    "inference_status",
                    "time_bins",
                    "ne_estimates",
                    "status",
                ],
            )?;
        }
        ParseKind::VcfIbdReport => {
            let value = read_json_document(proof_path)?;
            ensure_governed_keys_present(
                &value,
                proof_path,
                &[
                    "schema_version",
                    "stage_id",
                    "tool_id",
                    "execution_mode",
                    "pair_count",
                    "rows",
                    "status",
                ],
            )?;
        }
        ParseKind::VcfImputeMetrics => {
            let value = read_json_document(proof_path)?;
            ensure_governed_keys_present(
                &value,
                proof_path,
                &["schema_version", "stage_id", "tool_id", "variant_count", "imputed_genotypes"],
            )?;
        }
        ParseKind::VcfPhasingMetrics => {
            let value = read_json_document(proof_path)?;
            ensure_governed_keys_present(
                &value,
                proof_path,
                &["schema_version", "stage_id", "tool_id", "phased_genotypes", "phase_set_count"],
            )?;
        }
        ParseKind::VcfPopulationStructureReport => {
            let value = read_json_document(proof_path)?;
            ensure_governed_keys_present(
                &value,
                proof_path,
                &["schema_version", "tool_id", "status", "sample_groups", "distance_summary"],
            )?;
        }
        ParseKind::VcfQcMetrics => {
            let value = read_json_document(proof_path)?;
            ensure_governed_keys_present(
                &value,
                proof_path,
                &[
                    "schema_version",
                    "tool_id",
                    "sample_missingness",
                    "heterozygosity",
                    "hwe_summary",
                ],
            )?;
        }
        _ => return Err(anyhow::anyhow!("not a VCF parser kind")),
    }
    read_json_document(proof_path)
}
