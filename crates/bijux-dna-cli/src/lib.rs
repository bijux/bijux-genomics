pub mod commands;
pub mod cli {
    pub use crate::commands::cli::*;
}
// CLI library intentionally thin; execution logic lives in bijux-dna-api.
