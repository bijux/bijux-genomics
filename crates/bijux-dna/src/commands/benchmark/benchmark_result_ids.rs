use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BenchmarkResultScopeKind {
    SampleScope,
    AssetProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ParsedBenchmarkResultId {
    pub(crate) domain: String,
    pub(crate) corpus_id: String,
    pub(crate) stage_id: String,
    pub(crate) scope_id: String,
    pub(crate) tool_id: String,
    pub(crate) scope_kind: BenchmarkResultScopeKind,
}

pub(crate) fn build_sample_scoped_benchmark_result_id(
    domain: &str,
    corpus_id: &str,
    stage_id: &str,
    sample_scope: &str,
    tool_id: &str,
) -> String {
    build_benchmark_result_id(domain, corpus_id, stage_id, sample_scope, tool_id)
}

pub(crate) fn build_asset_profile_benchmark_result_id(
    domain: &str,
    corpus_id: &str,
    stage_id: &str,
    asset_profile_id: &str,
    tool_id: &str,
) -> String {
    build_benchmark_result_id(domain, corpus_id, stage_id, asset_profile_id, tool_id)
}

pub(crate) fn parse_benchmark_result_id(result_id: &str) -> Result<ParsedBenchmarkResultId> {
    let segments = result_id.split(':').collect::<Vec<_>>();
    if segments.len() != 5 {
        return Err(anyhow!(
            "benchmark result ids require five colon-delimited segments, found `{result_id}`"
        ));
    }
    Ok(ParsedBenchmarkResultId {
        domain: segments[0].to_string(),
        corpus_id: segments[1].to_string(),
        stage_id: segments[2].to_string(),
        scope_id: segments[3].to_string(),
        tool_id: segments[4].to_string(),
        scope_kind: benchmark_result_scope_kind(segments[0])?,
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

fn benchmark_result_scope_kind(domain: &str) -> Result<BenchmarkResultScopeKind> {
    match domain {
        "fastq" | "bam" => Ok(BenchmarkResultScopeKind::SampleScope),
        "vcf" => Ok(BenchmarkResultScopeKind::AssetProfile),
        other => Err(anyhow!("benchmark result ids do not support legacy domain `{other}`")),
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
        assert_eq!(parsed.domain, "bam");
        assert_eq!(parsed.corpus_id, "corpus-01-kinship-mini");
        assert_eq!(parsed.stage_id, "bam.kinship");
        assert_eq!(parsed.scope_id, "sample-set");
        assert_eq!(parsed.tool_id, "king");
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
        assert_eq!(parsed.domain, "vcf");
        assert_eq!(parsed.corpus_id, "vcf_production_regression");
        assert_eq!(parsed.stage_id, "vcf.call");
        assert_eq!(parsed.scope_id, "bam_bundle");
        assert_eq!(parsed.tool_id, "bcftools");
        assert_eq!(parsed.scope_kind, BenchmarkResultScopeKind::AssetProfile);
    }
}
