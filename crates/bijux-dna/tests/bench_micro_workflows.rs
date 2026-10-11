#![allow(clippy::expect_used, clippy::too_many_lines)]
//! Public micro-workflow contracts share one compilation target.
#[path = "contracts/benchmark/micro_workflows/adna/mod.rs"]
mod adna;
#[path = "contracts/benchmark/micro_workflows/amplicon/mod.rs"]
mod amplicon;
#[path = "contracts/benchmark/micro_workflows/core_germline/mod.rs"]
mod core_germline;
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;
