#![forbid(unsafe_code)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![deny(clippy::todo)]
#![allow(
    clippy::assigning_clones,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,
    clippy::default_trait_access,
    clippy::format_collect,
    clippy::format_push_string,
    clippy::items_after_statements,
    clippy::manual_let_else,
    clippy::map_unwrap_or,
    clippy::match_same_arms,
    clippy::needless_pass_by_value,
    clippy::too_many_arguments,
    clippy::too_many_lines,
    clippy::unnecessary_wraps
)]

mod application;
mod catalog;
mod cli;
mod commands;
mod dev_entrypoint;
mod model;
mod runtime;

fn main() -> anyhow::Result<()> {
    dev_entrypoint::run()
}
