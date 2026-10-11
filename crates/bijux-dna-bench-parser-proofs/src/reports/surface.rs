use super::ParseKind;

#[must_use]
pub fn parse_surface_label(parse_kind: ParseKind) -> &'static str {
    match parse_kind {
        ParseKind::FastqIndexReference => "fastq::parse_index_reference_report",
        ParseKind::FastqCorrectErrors => "fastq::parse_correct_errors_report",
        ParseKind::FastqDepleteHost => "fastq::parse_deplete_host_report",
        ParseKind::FastqDepleteRrna => "fastq::parse_deplete_rrna_report",
        ParseKind::FastqTrimReads => "fastq::parse_trim_reads_report",
        ParseKind::FastqMergePairs => "fastq::parse_merge_pairs_report",
        ParseKind::FastqFilterReads => "fastq::parse_filter_reads_report",
        ParseKind::FastqProfileReads => "fastq::parse_profile_reads_report",
        ParseKind::FastqReportQc => "fastq::parse_report_qc_report",
        ParseKind::FastqRemoveDuplicates => "fastq::parse_remove_duplicates_report",
        ParseKind::FastqExtractUmis => "fastq::parse_extract_umis_report",
        ParseKind::FastqScreenTaxonomy => "fastq::parse_screen_taxonomy_report",
        ParseKind::FastqInferAsvs => "fastq::parse_infer_asvs_report",
        ParseKind::FastqNormalizeAbundance => "fastq::parse_normalize_abundance_report",
        ParseKind::FastqDetectDuplicatesPremerge => {
            "fastq::parse_detect_duplicates_premerge_report"
        }
        ParseKind::BamValidationSummary => "serde_json::<BamValidationSummaryV1>",
        ParseKind::BamAlignmentProvenance => "serde_json::<BamAlignmentProvenanceV1>",
        ParseKind::BamContaminationJson => "bam::parse_contamination_json",
        ParseKind::BamOverlapCorrectionSummary => "serde_json::<BamOverlapCorrectionSummaryV1>",
        ParseKind::BamRecalibrationSummary => "serde_json::<BamRecalibrationSummaryV1>",
        ParseKind::BamDamageEvidence => "serde_json::<BamDamageEvidenceV1>",
        ParseKind::BamAuthenticityAdvisory => "serde_json::<BamAuthenticityAdvisoryV1>",
        ParseKind::BamComplexitySummary => "serde_json::<BamComplexitySummaryV1>",
        ParseKind::BamKinshipSummary => "serde_json::<BamKinshipSummaryV1>",
        ParseKind::BamSexJson => "bam::parse_sex_json",
        ParseKind::BamGenotypingJson => "serde_json::<Value> + governed keys",
        ParseKind::VcfCallMetrics => "serde_json::<Value> + governed vcf call metric keys",
        ParseKind::VcfDemographyReport => {
            "serde_json::<Value> + governed vcf demographic-history keys"
        }
        ParseKind::VcfIbdReport => "serde_json::<Value> + governed vcf identity-by-descent keys",
        ParseKind::VcfImputeMetrics => "serde_json::<Value> + governed vcf impute metric keys",
        ParseKind::VcfPhasingMetrics => "serde_json::<Value> + governed vcf phasing metric keys",
        ParseKind::VcfPopulationStructureReport => {
            "serde_json::<Value> + governed vcf population-structure keys"
        }
        ParseKind::VcfQcMetrics => "serde_json::<Value> + governed vcf qc metric keys",
    }
}
