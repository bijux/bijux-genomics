//! Owner: bijux-engine
//! Engine service modules.

pub mod runtime;

pub use runtime::EngineServices;

#[allow(dead_code)]
pub fn default_services() -> EngineServices {
    EngineServices
}
