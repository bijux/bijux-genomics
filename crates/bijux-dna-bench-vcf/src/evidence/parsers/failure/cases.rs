use bijux_dna_domain_vcf::VcfDomainStage;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VcfParserFailureReason {
    EmptyOutput,
    MalformedVcf,
    MissingIndex,
    MissingSampleColumn,
    MalformedPcaTable,
    MalformedImputationQualityFile,
    MalformedSegmentFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VcfParserFailureCase {
    pub(super) case_id: &'static str,
    pub(super) stage: VcfDomainStage,
    pub(super) tool_id: &'static str,
    pub(super) parser_id: &'static str,
    pub(super) fixture_dir: &'static str,
    pub(super) failure_reason: VcfParserFailureReason,
    pub(super) probe_artifact: &'static str,
    pub(super) expected_error_fragment: &'static str,
}

pub(super) const VCF_PARSER_FAILURE_CASES: &[VcfParserFailureCase] = &[
    VcfParserFailureCase {
        case_id: "empty-output",
        stage: VcfDomainStage::Call,
        tool_id: "bcftools",
        parser_id: "parse_bcftools_call_metrics",
        fixture_dir: "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call",
        failure_reason: VcfParserFailureReason::EmptyOutput,
        probe_artifact: "raw.calls.vcf",
        expected_error_fragment: "raw VCF is missing #CHROM header",
    },
    VcfParserFailureCase {
        case_id: "malformed-vcf",
        stage: VcfDomainStage::Call,
        tool_id: "bcftools",
        parser_id: "parse_bcftools_call_metrics",
        fixture_dir: "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.call",
        failure_reason: VcfParserFailureReason::MalformedVcf,
        probe_artifact: "raw.calls.vcf",
        expected_error_fragment: "malformed raw VCF record at line",
    },
    VcfParserFailureCase {
        case_id: "missing-index",
        stage: VcfDomainStage::Postprocess,
        tool_id: "bcftools",
        parser_id: "parse_bcftools_postprocess_metrics",
        fixture_dir: "benchmarks/tests/fixtures/bench/parsers/vcf/bcftools/vcf.postprocess",
        failure_reason: VcfParserFailureReason::MissingIndex,
        probe_artifact: "raw.postprocess.vcf.tbi",
        expected_error_fragment: "required tabix index for postprocess output is missing",
    },
    VcfParserFailureCase {
        case_id: "missing-sample-column",
        stage: VcfDomainStage::Phasing,
        tool_id: "shapeit5",
        parser_id: "parse_shapeit5_phasing_metrics",
        fixture_dir: "benchmarks/tests/fixtures/bench/parsers/vcf/phasing/shapeit5",
        failure_reason: VcfParserFailureReason::MissingSampleColumn,
        probe_artifact: "raw.phased.vcf",
        expected_error_fragment: "phased VCF row is missing sample columns",
    },
    VcfParserFailureCase {
        case_id: "malformed-pca-table",
        stage: VcfDomainStage::Pca,
        tool_id: "eigensoft",
        parser_id: "parse_eigensoft_pca_metrics",
        fixture_dir: "benchmarks/tests/fixtures/bench/parsers/vcf/eigensoft/pca",
        failure_reason: VcfParserFailureReason::MalformedPcaTable,
        probe_artifact: "raw.evec",
        expected_error_fragment: "does not contain any numeric components",
    },
    VcfParserFailureCase {
        case_id: "malformed-imputation-quality-file",
        stage: VcfDomainStage::Impute,
        tool_id: "beagle",
        parser_id: "parse_beagle_impute_metrics",
        fixture_dir: "benchmarks/tests/fixtures/bench/parsers/vcf/imputation/beagle/vcf.impute",
        failure_reason: VcfParserFailureReason::MalformedImputationQualityFile,
        probe_artifact: "raw.imputation_qc.json",
        expected_error_fragment: "raw.imputation_qc.json",
    },
    VcfParserFailureCase {
        case_id: "malformed-segment-file",
        stage: VcfDomainStage::Ibd,
        tool_id: "germline",
        parser_id: "parse_germline_ibd_segment_metrics",
        fixture_dir:
            "benchmarks/tests/fixtures/bench/parsers/vcf/segments/germline/vcf.ibd/complete",
        failure_reason: VcfParserFailureReason::MalformedSegmentFile,
        probe_artifact: "raw.ibd_filtered_segments.tsv",
        expected_error_fragment: "must have 7 columns",
    },
];
