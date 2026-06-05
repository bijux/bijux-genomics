//! Canonical BAM metrics schema v1.

mod catalog;
pub mod core;
pub mod downstream;
mod raw_parser_contract;
pub mod pre;

pub use core::{
    compare_damage_metrics, parse_addeam_json, parse_damageprofiler_json,
    parse_mapdamage2_misincorporation, parse_mosdepth_summary, parse_ngsbriggs_json,
    parse_picard_gc_bias_metrics, parse_picard_insert_size_metrics, parse_pmdtools_json,
    parse_preseq_estimates, parse_pydamage_json, parse_samtools_depth,
    parse_samtools_depth_with_uniformity, AdDeamMetricsV1, BamMetricsBundleV1, BamMetricsV1,
    ComplexityMetricsV1, CoverageMetricsV1, CoverageUniformityV1, DamageComparisonV1,
    DamageCoreFieldsV1, DamageMetricsV1, DamageProfilerMetricsV1, EffectiveCoverageV1,
    GcBiasMetricsV1, InsertSizeMetricsV1, MisincorporationCurveSummaryV1, MisincorporationPointV1,
    NgsBriggsMetricsV1, PmdHistogramBinV1, PmdScoreDistributionV1, PmdtoolsMetricsV1,
};
pub use downstream::{
    authenticity_score, contamination_cross_check, parse_contamination_json, parse_sex_json,
    suggest_trim_from_damage, AuthenticityEvidenceV1, AuthenticityScoreV1, BamInvariantStatusV1,
    BamStageVerdictV1, ContamMixMetricsV1, ContaminationInputScopeV1, ContaminationMetricsV1,
    ContaminationReconciliationV1, ContaminationRequiredInputsV1, ContaminationSufficiencyV1,
    ContaminationToolMetricsV1, ContaminationWarningV1, CoverageSufficiencyV1, GenotypingMetricsV1,
    HaplogroupSufficiencyV1, KinshipSufficiencyV1, LibraryTypeInferenceV1, SchmutziMetricsV1,
    SexConfidenceClass, SexInferenceV1, SexSufficiencyV1, TrimSuggestionV1, VerifyBamId2MetricsV1,
};
pub use pre::{
    parse_samtools_flagstat, parse_samtools_idxstats, parse_samtools_stats, AlignmentCountsV1,
    FragmentLengthSummaryV1, IdxstatsContigV1, IdxstatsSummaryV1, MapqSummaryV1,
};
pub use raw_parser_contract::{
    evaluate_bam_raw_parser_failure_contracts, BamRawParserFailureClass,
    BamRawParserFailureContractRow,
};

pub use crate::invariants::{
    evaluate_bam_invariants, BamInvariantEvaluation, BamInvariantThresholds,
};
