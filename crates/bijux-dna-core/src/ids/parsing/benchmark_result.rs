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
    DomainKind::try_from(value).map_err(|_| {
        BijuxError::validation(format!(
            "benchmark result ids do not support legacy domain `{value}`"
        ))
    })
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
    use crate::ids::DomainKind;

    fn domain_stage_id(domain: DomainKind, stage_name: &str) -> String {
        format!("{}.{}", domain.as_str(), stage_name)
    }

    #[test]
    fn parses_sample_scoped_result_id() {
        let stage_id = domain_stage_id(DomainKind::Bam, "kinship");
        let result_id = build_sample_scoped_benchmark_result_id(
            DomainKind::Bam.as_str(),
            "corpus-01-kinship-mini",
            &stage_id,
            "sample-set",
            "king",
        );
        let parsed = parse_benchmark_result_id(&result_id).expect("parse result id");
        assert_eq!(parsed.domain.as_str(), DomainKind::Bam.as_str());
        assert_eq!(parsed.corpus_id, "corpus-01-kinship-mini");
        assert_eq!(parsed.stage_id.as_str(), stage_id);
        assert_eq!(parsed.scope_id, "sample-set");
        assert_eq!(parsed.tool_id.as_str(), "king");
        assert_eq!(parsed.scope_kind, BenchmarkResultScopeKind::SampleScope);
    }

    #[test]
    fn parses_asset_profile_result_id() {
        let stage_id = domain_stage_id(DomainKind::Vcf, "call");
        let result_id = build_asset_profile_benchmark_result_id(
            DomainKind::Vcf.as_str(),
            "vcf_production_regression",
            &stage_id,
            "bam_bundle",
            "bcftools",
        );
        let parsed = parse_benchmark_result_id(&result_id).expect("parse result id");
        assert_eq!(parsed.domain.as_str(), DomainKind::Vcf.as_str());
        assert_eq!(parsed.corpus_id, "vcf_production_regression");
        assert_eq!(parsed.stage_id.as_str(), stage_id);
        assert_eq!(parsed.scope_id, "bam_bundle");
        assert_eq!(parsed.tool_id.as_str(), "bcftools");
        assert_eq!(parsed.scope_kind, BenchmarkResultScopeKind::AssetProfile);
    }
}
