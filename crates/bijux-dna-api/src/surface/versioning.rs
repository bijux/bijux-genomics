use serde::{Deserialize, Serialize};

/// Governed inventory of stable v1 route adapters and their bound response structs.
///
/// Stability: v1 (stable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRouteVersionInventoryV1 {
    pub schema_version: String,
    pub api_version: String,
    pub routes: Vec<bijux_dna_core::contract::ApiRouteAdapterV1>,
}

#[must_use]
pub fn route_version_inventory() -> ApiRouteVersionInventoryV1 {
    ApiRouteVersionInventoryV1 {
        schema_version: "bijux.api_route_inventory.v1".to_string(),
        api_version: "v1".to_string(),
        routes: bijux_dna_core::contract::governed_api_route_adapters(),
    }
}
