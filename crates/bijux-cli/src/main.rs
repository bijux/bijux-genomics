// split to keep module size manageable

mod commands;
mod render {
    pub use crate::commands::cli::render::*;
}

include!("commands/entry.rs");
