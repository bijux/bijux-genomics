//! Derivation of portable image-inventory publications from execution-site evidence.

mod derivation;
mod source;

pub use derivation::{
    derive_image_inventory_evidence, derive_image_inventory_publication, DerivedImageInventory,
};
pub use source::{
    ImageInventoryDerivationRequest, ImageInventoryScan, ImageInventoryScannedImage,
    ImageInventorySourceFile, RAW_INVENTORY_SCHEMA_V1,
};
