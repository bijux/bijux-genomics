//! Shared stage-to-external-asset requirements for run preflight.
//!
//! Stability: v1 (stable).

/// Typed external asset classes required before selected stages may run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageAssetClass {
    AdapterBank,
    TaxonomyDatabase,
    HostReferenceBundle,
    RrnaReferenceBundle,
    ContaminantReferenceBundle,
    ReferencePreparationBundle,
    ReferencePanelBundle,
}

impl StageAssetClass {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::AdapterBank => "adapter bank",
            Self::TaxonomyDatabase => "taxonomy database",
            Self::HostReferenceBundle => "host reference bundle",
            Self::RrnaReferenceBundle => "rRNA reference bundle",
            Self::ContaminantReferenceBundle => "contaminant reference bundle",
            Self::ReferencePreparationBundle => "reference-preparation bundle",
            Self::ReferencePanelBundle => "reference-panel bundle",
        }
    }
}

/// Stable preflight requirement for stages that need governed local assets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageExternalAssetRequirement {
    pub stage_id: &'static str,
    pub asset_class: StageAssetClass,
    pub reason: &'static str,
}

const REQUIREMENTS: &[StageExternalAssetRequirement] = &[
    StageExternalAssetRequirement {
        stage_id: "fastq.detect_adapters",
        asset_class: StageAssetClass::AdapterBank,
        reason: "adapter detection must resolve governed adapter-bank inputs before execution",
    },
    StageExternalAssetRequirement {
        stage_id: "fastq.screen_taxonomy",
        asset_class: StageAssetClass::TaxonomyDatabase,
        reason: "taxonomy screening must resolve a governed taxonomy database before execution",
    },
    StageExternalAssetRequirement {
        stage_id: "fastq.deplete_host",
        asset_class: StageAssetClass::HostReferenceBundle,
        reason: "host depletion must resolve a governed host-reference bundle before execution",
    },
    StageExternalAssetRequirement {
        stage_id: "fastq.deplete_rrna",
        asset_class: StageAssetClass::RrnaReferenceBundle,
        reason: "rRNA depletion must resolve a governed rRNA reference bundle before execution",
    },
    StageExternalAssetRequirement {
        stage_id: "fastq.deplete_reference_contaminants",
        asset_class: StageAssetClass::ContaminantReferenceBundle,
        reason: "contaminant depletion must resolve governed contaminant references before execution",
    },
    StageExternalAssetRequirement {
        stage_id: "core.prepare_reference",
        asset_class: StageAssetClass::ReferencePreparationBundle,
        reason: "reference preparation must resolve the governed upstream reference bundle before execution",
    },
    StageExternalAssetRequirement {
        stage_id: "vcf.prepare_reference_panel",
        asset_class: StageAssetClass::ReferencePanelBundle,
        reason: "panel preparation must resolve the governed reference panel before execution",
    },
];

#[must_use]
pub fn stage_external_asset_requirement(stage_id: &str) -> Option<StageExternalAssetRequirement> {
    REQUIREMENTS.iter().copied().find(|requirement| requirement.stage_id == stage_id)
}

#[must_use]
pub fn stage_requires_local_assets(stage_id: &str) -> bool {
    stage_external_asset_requirement(stage_id).is_some()
}

#[cfg(test)]
mod tests {
    use super::{stage_external_asset_requirement, stage_requires_local_assets, StageAssetClass};

    #[test]
    fn stage_asset_requirements_are_exact_not_fuzzy() {
        let requirement = stage_external_asset_requirement("fastq.detect_adapters")
            .expect("detect adapters should require governed local assets");
        assert_eq!(requirement.asset_class, StageAssetClass::AdapterBank);

        assert!(stage_requires_local_assets("fastq.detect_adapters"));
        assert!(!stage_requires_local_assets("fastq.detect_adapters.legacy"));
        assert!(!stage_requires_local_assets("fastq.screening_summary"));
        assert!(!stage_requires_local_assets("bam.coverage"));
    }

    #[test]
    fn stage_asset_requirements_cover_governed_reference_and_screening_stages() {
        assert_eq!(
            stage_external_asset_requirement("fastq.screen_taxonomy")
                .expect("screen taxonomy should require governed local assets")
                .asset_class,
            StageAssetClass::TaxonomyDatabase
        );
        assert_eq!(
            stage_external_asset_requirement("vcf.prepare_reference_panel")
                .expect("prepare reference panel should require governed local assets")
                .asset_class,
            StageAssetClass::ReferencePanelBundle
        );
        assert_eq!(
            stage_external_asset_requirement("core.prepare_reference")
                .expect("prepare reference should require governed local assets")
                .asset_class,
            StageAssetClass::ReferencePreparationBundle
        );
    }
}
