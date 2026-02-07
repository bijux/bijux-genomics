// split to keep module size manageable

mod commands;
mod render {
    pub(crate) use crate::commands::cli::render::*;
}

include!("commands/entry.rs");
