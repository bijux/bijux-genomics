use crate::model::check::{CheckOutcome, CheckStatus};
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct CargoTestKey {
    pub package: &'static str,
    pub test_bin: &'static str,
    pub filter: &'static str,
}

#[derive(Debug, Clone)]
pub(super) enum CargoTestObservation {
    Completed { status: CheckStatus, detail: String },
    NoTests { filter: String },
    LaunchFailure { detail: String },
}

impl CargoTestObservation {
    pub(super) fn for_check(&self, id: &str) -> CheckOutcome {
        match self {
            Self::Completed { status, detail } => CheckOutcome::leaf(id, *status, detail.clone()),
            Self::NoTests { filter } => CheckOutcome::leaf(
                id,
                CheckStatus::Failed,
                format!("check `{id}` matched no tests for filter `{filter}`"),
            ),
            Self::LaunchFailure { detail } => {
                CheckOutcome::leaf(id, CheckStatus::Failed, detail.clone())
            }
        }
    }
}

/// Retain only exact audited read-only policy observations until an effect boundary.
#[derive(Default)]
pub(super) struct RequestPolicyObservations {
    observations: std::collections::BTreeMap<CargoTestKey, CargoTestObservation>,
}

impl RequestPolicyObservations {
    pub(super) fn clear(&mut self) {
        self.observations.clear();
    }
    pub(super) fn observe(
        &mut self,
        key: CargoTestKey,
        execute: impl FnOnce() -> Result<CargoTestObservation>,
    ) -> CargoTestObservation {
        if !read_only_policy(key) {
            self.clear();
            let observed = execute().unwrap_or_else(|error| CargoTestObservation::LaunchFailure {
                detail: error.to_string(),
            });
            self.clear();
            return observed;
        }
        self.observations
            .entry(key)
            .or_insert_with(|| {
                execute().unwrap_or_else(|error| CargoTestObservation::LaunchFailure {
                    detail: error.to_string(),
                })
            })
            .clone()
    }
}

fn read_only_policy(key: CargoTestKey) -> bool {
    key.package == "bijux-dna-policies"
        && key.test_bin == "contracts"
        && matches!(
            key.filter,
            "assets_governance_policy"
                | "generated_configs_policy"
                | "contract_authority_policy"
                | "tool_registry_reproducibility_policy"
                | "vcf_support_gate_policy"
                | "examples_golden_hygiene_policy"
                | "examples_cli_command_policy"
                | "stage_executor_parity_policy"
                | "opentelemetry_version_policy"
                | "error_boundary_policy"
        )
}
