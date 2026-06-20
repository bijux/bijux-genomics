use serde::{Deserialize, Serialize};

use crate::foundation::{BijuxError, Result};
use crate::ids::{DomainKind, StageId, ToolId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkResultScopeKind {
    SampleScope,
    AssetProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BenchmarkResultIdentity {
    pub domain: DomainKind,
    pub corpus_id: String,
    pub stage_id: StageId,
    pub scope_id: String,
    pub tool_id: ToolId,
    pub scope_kind: BenchmarkResultScopeKind,
}

#[must_use]
pub fn build_sample_scoped_benchmark_result_id(
    domain: &str,
    corpus_id: &str,
    stage_id: &str,
    sample_scope: &str,
    tool_id: &str,
) -> String {
    build_benchmark_result_id(domain, corpus_id, stage_id, sample_scope, tool_id)
}

#[must_use]
pub fn build_asset_profile_benchmark_result_id(
    domain: &str,
    corpus_id: &str,
    stage_id: &str,
    asset_profile_id: &str,
    tool_id: &str,
) -> String {
    build_benchmark_result_id(domain, corpus_id, stage_id, asset_profile_id, tool_id)
}

pub fn parse_benchmark_result_id(result_id: &str) -> Result<BenchmarkResultIdentity> {
    let segments = result_id.split(':').collect::<Vec<_>>();
    if segments.len() != 5 {
        return Err(BijuxError::validation(format!(
            "benchmark result ids require five colon-delimited segments, found `{result_id}`"
        )));
    }

    let domain = parse_benchmark_result_domain(segments[0])?;
    Ok(BenchmarkResultIdentity {
        domain,
        corpus_id: segments[1].to_string(),
        stage_id: StageId::try_from(segments[2])?,
        scope_id: segments[3].to_string(),
        tool_id: ToolId::try_from(segments[4])?,
        scope_kind: benchmark_result_scope_kind(domain),
    })
}

fn build_benchmark_result_id(
    domain: &str,
    corpus_id: &str,
    stage_id: &str,
    scope_id: &str,
    tool_id: &str,
) -> String {
    format!("{domain}:{corpus_id}:{stage_id}:{scope_id}:{tool_id}")
}

fn parse_benchmark_result_domain(value: &str) -> Result<DomainKind> {
    match value {
        "fastq" => Ok(DomainKind::Fastq),
        "bam" => Ok(DomainKind::Bam),
        "vcf" => Ok(DomainKind::Vcf),
        other => Err(BijuxError::validation(format!(
            "benchmark result ids do not support legacy domain `{other}`"
        ))),
    }
}

fn benchmark_result_scope_kind(domain: DomainKind) -> BenchmarkResultScopeKind {
    match domain {
        DomainKind::Fastq | DomainKind::Bam => BenchmarkResultScopeKind::SampleScope,
        DomainKind::Vcf => BenchmarkResultScopeKind::AssetProfile,
        DomainKind::Cross => BenchmarkResultScopeKind::AssetProfile,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_asset_profile_benchmark_result_id, build_sample_scoped_benchmark_result_id,
        parse_benchmark_result_id, BenchmarkResultScopeKind,
    };

    #[test]
    fn parses_sample_scoped_result_id() {
        let result_id = build_sample_scoped_benchmark_result_id(
            "bam",
            "corpus-01-kinship-mini",
            "bam.kinship",
            "sample-set",
            "king",
        );
        let parsed = parse_benchmark_result_id(&result_id).expect("parse result id");
        assert_eq!(parsed.domain.as_str(), "bam");
        assert_eq!(parsed.corpus_id, "corpus-01-kinship-mini");
        assert_eq!(parsed.stage_id.as_str(), "bam.kinship");
        assert_eq!(parsed.scope_id, "sample-set");
        assert_eq!(parsed.tool_id.as_str(), "king");
        assert_eq!(parsed.scope_kind, BenchmarkResultScopeKind::SampleScope);
    }

    #[test]
    fn parses_asset_profile_result_id() {
        let result_id = build_asset_profile_benchmark_result_id(
            "vcf",
            "vcf_production_regression",
            "vcf.call",
            "bam_bundle",
            "bcftools",
        );
        let parsed = parse_benchmark_result_id(&result_id).expect("parse result id");
        assert_eq!(parsed.domain.as_str(), "vcf");
        assert_eq!(parsed.corpus_id, "vcf_production_regression");
        assert_eq!(parsed.stage_id.as_str(), "vcf.call");
        assert_eq!(parsed.scope_id, "bam_bundle");
        assert_eq!(parsed.tool_id.as_str(), "bcftools");
        assert_eq!(parsed.scope_kind, BenchmarkResultScopeKind::AssetProfile);
    }
}
