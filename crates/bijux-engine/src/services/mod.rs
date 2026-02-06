//! Owner: bijux-engine
//! Engine service modules.

pub mod runtime_services;

pub use runtime_services::EngineServices;

#[allow(dead_code)]
pub fn default_services() -> EngineServices {
    EngineServices
}
