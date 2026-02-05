// split to keep module size manageable

mod commands;
mod env;
mod render;

include!("commands/entry.rs");
