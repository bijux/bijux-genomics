use anyhow::{anyhow, Result};

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
    pub stage_prefix: &'static str,
}

impl DomainStageRunnerContract for PrefixDomainStageRunnerContract {
    fn domain(&self) -> &'static str {
        self.domain_name
    }

    fn supports_stage(&self, stage_id: &str) -> bool {
        stage_id.starts_with(self.stage_prefix)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerContractKind {
    Docker,
}

impl std::fmt::Display for RunnerContractKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Docker => f.write_str("docker"),
        }
    }
}

const DOCKER_DOMAIN_CONTRACTS: &[PrefixDomainStageRunnerContract] = &[
    PrefixDomainStageRunnerContract {
        domain_name: "fastq",
        stage_prefix: "fastq.",
    },
    PrefixDomainStageRunnerContract {
        domain_name: "bam",
        stage_prefix: "bam.",
    },
    PrefixDomainStageRunnerContract {
        domain_name: "vcf",
        stage_prefix: "vcf.",
    },
    PrefixDomainStageRunnerContract {
        domain_name: "core",
        stage_prefix: "core.",
    },
    PrefixDomainStageRunnerContract {
        domain_name: "cross",
        stage_prefix: "cross.",
    },
    PrefixDomainStageRunnerContract {
        domain_name: "report",
        stage_prefix: "report.",
    },
];

/// # Errors
/// Returns an error when no runner contract can execute the stage.
pub fn ensure_stage_supported_by_runner(
    runner: RunnerContractKind,
    stage_id: &str,
) -> Result<()> {
    let contracts: &[PrefixDomainStageRunnerContract] = match runner {
        RunnerContractKind::Docker => DOCKER_DOMAIN_CONTRACTS,
    };
    if contracts.iter().any(|contract| contract.supports_stage(stage_id)) {
        return Ok(());
    }
    Err(anyhow!(
        "runner {} has no stage-runner contract for stage {}",
        runner,
        stage_id
    ))
}

