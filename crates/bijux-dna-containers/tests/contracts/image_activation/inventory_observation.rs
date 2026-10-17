//! The activation plan must bind the exact bytes used to select its publications.
use super::{
    create_image_activation_plan, ContainerImageInventoryPublication, ContainerImageState, Fixture,
    ImageActivationPlanRequest,
};
use crate::inventory_capture::{InventoryCapture, InventoryWriter};
use anyhow::{ensure, Result};
use std::collections::BTreeMap;
use std::path::Path;

const CASE: &str = "mod_contracts_image_activation_rs::inventory_observation::activation_inventory_identity_matches_captured_bytes";

#[test]
fn activation_inventory_identity_matches_captured_bytes() -> Result<()> {
    if crate::inventory_capture::write_inventory_child_request(
        CASE,
        Path::new("STUDY/containers/evidence/activation/commit"),
        validate_capture,
    )? {
        return Ok(());
    }
    let fixture = Fixture::new()?;
    let captured = std::fs::read(&fixture.inventory)?;
    let mut replacement: ContainerImageInventoryPublication = serde_json::from_slice(&captured)?;
    replacement.catalog_sha256 = "d".repeat(64);
    replacement.summary.states = BTreeMap::from([("missing".to_string(), 3)]);
    for tool in &mut replacement.tools {
        tool.state = ContainerImageState::Missing;
        tool.selected_image_sha256 = None;
        tool.candidates.clear();
    }
    let replacement = serde_json::to_vec_pretty(&replacement)?;
    let mut writer = InventoryWriter::start(
        &fixture.inventory,
        &captured,
        &replacement,
        InventoryCapture::Valid,
        CASE,
    )?;
    let planned = fixture.create_plan();
    writer.finish()?;
    let plan = planned?;
    ensure!(plan.file_count == 2, "actual captured activation fixture selects its two images");
    ensure!(
        std::fs::read(fixture.staging.join("alpha.sif"))? == b"alpha"
            && std::fs::read(fixture.staging.join("beta.sif"))? == b"beta",
        "source images unchanged"
    );
    ensure!(
        std::fs::read(&fixture.inventory)? == replacement && captured != replacement,
        "replacement must be published before reader EOF"
    );
    let Err(fresh) = create_image_activation_plan(
        &ImageActivationPlanRequest {
            inventory_path: &fixture.inventory,
            study_root: &fixture.study,
            source_root: &fixture.staging,
            active_root: &fixture.active,
            created_at_utc: "2026-08-05T10:00:00Z",
        },
        &fixture.plan.with_file_name("fresh-image-activation-plan.json"),
    ) else {
        anyhow::bail!("empty replacement inventory unexpectedly produced an activation plan");
    };
    ensure!(
        fresh.to_string() == "image-activation inventory selects no retained images",
        "actual replacement public facade refusal control: {fresh:#}"
    );
    eprintln!("ACTIVATION_INVENTORY_CAPTURE_CONTROLS_PASS: selected=2, replacement=0, writer receipt complete; captured_plan_digest={}", plan.inventory.sha256);
    ensure!(
        plan.inventory.sha256 == bijux_dna_infra::hash_bytes_sha256(&captured),
        "activation inventory digest must bind the captured selection bytes, not replacement path"
    );
    Ok(())
}

fn validate_capture(
    captured_path: &Path,
    replacement_path: &Path,
    _captured: &[u8],
    scenario: InventoryCapture,
) -> Result<()> {
    ensure!(scenario == InventoryCapture::Valid, "activation captured scenario");
    let original = bijux_dna_containers::api::load_image_inventory_publication(captured_path)?;
    ensure!(
        original.summary.tool_count == 3 && original.summary.states.get("current") == Some(&2),
        "actual captured current-image control"
    );
    let fresh = bijux_dna_containers::api::load_image_inventory_publication(replacement_path)?;
    ensure!(
        fresh.summary.states == BTreeMap::from([("missing".to_string(), 3)])
            && fresh.tools.iter().all(
                |tool| tool.state == ContainerImageState::Missing && tool.candidates.is_empty()
            ),
        "actual replacement must select no current image"
    );
    Ok(())
}
