use super::model::{
    LocalVcfStatsSmokeMetrics, VcfStatsTruthCounts, LOCAL_VCF_STATS_SMOKE_METRICS_SCHEMA_VERSION,
};
use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::metrics::VcfStatsMetricsV1;
use std::{fs, path::Path};
pub(super) fn read_stats_json(path: &Path) -> Result<VcfStatsMetricsV1> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}
pub(super) fn validate_stats_against_truth(
    stats: &VcfStatsMetricsV1,
    truth: &VcfStatsTruthCounts,
    sample_name: &str,
) -> Result<()> {
    if stats.schema_version != "bijux.vcf.stats.v1" {
        bail!("VCF stats smoke requires normalized stats schema, found `{}`", stats.schema_version);
    }
    if stats.sample_name != sample_name {
        bail!(
            "VCF stats smoke sample name drifted: expected `{sample_name}`, found `{}`",
            stats.sample_name
        );
    }
    if stats.variants_total != truth.variants {
        bail!(
            "VCF stats smoke variant count drifted: expected {}, found {}",
            truth.variants,
            stats.variants_total
        );
    }
    if stats.snps != truth.snps {
        bail!("VCF stats smoke SNP count drifted: expected {}, found {}", truth.snps, stats.snps);
    }
    if stats.indels != truth.indels {
        bail!(
            "VCF stats smoke indel count drifted: expected {}, found {}",
            truth.indels,
            stats.indels
        );
    }
    if stats.sample_count != truth.samples {
        bail!(
            "VCF stats smoke sample count drifted: expected {}, found {}",
            truth.samples,
            stats.sample_count
        );
    }
    let expected_ti_tv = f64::from(u32::try_from(truth.transitions)?)
        / f64::from(u32::try_from(truth.transversions)?);
    let actual_ti_tv = stats
        .ti_tv
        .ok_or_else(|| anyhow!("VCF stats smoke requires ti_tv in normalized stats output"))?;
    if (actual_ti_tv - expected_ti_tv).abs() > 1e-9 {
        bail!("VCF stats smoke ti/tv drifted: expected {expected_ti_tv}, found {actual_ti_tv}");
    }
    Ok(())
}

pub(super) fn build_metrics(
    contract: &super::model::GovernedVcfStatsSmokeContract,
    truth: &super::model::VcfStatsTruthCounts,
    stats_json: &VcfStatsMetricsV1,
) -> Result<super::model::LocalVcfStatsSmokeMetrics> {
    let ti_tv = stats_json
        .ti_tv
        .ok_or_else(|| anyhow!("governed VCF stats smoke requires normalized ti_tv"))?;

    let metrics = LocalVcfStatsSmokeMetrics {
        schema_version: LOCAL_VCF_STATS_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        variant_count: truth.variants,
        snp_count: truth.snps,
        indel_count: truth.indels,
        transition_count: truth.transitions,
        transversion_count: truth.transversions,
        ti_tv,
        sample_count: truth.samples,
        tool_id: contract.tool_id.clone(),
        exit_code: 0,
    };

    Ok(metrics)
}
