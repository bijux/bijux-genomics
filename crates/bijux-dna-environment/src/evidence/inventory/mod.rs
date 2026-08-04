//! Derivation of portable image-inventory publications from execution-site evidence.

mod derivation;
mod source;

pub use derivation::derive_image_inventory_publication;
pub use source::ImageInventoryDerivationRequest;
