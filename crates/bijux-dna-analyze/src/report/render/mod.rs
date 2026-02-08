//! Owner: bijux-dna-analyze
//! Report renderers.
//! Renderers accept `ReportModel` only and perform no fact querying.

pub mod bundle;
pub mod html;
pub mod json;
pub mod markdown;

/// Stable render namespace used in reports and tests.
#[allow(dead_code)]
pub const RENDER_NAMESPACE: &str = "report.render.v1";
