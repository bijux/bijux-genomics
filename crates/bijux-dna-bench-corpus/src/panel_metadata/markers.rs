//! Plan exact metadata marker bytes separately from destination admission and effects.
pub(super) use super::marker_admission::admit_markers;
use super::marker_admission::{admit_component, admit_relative};
use super::model::{ResolvedVcfPanelAssetContract, ResolvedVcfPanelDerivationContract};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub(super) struct Marker {
    pub(super) path: PathBuf,
    pub(super) payload: String,
}

pub(super) fn plan_markers(
    root: &Path,
    contract: &ResolvedVcfPanelAssetContract,
) -> Result<Vec<Marker>> {
    admit_component(&contract.panel_id)?;
    admit_component(&contract.map_id)?;
    let panels = root.join("fixtures/panels").join(&contract.panel_id);
    let maps = root.join("fixtures/maps").join(&contract.map_id);
    let mut markers = Vec::new();
    for relative in &contract.panel_files {
        admit_relative(Path::new(relative))?;
        let payload = contract
            .panel_derivations
            .iter()
            .find(|row| row.path == *relative)
            .map_or_else(|| marker_payload("panel", relative), derived_payload);
        markers.push(Marker { path: panels.join(relative), payload });
    }
    if !contract.panel_files.iter().any(|path| path.ends_with("panel.msav")) {
        markers.push(Marker {
            path: panels.join("panel.msav"),
            payload: derived_payload(&minimac_marker()),
        });
    }
    for relative in &contract.map_files {
        admit_relative(Path::new(relative))?;
        markers
            .push(Marker { path: maps.join(relative), payload: marker_payload("map", relative) });
    }
    Ok(markers)
}

fn marker_payload(kind: &str, relative: &str) -> String {
    format!("fixture_class=local_smoke\nasset_kind={kind}\nrelative_path={relative}\n")
}

fn derived_payload(derivation: &ResolvedVcfPanelDerivationContract) -> String {
    format!(
        "fixture_class=local_smoke\nasset_kind=derived_panel\nderived_from={}\nderivation_tool={}\nderivation_tool_version={}\nderivation_tool_artifact_sha256={}\nderivation_argv={}\n",
        derivation.source_file_name, derivation.tool_id, derivation.tool_version,
        derivation.tool_artifact_sha256, derivation.argv.join(" "),
    )
}

fn minimac_marker() -> ResolvedVcfPanelDerivationContract {
    ResolvedVcfPanelDerivationContract {
        path: "panel.msav".to_string(),
        source_file_name: "panel_vcf".to_string(),
        tool_id: "minimac4".to_string(),
        tool_version: "4.1.6".to_string(),
        tool_artifact_sha256: "e627d6955c1c103caa0bd2ccc2ecf674cafc4a974694fc38b91c0a6bf000b478"
            .to_string(),
        argv: [
            "minimac4",
            "--compress-reference",
            "panel.vcf.gz",
            "--output",
            "panel.msav",
            "--threads",
            "8",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    }
}
