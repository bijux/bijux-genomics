#![forbid(unsafe_code)]
#![deny(clippy::dbg_macro, clippy::print_stdout, clippy::print_stderr)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]
#![deny(clippy::todo)]

mod application;
mod app;
mod catalog;
mod cli;
mod commands;
mod model;
mod runtime;

fn main() -> anyhow::Result<()> {
    app::run()
}
