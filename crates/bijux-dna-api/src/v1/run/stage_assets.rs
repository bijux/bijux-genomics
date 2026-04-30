//! Re-export governed stage asset requirements for v1 callers.
//!
//! Stability: v1 (stable).

pub use crate::runtime::run::{
    stage_external_asset_requirement, stage_requires_local_assets, StageAssetClass,
    StageExternalAssetRequirement,
};
