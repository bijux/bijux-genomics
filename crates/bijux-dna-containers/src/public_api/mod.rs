//! Stable public API facade for external crates.

mod stable_surface;

pub mod api {
    pub use super::stable_surface::*;
}
