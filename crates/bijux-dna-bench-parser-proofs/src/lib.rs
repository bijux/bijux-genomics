//! Governed parser proof contracts and aggregation over explicit materialization ports.
mod admission;
pub mod catalog;
mod materialization;
pub mod model;
pub mod ports;
mod publication;
pub mod reports;
mod service;
mod values;
pub use publication::publish_report;
pub use service::{build_report, materialize_row};
