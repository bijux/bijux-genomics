//! Storage fixture admission delegates byte transport to the shared inventory capture owner.
use super::{write_inventory, Fixture};
use anyhow::{ensure, Result};
use std::path::Path;

pub(super) use crate::inventory_capture::InventoryCapture;

const COHERENCE_CASE: &str = "mod_contracts_image_storage_reconciliation_rs::plan_inventory_identity_matches_captured_bytes_after_path_replacement";

pub(super) fn write_inventory_child_request() -> Result<bool> {
    crate::inventory_capture::write_inventory_child_request(
        COHERENCE_CASE,
        &Path::new("STUDY/containers/evidence/inventory").join("a".repeat(40)),
        validate_capture,
    )
}

fn validate_capture(
    captured_path: &Path,
    replacement_path: &Path,
    captured: &[u8],
    scenario: InventoryCapture,
) -> Result<()> {
    match scenario {
        InventoryCapture::Valid => {
            let original =
                bijux_dna_containers::api::load_image_inventory_publication(captured_path)?;
            ensure!(
                original.summary.tool_count == 3
                    && original.summary.unreferenced_obsolete_count == 2
                    && original.summary.historical_dependency_count == 2
                    && original.summary.staged_image_count == 1
                    && original.summary.unknown_image_count == 1,
                "unexpected captured inventory scenario"
            );
        }
        InventoryCapture::MalformedJson => {
            ensure!(captured == b"{malformed\n", "unexpected malformed writer scenario");
        }
        InventoryCapture::NonUtf8 => {
            ensure!(captured == b"\xff\xfe", "unexpected non-UTF8 writer scenario");
        }
    }
    let fresh = bijux_dna_containers::api::load_image_inventory_publication(replacement_path)?;
    ensure!(
        fresh.summary.unreferenced_obsolete_count == 0
            && fresh.summary.historical_dependency_count == 0
            && fresh.summary.staged_image_count == 0
            && fresh.summary.unknown_image_count == 0,
        "replacement must be the no-move fixture"
    );
    Ok(())
}

pub(super) struct InventoryWriter(crate::inventory_capture::InventoryWriter);

impl InventoryWriter {
    pub(super) fn start(
        fixture: &Fixture,
        captured: &[u8],
        scenario: InventoryCapture,
    ) -> Result<Self> {
        let replacement_path = fixture.inventory.with_file_name("inventory-replacement.json");
        write_inventory(&replacement_path, fixture, false)?;
        let replacement = std::fs::read(&replacement_path)?;
        Ok(Self(crate::inventory_capture::InventoryWriter::start(
            &fixture.inventory,
            captured,
            &replacement,
            scenario,
            COHERENCE_CASE,
        )?))
    }

    pub(super) fn finish(&mut self) -> Result<()> {
        self.0.finish()
    }
}
