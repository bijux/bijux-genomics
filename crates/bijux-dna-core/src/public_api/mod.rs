pub mod contracts;

pub mod catalog;

pub mod identity;

/// Curated access to shared metrics types and registries.
pub mod metrics {
    pub use crate::metrics;
}

/// Curated access to stable import ergonomics.
pub mod ergonomics {
    pub use crate::prelude;
}
