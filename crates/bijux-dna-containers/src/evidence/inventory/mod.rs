//! Derivation of portable image-inventory publications from execution-site evidence.

mod activation;
mod derivation;
mod report;
mod source;

pub use activation::image_inventory_request_from_activation_plan;
pub use derivation::{
    derive_image_inventory_evidence, derive_image_inventory_publication, DerivedImageInventory,
};
pub use report::render_image_inventory_markdown;
pub use source::{
    ImageInventoryDerivationRequest, ImageInventoryEvidenceRootIdentity,
    ImageInventoryEvidenceRoots, ImageInventoryScan, ImageInventoryScannedImage,
    ImageInventorySourceFile, RAW_INVENTORY_SCHEMA_V1, RAW_INVENTORY_SCHEMA_V2,
};
