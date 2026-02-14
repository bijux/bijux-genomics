use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;

use bijux_dna_core::contract::ExecutionStep;

#[derive(Debug, Clone)]
pub struct Invocation {
    pub step: ExecutionStep,
    pub attempt: u32,
}

#[derive(Debug, Clone)]
pub struct Artifact {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Debug, Clone)]
pub struct RunnerResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub artifacts: Vec<Artifact>,
}

pub trait Runner {
    /// # Errors
    /// Returns an error if the runner cannot execute the invocation or capture results.
    fn run(&self, invocation: &Invocation) -> Result<RunnerResult>;
}

mod stage_runner_contract {
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
        pub stage_domain: &'static str,
    }

    impl DomainStageRunnerContract for PrefixDomainStageRunnerContract {
        fn domain(&self) -> &'static str {
            self.domain_name
        }

        fn supports_stage(&self, stage_id: &str) -> bool {
            stage_id
                .split_once('.')
                .map(|(domain, _)| domain == self.stage_domain)
                .unwrap_or(false)
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
            stage_domain: "fastq",
        },
        PrefixDomainStageRunnerContract {
            domain_name: "bam",
            stage_domain: "bam",
        },
        PrefixDomainStageRunnerContract {
            domain_name: "vcf",
            stage_domain: "vcf",
        },
        PrefixDomainStageRunnerContract {
            domain_name: "core",
            stage_domain: "core",
        },
        PrefixDomainStageRunnerContract {
            domain_name: "cross",
            stage_domain: "cross",
        },
        PrefixDomainStageRunnerContract {
            domain_name: "report",
            stage_domain: "report",
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
}

pub use stage_runner_contract::{
    ensure_stage_supported_by_runner, DomainStageRunnerContract, PrefixDomainStageRunnerContract,
    RunnerContractKind,
};
