pub mod commands;
pub mod cli {
    pub use crate::commands::cli::*;
}
pub mod env;
pub mod main_helpers;
pub mod render;
// CLI library intentionally thin; execution logic lives in bijux-api.
