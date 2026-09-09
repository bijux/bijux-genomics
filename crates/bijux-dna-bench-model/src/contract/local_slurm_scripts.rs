//! Owner: bijux-dna-bench-model
//! Pure report construction and admission for local SLURM script inventories.

use crate::{
    BenchmarkLocalAllDomainSlurmScriptEntry, BenchmarkLocalAllDomainSlurmScriptsReport,
    BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SCRIPTS_SCHEMA_V1,
};
use anyhow::{anyhow, Result};
use std::collections::{BTreeMap, BTreeSet};

/// Summarize the exact script entries produced by an adapter, then admit coverage.
///
/// # Errors
/// Rejects unsupported job kinds, duplicate identities and incomplete coverage.
pub fn build_benchmark_local_all_domain_slurm_scripts_report(
    output_root: String,
    scripts: Vec<BenchmarkLocalAllDomainSlurmScriptEntry>,
    expected_benchmark_job_count: usize,
    expected_essential_pipeline_job_count: usize,
) -> Result<BenchmarkLocalAllDomainSlurmScriptsReport> {
    let (domain_counts, benchmark_domain_counts, essential_pipeline_domain_counts, pipeline_count) =
        summarize(&scripts)?;
    let report = BenchmarkLocalAllDomainSlurmScriptsReport {
        schema_version: BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SCRIPTS_SCHEMA_V1.to_string(),
        output_root,
        script_count: scripts.len(),
        benchmark_job_count: benchmark_domain_counts.values().sum(),
        essential_pipeline_job_count: essential_pipeline_domain_counts.values().sum(),
        pipeline_count,
        domain_counts,
        benchmark_domain_counts,
        essential_pipeline_domain_counts,
        scripts,
    };
    validate_benchmark_local_all_domain_slurm_scripts_report(
        &report,
        expected_benchmark_job_count,
        expected_essential_pipeline_job_count,
    )?;
    Ok(report)
}

type DomainCounts = BTreeMap<String, usize>;
fn summarize(
    scripts: &[BenchmarkLocalAllDomainSlurmScriptEntry],
) -> Result<(DomainCounts, DomainCounts, DomainCounts, usize)> {
    let mut domains = BTreeMap::new();
    let mut benchmarks = BTreeMap::new();
    let mut pipelines = BTreeMap::new();
    let mut pipeline_ids = BTreeSet::new();
    for entry in scripts {
        *domains.entry(entry.domain.clone()).or_default() += 1;
        match entry.job_kind.as_str() {
            "benchmark_result"
                if entry.result_id.is_some()
                    && entry.pipeline_id.is_none()
                    && entry.node_id.is_none() =>
            {
                *benchmarks.entry(entry.domain.clone()).or_default() += 1;
            }
            "essential_pipeline_node" if entry.result_id.is_none() && entry.node_id.is_some() => {
                let pipeline =
                    entry.pipeline_id.as_deref().filter(|id| !id.trim().is_empty()).ok_or_else(
                        || anyhow!("all-domain slurm script entry requires a pipeline identity"),
                    )?;
                pipeline_ids.insert(pipeline);
                *pipelines.entry(entry.domain.clone()).or_default() += 1;
            }
            _ => {
                return Err(anyhow!(
                    "all-domain slurm script entry has an invalid job kind or identity"
                ))
            }
        }
    }
    Ok((domains, benchmarks, pipelines, pipeline_ids.len()))
}

/// Admit a serialized script report against independently expected job counts.
///
/// # Errors
/// Rejects unsupported versions, inconsistent aggregates and incomplete job coverage.
pub fn validate_benchmark_local_all_domain_slurm_scripts_report(
    report: &BenchmarkLocalAllDomainSlurmScriptsReport,
    expected_benchmark_job_count: usize,
    expected_essential_pipeline_job_count: usize,
) -> Result<()> {
    if report.schema_version != BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SCRIPTS_SCHEMA_V1
        || report.output_root.trim().is_empty()
    {
        return Err(anyhow!(
            "all-domain slurm script report requires a supported schema and output root"
        ));
    }
    let (domains, benchmarks, pipelines, pipeline_count) = summarize(&report.scripts)?;
    if report.script_count != report.scripts.len()
        || report.domain_counts != domains
        || report.benchmark_domain_counts != benchmarks
        || report.essential_pipeline_domain_counts != pipelines
        || report.pipeline_count != pipeline_count
        || report.benchmark_job_count != benchmarks.values().sum::<usize>()
        || report.essential_pipeline_job_count != pipelines.values().sum::<usize>()
    {
        return Err(anyhow!(
            "all-domain slurm script report aggregates do not close over its entries"
        ));
    }
    if report.benchmark_job_count != expected_benchmark_job_count {
        return Err(anyhow!(
            "all-domain slurm script generation must cover every expected benchmark result job (expected {}, found {})",
            expected_benchmark_job_count,
            report.benchmark_job_count
        ));
    }
    if report.essential_pipeline_job_count != expected_essential_pipeline_job_count {
        return Err(anyhow!(
            "all-domain slurm script generation must cover every essential pipeline node (expected {}, found {})",
            expected_essential_pipeline_job_count,
            report.essential_pipeline_job_count
        ));
    }
    if report.script_count != report.benchmark_job_count + report.essential_pipeline_job_count {
        return Err(anyhow!(
            "all-domain slurm script generation must keep script_count equal to benchmark jobs plus essential pipeline jobs"
        ));
    }
    if report.benchmark_domain_counts.get("vcf").copied().unwrap_or_default() == 0 {
        return Err(anyhow!(
            "all-domain slurm script generation must include benchmark-ready VCF jobs"
        ));
    }
    if report.essential_pipeline_job_count == 0 || report.pipeline_count == 0 {
        return Err(anyhow!(
            "all-domain slurm script generation must include essential pipeline jobs"
        ));
    }
    let unique_paths =
        report.scripts.iter().map(|entry| entry.script_path.as_str()).collect::<BTreeSet<_>>();
    if unique_paths.len() != report.scripts.len() {
        return Err(anyhow!(
            "all-domain slurm script generation must keep one unique script path per job"
        ));
    }
    let unique_job_ids =
        report.scripts.iter().map(|entry| entry.job_id_local.as_str()).collect::<BTreeSet<_>>();
    if unique_job_ids.len() != report.scripts.len() {
        return Err(anyhow!(
            "all-domain slurm script generation must keep one unique local job id per script"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entries() -> Result<Vec<BenchmarkLocalAllDomainSlurmScriptEntry>> {
        Ok(serde_json::from_value(json!([
            {"job_id_local":"benchmark:vcf.stats", "job_kind":"benchmark_result", "result_id":"vcf.stats", "pipeline_id":null,"node_id":null,
             "domain":"vcf","stage_id":"vcf.stats","tool_id":"bcftools","corpus_id":"sample","asset_profile_id":"cohort","dependency_count":0,
             "cpus_per_task":1,"memory_mb":128,"time_limit":"00:01:00","script_path":"benchmark/job.sbatch","stdout_path":"benchmark/stdout.log","stderr_path":"benchmark/stderr.log","command_step_count":1},
            {"job_id_local":"pipeline:germline:vcf.stats", "job_kind":"essential_pipeline_node", "result_id":null, "pipeline_id":"germline","node_id":"vcf.stats",
             "domain":"vcf","stage_id":"vcf.stats","tool_id":"bcftools","corpus_id":"sample","asset_profile_id":"cohort","dependency_count":0,
             "cpus_per_task":1,"memory_mb":128,"time_limit":"00:01:00","script_path":"pipeline/job.sbatch","stdout_path":"pipeline/stdout.log","stderr_path":"pipeline/stderr.log","command_step_count":1}
        ]))?)
    }

    #[test]
    fn script_report_roundtrip_preserves_wire_entries_and_coverage() -> Result<()> {
        let rows = entries()?;
        let report = build_benchmark_local_all_domain_slurm_scripts_report(
            "scripts".to_string(),
            rows.clone(),
            1,
            1,
        )?;
        assert_eq!(report.script_count, 2);
        assert_eq!(report.pipeline_count, 1);
        assert_eq!(report.domain_counts, BTreeMap::from([("vcf".to_string(), 2)]));
        assert_eq!(report.scripts, rows);
        let wire = serde_json::to_value(&report)?;
        assert_eq!(wire["schema_version"], "bijux.bench.local_all_domain_slurm_scripts.v1");
        let decoded = serde_json::from_value(wire)?;
        assert_eq!(report, decoded);
        validate_benchmark_local_all_domain_slurm_scripts_report(&decoded, 1, 1)?;
        assert!(validate_benchmark_local_all_domain_slurm_scripts_report(&decoded, 2, 1).is_err());
        assert!(validate_benchmark_local_all_domain_slurm_scripts_report(&decoded, 1, 2).is_err());
        Ok(())
    }

    #[test]
    fn script_report_rejects_forged_aggregates_identities_and_unsupported_shapes() -> Result<()> {
        let report = build_benchmark_local_all_domain_slurm_scripts_report(
            "scripts".to_string(),
            entries()?,
            1,
            1,
        )?;
        let original = serde_json::to_value(&report)?;
        for (pointer, value) in [
            ("/schema_version", json!("bijux.bench.local_all_domain_slurm_scripts.v9")),
            ("/output_root", json!(" ")),
            ("/script_count", json!(3)),
            ("/benchmark_job_count", json!(2)),
            ("/essential_pipeline_job_count", json!(0)),
            ("/pipeline_count", json!(2)),
            ("/domain_counts/vcf", json!(3)),
            ("/benchmark_domain_counts/vcf", json!(0)),
            ("/essential_pipeline_domain_counts/vcf", json!(0)),
            ("/scripts/1/script_path", json!("benchmark/job.sbatch")),
            ("/scripts/1/job_id_local", json!("benchmark:vcf.stats")),
            ("/scripts/0/job_kind", json!("unknown")),
            ("/scripts/0/pipeline_id", json!("germline")),
            ("/scripts/1/pipeline_id", json!(null)),
        ] {
            let mut wire = original.clone();
            let target = wire
                .pointer_mut(pointer)
                .ok_or_else(|| anyhow!("missing fixture pointer {pointer}"))?;
            *target = value;
            let mutated = serde_json::from_value(wire)?;
            assert!(
                validate_benchmark_local_all_domain_slurm_scripts_report(&mutated, 1, 1).is_err(),
                "accepted {pointer}"
            );
        }
        let mut wire = original;
        wire["scripts"][0]["unsupported"] = json!(true);
        assert!(serde_json::from_value::<BenchmarkLocalAllDomainSlurmScriptsReport>(wire).is_err());
        let mut rows = entries()?;
        rows[0].domain = "bam".to_string();
        assert!(build_benchmark_local_all_domain_slurm_scripts_report(
            "scripts".to_string(),
            rows,
            1,
            1
        )
        .is_err());
        Ok(())
    }
}
