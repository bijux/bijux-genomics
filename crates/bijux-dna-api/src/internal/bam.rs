//! BAM-specific internal stage wiring for the API crate.
//!
//! This module owns internal BAM stage composition and keeps the stage tree
//! grouped under a durable BAM namespace without leaving a placeholder
//! directory module behind.

pub(crate) mod stages;
