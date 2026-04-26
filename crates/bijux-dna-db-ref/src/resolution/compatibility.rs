use anyhow::{bail, Result};

use crate::{MapCatalogEntry, PanelCatalogEntry};

/// # Errors
/// Returns an error if tool compatibility requirements are not satisfied.
pub fn validate_imputation_tool_compatibility(
    tool_id: &str,
    panel: &PanelCatalogEntry,
    map: &MapCatalogEntry,
) -> Result<()> {
    if panel.species_id != map.species_id || panel.build_id != map.build_id {
        bail!(
            "panel {} ({}/{}) is not compatible with map {} ({}/{})",
            panel.id,
            panel.species_id,
            panel.build_id,
            map.id,
            map.species_id,
            map.build_id
        );
    }
    if !panel.compatibility.tool_tags.iter().any(|tag| tag == tool_id) {
        bail!("panel {} not compatible with tool {}", panel.id, tool_id);
    }
    if !map.compatibility.tool_tags.iter().any(|tag| tag == tool_id) {
        bail!("map {} not compatible with tool {}", map.id, tool_id);
    }
    if tool_id == "minimac4" && !panel.compatibility.supports_minimac_m3vcf {
        bail!("minimac4 requires m3vcf-compatible panel representation");
    }
    if tool_id == "minimac4" && !panel.files.iter().any(|file| file.name == "panel_m3vcf") {
        bail!("minimac4 requires `panel_m3vcf` in panel files");
    }
    if tool_id == "glimpse" && panel.compatibility.glimpse_reference_format.trim().is_empty() {
        bail!("GLIMPSE requires declared reference format");
    }
    if tool_id == "glimpse"
        && !matches!(
            panel.compatibility.glimpse_reference_format.as_str(),
            "bcf+sites" | "bcf" | "sites"
        )
    {
        bail!("GLIMPSE requires supported reference format (bcf+sites|bcf|sites)");
    }
    if matches!(tool_id, "impute5" | "minimac4") && map.compatibility.coordinate_system != "bp" {
        bail!("{tool_id} requires bp coordinate-system genetic map");
    }
    if tool_id == "impute5" && !panel.compatibility.requires_phased {
        bail!("impute5 requires phased panel compatibility");
    }
    if tool_id == "beagle" && !panel.compatibility.supports_gl_input {
        bail!("beagle requires panel compatibility with GL input");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_imputation_tool_compatibility;
    use crate::{
        CatalogCompatibility, CatalogFileEntry, MapCatalogEntry, MapCompatibility,
        PanelCatalogEntry,
    };

    #[test]
    fn validate_imputation_tool_compatibility_rejects_panel_map_build_mismatch() {
        let panel = PanelCatalogEntry {
            id: "panel".to_string(),
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            status: "production".to_string(),
            version: "1.0.0".to_string(),
            license: "CC-BY-4.0".to_string(),
            lock_ref: "locks/panel_locks.toml#locks.panel".to_string(),
            citation: None,
            files: vec![CatalogFileEntry {
                name: "panel_m3vcf".to_string(),
                path: "panel.m3vcf.gz".to_string(),
                format: "m3vcf.gz".to_string(),
                url: "https://example.org/panel.m3vcf.gz".to_string(),
                checksum_sha256: "a".repeat(64),
                required: true,
            }],
            compatibility: CatalogCompatibility {
                tool_tags: vec!["minimac4".to_string()],
                requires_phased: true,
                supports_gl_input: true,
                supports_minimac_m3vcf: true,
                glimpse_reference_format: "bcf+sites".to_string(),
            },
        };
        let map = MapCatalogEntry {
            id: "map".to_string(),
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh37".to_string(),
            status: "production".to_string(),
            version: "1.0.0".to_string(),
            lock_ref: "locks/map_locks.toml#locks.map".to_string(),
            citation: None,
            files: Vec::new(),
            compatibility: MapCompatibility {
                tool_tags: vec!["minimac4".to_string()],
                coordinate_system: "bp".to_string(),
            },
        };

        let Err(error) = validate_imputation_tool_compatibility("minimac4", &panel, &map) else {
            panic!("panel/map build mismatch must fail");
        };

        assert!(error.to_string().contains("not compatible with map"));
    }
}
