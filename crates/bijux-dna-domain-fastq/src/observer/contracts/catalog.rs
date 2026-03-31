use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObserverSpecializationContract {
    pub stage_id: &'static str,
    pub tool_id: &'static str,
    pub semantic_surface: &'static str,
}

pub(crate) const fn contract(
    stage_id: &'static str,
    tool_id: &'static str,
    semantic_surface: &'static str,
) -> ObserverSpecializationContract {
    ObserverSpecializationContract {
        stage_id,
        tool_id,
        semantic_surface,
    }
}

#[must_use]
pub fn observer_specialization_contracts() -> &'static [ObserverSpecializationContract] {
    specialization_contracts().as_slice()
}

fn specialization_contracts() -> &'static Vec<ObserverSpecializationContract> {
    static CONTRACTS: OnceLock<Vec<ObserverSpecializationContract>> = OnceLock::new();
    CONTRACTS.get_or_init(|| {
        let mut all = Vec::new();
        for group in [
            super::core::CONTRACTS,
            super::transform::CONTRACTS,
            super::amplicon::CONTRACTS,
        ] {
            all.extend_from_slice(group);
        }
        all
    })
}
