use std::collections::BTreeSet;

use anyhow::{anyhow, bail, Result};
use bijux_dna_domain_vcf::{contracts::stage_metrics_contract, VcfDomainStage};

const GOVERNED_NORMALIZED_METADATA_KEYS: &[&str] = &["schema_version", "stage_id", "tool_id"];

pub(super) fn ensure_registered_stage_metrics(
    stage: VcfDomainStage,
    observed_fixture: &serde_json::Value,
) -> Result<()> {
    let normalized = observed_fixture
        .get("normalized")
        .ok_or_else(|| anyhow!("VCF parser fixture payload is missing `normalized`"))?;
    ensure_registered_normalized_metrics(stage, normalized)
}

pub(super) fn ensure_registered_normalized_metrics(
    stage: VcfDomainStage,
    normalized: &serde_json::Value,
) -> Result<()> {
    let normalized_object = normalized.as_object().ok_or_else(|| {
        anyhow!(
            "VCF parser fixture for stage `{}` must normalize into a JSON object",
            stage.as_str()
        )
    })?;
    let registered_metrics = stage_metrics_contract(stage)
        .required_metrics
        .iter()
        .map(|metric_id| (*metric_id).to_string())
        .chain(GOVERNED_NORMALIZED_METADATA_KEYS.iter().map(|metric_id| (*metric_id).to_string()))
        .collect::<BTreeSet<_>>();
    let observed_metrics =
        normalized_object.keys().map(|metric_id| metric_id.to_string()).collect::<BTreeSet<_>>();
    let unregistered_metrics =
        observed_metrics.difference(&registered_metrics).cloned().collect::<Vec<_>>();

    if unregistered_metrics.is_empty() {
        return Ok(());
    }

    bail!(
        "VCF parser emitted unregistered metrics for stage `{}`: {}",
        stage.as_str(),
        unregistered_metrics.join(", ")
    );
}

#[test]
fn vcf_metric_registry_rejects_unregistered_stage_metrics() {
    let err = ensure_registered_normalized_metrics(
        VcfDomainStage::Stats,
        &serde_json::json!({
            "variant_count": 4,
            "snp_count": 3,
            "indel_count": 1,
            "transition_count": 2,
            "transversion_count": 1,
            "ti_tv": 2.0,
            "sample_count": 1,
            "unexpected_metric": 7
        }),
    )
    .expect_err("unexpected metric must be rejected");

    assert!(err.to_string().contains("unexpected_metric"), "{err}");
}
