use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_core::ids::ToolId;
use bijux_dna_planner_bam::stage_api::load_bam_domain_tool_contract_metadata;
use bijux_dna_planner_fastq::stage_api::load_fastq_domain_tool_contract_metadata;

use crate::commands::benchmark::local_stage_inventory::{
    load_local_stage_inventory, BenchLocalDomain,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ReadinessDomain {
    Fastq,
    Bam,
}

impl ReadinessDomain {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Fastq => "fastq",
            Self::Bam => "bam",
        }
    }

    fn tool_directory_relative_path(self) -> &'static str {
        match self {
            Self::Fastq => "domain/fastq/tools",
            Self::Bam => "domain/bam/tools",
        }
    }

    fn bench_local_domain(self) -> BenchLocalDomain {
        match self {
            Self::Fastq => BenchLocalDomain::Fastq,
            Self::Bam => BenchLocalDomain::Bam,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReadinessToolContract {
    pub(crate) domain: ReadinessDomain,
    pub(crate) tool_id: String,
    pub(crate) support_status: String,
    pub(crate) stage_ids: Vec<String>,
    pub(crate) planned_stage_ids: Vec<String>,
}

impl ReadinessToolContract {
    pub(crate) fn pair_support_status(&self, stage_id: &str) -> &'static str {
        if self.support_status == "planned"
            || self.planned_stage_ids.iter().any(|candidate| candidate == stage_id)
        {
            "planned"
        } else {
            "supported"
        }
    }

    pub(crate) fn admitted_stage_ids(&self) -> Vec<String> {
        let mut stage_ids = self
            .stage_ids
            .iter()
            .cloned()
            .chain(self.planned_stage_ids.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        stage_ids.sort();
        stage_ids
    }

    pub(crate) fn benchmark_stage_overlap(
        &self,
        benchmark_stage_ids: &BTreeSet<String>,
    ) -> Vec<String> {
        self.admitted_stage_ids()
            .into_iter()
            .filter(|stage_id| benchmark_stage_ids.contains(stage_id))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReadinessStageAdmission {
    pub(crate) domain: ReadinessDomain,
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) support_status: String,
}

pub(crate) fn load_benchmark_stage_ids(
    repo_root: &Path,
    domain: ReadinessDomain,
) -> Result<BTreeSet<String>> {
    let inventory = load_local_stage_inventory(repo_root, domain.bench_local_domain())?;
    Ok(inventory
        .stages
        .iter()
        .map(|entry| entry.stage_id.clone())
        .collect::<BTreeSet<_>>())
}

pub(crate) fn load_tool_contracts(
    repo_root: &Path,
    domain: ReadinessDomain,
) -> Result<Vec<ReadinessToolContract>> {
    let tools_dir = repo_root.join(domain.tool_directory_relative_path());
    let mut tool_ids = fs::read_dir(&tools_dir)
        .with_context(|| format!("read {}", tools_dir.display()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("iterate {}", tools_dir.display()))?
        .into_iter()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
                return None;
            }
            let stem = path.file_stem()?.to_str()?;
            if stem == "_schema" {
                return None;
            }
            Some(stem.to_string())
        })
        .collect::<Vec<_>>();
    tool_ids.sort();

    let mut contracts = Vec::with_capacity(tool_ids.len());
    for tool_id in tool_ids {
        let tool_id = ToolId::new(tool_id);
        let contract = match domain {
            ReadinessDomain::Fastq => {
                let metadata = load_fastq_domain_tool_contract_metadata(repo_root, &tool_id)?;
                ReadinessToolContract {
                    domain,
                    tool_id: metadata.tool_id.as_str().to_string(),
                    support_status: metadata.support_level.as_str().to_string(),
                    stage_ids: metadata
                        .stage_ids
                        .iter()
                        .map(|stage_id| stage_id.as_str().to_string())
                        .collect(),
                    planned_stage_ids: metadata
                        .planned_stage_ids
                        .iter()
                        .map(|stage_id| stage_id.as_str().to_string())
                        .collect(),
                }
            }
            ReadinessDomain::Bam => {
                let metadata = load_bam_domain_tool_contract_metadata(repo_root, &tool_id)?;
                ReadinessToolContract {
                    domain,
                    tool_id: metadata.tool_id.as_str().to_string(),
                    support_status: metadata.support_level.as_str().to_string(),
                    stage_ids: metadata
                        .stage_ids
                        .iter()
                        .map(|stage_id| stage_id.as_str().to_string())
                        .collect(),
                    planned_stage_ids: metadata
                        .planned_stage_ids
                        .iter()
                        .map(|stage_id| stage_id.as_str().to_string())
                        .collect(),
                }
            }
        };
        contracts.push(contract);
    }
    contracts.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    Ok(contracts)
}

pub(crate) fn load_stage_admissions(
    repo_root: &Path,
    domain: ReadinessDomain,
) -> Result<BTreeMap<String, Vec<ReadinessStageAdmission>>> {
    let benchmark_stage_ids = load_benchmark_stage_ids(repo_root, domain)?;
    let mut stage_admissions = BTreeMap::<String, Vec<ReadinessStageAdmission>>::new();
    for contract in load_tool_contracts(repo_root, domain)? {
        for stage_id in contract.benchmark_stage_overlap(&benchmark_stage_ids) {
            stage_admissions.entry(stage_id.clone()).or_default().push(ReadinessStageAdmission {
                domain,
                stage_id: stage_id.clone(),
                tool_id: contract.tool_id.clone(),
                support_status: contract.pair_support_status(&stage_id).to_string(),
            });
        }
    }
    for admissions in stage_admissions.values_mut() {
        admissions.sort_by(|left, right| left.tool_id.cmp(&right.tool_id));
    }
    Ok(stage_admissions)
}
