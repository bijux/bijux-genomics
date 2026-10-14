use std::path::Path;
use std::time::{Duration, SystemTime};

use anyhow::{ensure, Context, Result};

use super::{
    apply_heap_policy, effective_probe_commands, validate_request, ApptainerSmokeManifest,
    ApptainerSmokeRequest, APPTAINER_SMOKE_MANIFEST_SCHEMA_V6,
};

/// Reuse only current successful smoke observations for the exact requested image and probes.
///
/// An unreadable, unsupported, failed, stale, or mismatched receipt requires a fresh smoke.
/// This does not execute the runtime or authenticate the executable provider.
#[must_use]
pub fn can_reuse_apptainer_smoke(
    path: &Path,
    request: &ApptainerSmokeRequest,
    now: SystemTime,
) -> bool {
    admit_receipt(path, request, now).is_ok()
}

fn admit_receipt(path: &Path, request: &ApptainerSmokeRequest, now: SystemTime) -> Result<()> {
    validate_request(request)?;
    let metadata = path.metadata().context("read smoke receipt metadata")?;
    ensure!(metadata.is_file(), "smoke receipt is not a file");
    let age = now.duration_since(metadata.modified()?).context("future smoke receipt mtime")?;
    ensure!(age < Duration::from_hours(168), "smoke receipt mtime is stale");
    let receipt: ApptainerSmokeManifest =
        serde_json::from_slice(&std::fs::read(path)?).context("decode smoke receipt")?;
    ensure!(
        receipt.schema_version == APPTAINER_SMOKE_MANIFEST_SCHEMA_V6,
        "unsupported smoke receipt schema"
    );
    let checked = SystemTime::UNIX_EPOCH
        .checked_add(Duration::from_secs(receipt.checked_at_unix_s))
        .context("smoke checked timestamp overflow")?;
    let checked_age = now.duration_since(checked).context("future smoke check")?;
    ensure!(
        receipt.checked_at_unix_s != 0 && checked_age < Duration::from_hours(168),
        "smoke check is stale or absent"
    );
    ensure!(
        receipt.status == "ok" && receipt.quick_smoke && receipt.version_matches_contract,
        "smoke receipt is not a successful quick smoke"
    );
    for (observed, expected, label) in [
        (&receipt.tool_id, &request.tool_id, "tool"),
        (&receipt.stage_id, &request.stage_id, "stage"),
        (&receipt.build_evidence_id, &request.build_evidence_id, "build"),
        (&receipt.repository_commit, &request.repository_commit, "repository"),
        (&receipt.registry_digest, &request.registry_digest, "registry"),
        (&receipt.sif_sha256, &request.image_sha256, "image"),
        (&receipt.version_cmd, &request.version_command, "version command"),
        (&receipt.expected_version_regex, &request.expected_version_regex, "version contract"),
        (&receipt.help_cmd, &request.help_command, "help command"),
        (&receipt.upstream, &request.upstream, "upstream"),
    ] {
        ensure!(observed == expected, "smoke receipt {label} identity differs");
    }
    ensure!(receipt.java_heap_mb == request.java_heap_mb, "smoke heap policy differs");
    ensure!(!receipt.version.is_empty(), "smoke receipt has no matched version");
    ensure!(receipt.help_ok, "smoke receipt help policy failed");
    let commands = effective_probe_commands(request);
    ensure!(
        receipt.probe_commands == commands && receipt.probe_results.len() == commands.len(),
        "smoke receipt probe inventory differs"
    );
    for (probe, command) in receipt.probe_results.iter().zip(&commands) {
        ensure!(
            probe.command == *command
                && probe.applied_command
                    == apply_heap_policy(&request.tool_id, command, request.java_heap_mb),
            "smoke receipt probe command differs"
        );
        ensure!(
            probe.ok && probe.exit_code == Some(0) && !probe.timed_out && probe.error.is_none(),
            "smoke receipt contains a failed or incomplete probe"
        );
    }
    Ok(())
}
