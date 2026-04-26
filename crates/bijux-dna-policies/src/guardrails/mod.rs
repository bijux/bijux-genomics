mod baseline;
mod configuration;
mod presets;
mod runner;
mod source_inventory;
mod stable_surface;

pub use stable_surface::*;

#[allow(dead_code)]
const GUARDRAIL_MODULE_REGISTRY: &str = "policy_guardrail_runtime";
