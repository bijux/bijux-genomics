//! Versioned external-artifact wire admission without filesystem or process effects.
use super::super::EvidenceSourceIdentity;
use super::model::{
    ExternalArtifactAdmission, ExternalArtifactAdmissionStatus, ExternalArtifactIdentity,
    ExternalArtifactKind, ExternalArtifactVersionProbe, EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V1,
    EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2,
};
use anyhow::{bail, Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyExternalArtifactAdmissionV1 {
    schema_version: String,
    status: ExternalArtifactAdmissionStatus,
    recorded_at_utc: String,
    repository_commit: String,
    catalog: EvidenceSourceIdentity,
    distributable_image_root: String,
    tool_id: String,
    version: String,
    support_disposition: String,
    support_reason: String,
    stage_ids: Vec<String>,
    expected_binary: String,
    version_command: String,
    expected_version_regex: String,
    redistribution_permitted: bool,
    lawful_access_basis: Option<String>,
    artifact: Option<ExternalArtifactIdentity>,
    version_probe: Option<ExternalArtifactVersionProbe>,
    blocker_reason: Option<String>,
}

pub(super) fn decode_external_artifact_admission(
    bytes: &[u8],
) -> Result<ExternalArtifactAdmission> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).context("parse external-artifact admission JSON")?;
    let schema_version = value
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
        .context("external-artifact admission lacks a schema version")?;
    let admission = if schema_version == EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V1 {
        migrate_external_artifact_admission_v1(value)?
    } else {
        serde_json::from_value(value).context("parse external-artifact admission")?
    };
    validate_external_artifact_admission(&admission)?;
    Ok(admission)
}

fn migrate_external_artifact_admission_v1(
    value: serde_json::Value,
) -> Result<ExternalArtifactAdmission> {
    if value.get("status").is_none() {
        let mut admission: ExternalArtifactAdmission = serde_json::from_value(value)
            .context("parse mis-versioned external-artifact admission")?;
        admission.schema_version = EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2.to_string();
        return Ok(admission);
    }
    let legacy: LegacyExternalArtifactAdmissionV1 =
        serde_json::from_value(value).context("parse v1 external-artifact admission")?;
    if legacy.schema_version != EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V1
        || legacy.support_disposition != "unsupported"
        || legacy.redistribution_permitted
        || legacy.status == ExternalArtifactAdmissionStatus::Unavailable
    {
        bail!("legacy external-artifact admission has invalid support semantics");
    }
    Ok(ExternalArtifactAdmission {
        schema_version: EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2.to_string(),
        availability: legacy.status,
        recorded_at_utc: legacy.recorded_at_utc,
        repository_commit: legacy.repository_commit,
        catalog: legacy.catalog,
        distributable_image_root: legacy.distributable_image_root,
        tool_id: legacy.tool_id,
        version: legacy.version,
        product_support: crate::evidence::ContainerProductSupport::Supported,
        artifact_delivery: crate::evidence::ContainerArtifactDelivery::ExternalArtifact,
        delivery_reason: legacy.support_reason,
        stage_ids: legacy.stage_ids,
        expected_binary: legacy.expected_binary,
        version_command: legacy.version_command,
        expected_version_regex: legacy.expected_version_regex,
        redistribution_permitted: false,
        lawful_access_basis: legacy.lawful_access_basis,
        artifact: legacy.artifact,
        version_probe: legacy.version_probe,
        blocker_reason: legacy.blocker_reason,
    })
}

pub(super) fn validate_persisted_version_probe(
    admission: &ExternalArtifactAdmission,
    artifact: &ExternalArtifactIdentity,
) -> Result<()> {
    let probe = admission.version_probe.as_ref().context("admission omits version probe")?;
    let tokens = admission.version_command.split_ascii_whitespace().collect::<Vec<_>>();
    let [program, version_arguments @ ..] = tokens.as_slice() else {
        bail!("external artifact version command is empty");
    };
    if *program != admission.expected_binary {
        bail!("external artifact version command does not invoke the expected binary");
    }
    let valid_command = match artifact.kind {
        ExternalArtifactKind::Binary => {
            probe.command.first().is_some_and(|value| value == &artifact.path)
                && probe.command[1..]
                    == version_arguments
                        .iter()
                        .map(|argument| (*argument).to_string())
                        .collect::<Vec<_>>()
        }
        ExternalArtifactKind::ApptainerSif => {
            probe.command.len() >= 4
                && probe.command[1] == "exec"
                && probe.command[2] == artifact.path
                && probe.command[3] == admission.expected_binary
                && probe.command[4..]
                    == version_arguments
                        .iter()
                        .map(|argument| (*argument).to_string())
                        .collect::<Vec<_>>()
        }
    };
    let expected = Regex::new(&admission.expected_version_regex)
        .context("compile persisted external artifact version contract")?;
    if !valid_command || !expected.is_match(&probe.output) {
        bail!("external artifact version probe evidence changed");
    }
    Ok(())
}

pub(super) fn validate_external_artifact_admission(
    admission: &ExternalArtifactAdmission,
) -> Result<()> {
    if admission.schema_version != EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2 {
        bail!("unsupported external-artifact admission schema");
    }
    require_timestamp(&admission.recorded_at_utc)?;
    validate_sha256(&admission.catalog.sha256, "external artifact catalog")?;
    if admission.catalog.path != "containers/contracts/catalog.toml" {
        bail!("external-artifact catalog path is not canonical");
    }
    if admission.repository_commit.len() != 40
        || !admission.repository_commit.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("external-artifact repository commit is invalid");
    }
    if admission.tool_id.trim().is_empty()
        || admission.version.trim().is_empty()
        || admission.product_support != crate::evidence::ContainerProductSupport::Supported
        || admission.artifact_delivery
            == crate::evidence::ContainerArtifactDelivery::RedistributableImage
        || admission.delivery_reason.trim().is_empty()
        || admission.distributable_image_root.trim().is_empty()
        || !Path::new(&admission.distributable_image_root).is_absolute()
        || admission.stage_ids.is_empty()
        || admission.expected_binary.trim().is_empty()
        || admission.version_command.trim().is_empty()
        || admission.expected_version_regex.trim().is_empty()
        || admission.redistribution_permitted
    {
        bail!("external-artifact admission has an incomplete or distributable tool contract");
    }
    let mut stages = admission.stage_ids.clone();
    stages.sort();
    stages.dedup();
    if stages != admission.stage_ids {
        bail!("external-artifact stage bindings must be sorted and unique");
    }
    match admission.availability {
        ExternalArtifactAdmissionStatus::Admitted => {
            let basis = admission.lawful_access_basis.as_deref().unwrap_or_default();
            let artifact = admission.artifact.as_ref().context("admission omits artifact")?;
            let probe =
                admission.version_probe.as_ref().context("admission omits version probe")?;
            if basis.trim().is_empty() || admission.blocker_reason.is_some() {
                bail!("admitted external artifact has invalid access state");
            }
            validate_sha256(&artifact.sha256, "external artifact")?;
            validate_sha256(&probe.output_sha256, "external artifact version output")?;
            if artifact.path.trim().is_empty()
                || !Path::new(&artifact.path).is_absolute()
                || artifact.size_bytes == 0
                || probe.command.is_empty()
                || probe.output.trim().is_empty()
                || bijux_dna_infra::hash_bytes_sha256(probe.output.as_bytes())
                    != probe.output_sha256
            {
                bail!("admitted external artifact has incomplete identity evidence");
            }
        }
        ExternalArtifactAdmissionStatus::BlockedExternalAccess => {
            if admission.artifact.is_some()
                || admission.version_probe.is_some()
                || admission.lawful_access_basis.is_some()
                || admission.blocker_reason.as_deref().unwrap_or_default().trim().is_empty()
            {
                bail!("blocked external artifact has invalid access state");
            }
        }
        ExternalArtifactAdmissionStatus::Unavailable => {
            if admission.artifact.is_some()
                || admission.version_probe.is_some()
                || admission.lawful_access_basis.is_some()
                || admission.blocker_reason.as_deref().unwrap_or_default().trim().is_empty()
            {
                bail!("unavailable external artifact has invalid access state");
            }
        }
    }
    Ok(())
}

pub(super) fn validate_sha256(value: &str, label: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} SHA-256 identity is invalid");
    }
    Ok(())
}

pub(super) fn require_timestamp(value: &str) -> Result<()> {
    if value.trim().is_empty() || !value.ends_with('Z') || !value.contains('T') {
        bail!("external-artifact timestamp is not UTC RFC 3339");
    }
    Ok(())
}
