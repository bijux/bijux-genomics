pub(crate) use bijux_dna_core::ids::{
    build_asset_profile_benchmark_result_id, build_sample_scoped_benchmark_result_id,
    parse_benchmark_result_id, BenchmarkResultScopeKind,
};

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
