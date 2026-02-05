// split to keep module size manageable

mod commands;
mod env;
mod main_helpers;
mod render;

include!("commands/entry.rs");
