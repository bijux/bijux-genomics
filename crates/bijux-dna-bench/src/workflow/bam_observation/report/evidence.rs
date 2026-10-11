//! Owner: bijux-dna-bench
//! Shared evidence projection for admitted BAM observational attempts.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkCampaignPlan, BenchmarkReportDeterminism, BenchmarkReportVerdict,
    BenchmarkStageCampaignSpec, BenchmarkStageInterfaceSpec, BenchmarkToolReport,
};

use super::science::performance_distributions;
use super::BamObservationReportPolicy;
use crate::workflow::stage_report::{
    load_admitted_attempt_evidence, read_resource_observation, ComparableAttemptEvidence,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn load_attempt_evidence(
    repository_root: &Path,
    study_root: &Path,
    benchmark_root: &Path,
    campaign: &BenchmarkStageCampaignSpec,
    interface: &BenchmarkStageInterfaceSpec,
    plan: &BenchmarkCampaignPlan,
    identity: &bijux_dna_bench_model::BenchmarkAttemptIdentity,
    policy: &BamObservationReportPolicy,
) -> Result<ComparableAttemptEvidence> {
    let admitted = load_admitted_attempt_evidence(
        repository_root,
        study_root,
        benchmark_root,
        campaign,
        plan,
        identity,
    )?;
    let spec = &admitted.spec;
    let resources = (1..=campaign.measured_repetitions)
        .map(|repetition| {
            read_resource_observation(
                &Path::new(&spec.outputs.result_root)
                    .join(format!("repetitions/repetition-{repetition}.resource.tsv")),
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let science = (1..=campaign.measured_repetitions)
        .map(|repetition| (policy.derive)(spec, repetition))
        .collect::<Result<Vec<_>>>()?;
    let input_identity = science
        .iter()
        .map(|row| (row.bam_input_bytes, row.bam_index_input_bytes, row.input_bytes))
        .collect::<BTreeSet<_>>();
    if input_identity.len() != 1 {
        bail!("BAM observational repetitions disagree on input bytes");
    }
    let bundle_digests =
        science.iter().map(|row| row.normalized_bundle_sha256.as_str()).collect::<BTreeSet<_>>();
    if bundle_digests.len() != 1 {
        bail!("normalized BAM observational bundles differ across repetitions");
    }
    let scientific_observations = campaign
        .scientific_metrics
        .iter()
        .map(|metric| {
            let values = science
                .iter()
                .map(|row| {
                    row.values
                        .get(metric)
                        .cloned()
                        .with_context(|| format!("scientific observation omits {metric}"))
                })
                .collect::<Result<Vec<_>>>()?;
            Ok((metric.clone(), values))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let performance = performance_distributions(&resources, &science)?;
    let expected = campaign.performance_metrics.iter().cloned().collect::<BTreeSet<_>>();
    let observed = performance.keys().cloned().collect::<BTreeSet<_>>();
    if observed != expected {
        bail!("derived BAM observational performance closure differs from campaign");
    }
    if admitted.interface.accepted_input_contracts != interface.accepted_input_contracts
        || admitted.interface.output_contract != interface.output_contract
        || admitted.interface.normalized_result_contract != interface.normalized_result_contract
    {
        bail!("attempt stage interface differs from repository authority");
    }
    let first = science.first().context("BAM observational evidence has no repetitions")?;
    let tool_report = BenchmarkToolReport {
        tool: identity.tool_id.clone(),
        verdict: BenchmarkReportVerdict::Passed,
        repository_commit: spec.source.repository_commit.clone(),
        image: admitted.image,
        slurm: admitted.slurm,
        source_evidence_sha256: admitted.source_evidence_sha256,
        input_locks: admitted.input_locks,
        input_bytes: BTreeMap::from([
            ("bam".to_string(), first.bam_input_bytes),
            ("bam_index".to_string(), first.bam_index_input_bytes),
            ("total".to_string(), first.input_bytes),
        ]),
        performance,
        scientific_observations,
        scientific_strata: Vec::new(),
        determinism: BenchmarkReportDeterminism {
            artifact: campaign.result_artifact.clone(),
            sha256: first.normalized_bundle_sha256.clone(),
            digest_basis: Some(policy.digest_basis.to_string()),
            identical_across_repetitions: true,
        },
    };
    Ok(ComparableAttemptEvidence {
        tool_report,
        input_lock_sha256: admitted.input_lock_sha256,
        parameter_lock_sha256: admitted.parameter_lock_sha256,
        interface: admitted.interface,
    })
}
