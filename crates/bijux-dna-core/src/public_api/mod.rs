pub mod contracts;

pub mod catalog;

/// Curated access to typed identities and their validators.
pub mod identity {
    pub use crate::ids;
}

/// Curated access to shared metrics types and registries.
pub mod metrics {
    pub use crate::metrics;
}

/// Curated access to stable import ergonomics.
pub mod ergonomics {
    pub use crate::prelude;
}
