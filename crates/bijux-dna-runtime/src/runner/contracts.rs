use anyhow::{anyhow, Result};

use super::contract_kinds::RunnerContractKind;

/// Domain-level stage execution contract.
///
/// Implementations declare whether a runner can execute a stage id for a domain.
pub trait DomainStageRunnerContract {
    fn domain(&self) -> &'static str;
    fn supports_stage(&self, stage_id: &str) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct PrefixDomainStageRunnerContract {
    pub domain_name: &'static str,
    pub stage_domain: &'static str,
}

impl DomainStageRunnerContract for PrefixDomainStageRunnerContract {
    fn domain(&self) -> &'static str {
        self.domain_name
    }

    fn supports_stage(&self, stage_id: &str) -> bool {
        stage_id.split_once('.').is_some_and(|(domain, _)| domain == self.stage_domain)
    }
}

const CONTAINER_RUNNER_DOMAIN_CONTRACTS: &[PrefixDomainStageRunnerContract] = &[
    PrefixDomainStageRunnerContract { domain_name: "fastq", stage_domain: "fastq" },
    PrefixDomainStageRunnerContract { domain_name: "bam", stage_domain: "bam" },
    PrefixDomainStageRunnerContract { domain_name: "vcf", stage_domain: "vcf" },
    PrefixDomainStageRunnerContract { domain_name: "core", stage_domain: "core" },
    PrefixDomainStageRunnerContract { domain_name: "cross", stage_domain: "cross" },
    PrefixDomainStageRunnerContract { domain_name: "report", stage_domain: "report" },
];

/// # Errors
/// Returns an error when no runner contract can execute the stage.
pub fn ensure_stage_supported_by_runner(runner: RunnerContractKind, stage_id: &str) -> Result<()> {
    let contracts: &[PrefixDomainStageRunnerContract] = match runner {
        RunnerContractKind::Local | RunnerContractKind::Docker | RunnerContractKind::Apptainer => {
            CONTAINER_RUNNER_DOMAIN_CONTRACTS
        }
    };
    if contracts.iter().any(|contract| contract.supports_stage(stage_id)) {
        return Ok(());
    }
    Err(anyhow!("runner {runner} has no stage-runner contract for stage {stage_id}",))
}
