//! Canonical BAM metrics schema v1.

mod alignment;
mod authenticity;
mod bundle;
mod complexity;
mod contamination;
mod coverage;
mod damage;
mod fragment;
mod genotyping;
mod idxstats;
mod mapq;
mod sex;
mod sufficiency;
mod verdict;

pub use alignment::{parse_samtools_flagstat, parse_samtools_stats, AlignmentCountsV1};
pub use authenticity::{
    authenticity_score, contamination_cross_check, infer_library_type_from_damage,
    suggest_trim_from_damage, AuthenticityEvidenceV1, AuthenticityScoreV1, LibraryTypeInferenceV1,
    TrimSuggestionV1,
};
pub use bundle::{BamMetricsBundleV1, BamMetricsV1};
pub use complexity::{parse_preseq_estimates, ComplexityMetricsV1};
pub use contamination::{
    parse_contamination_json, ContaminationMetricsV1, ContaminationReconciliationV1,
};
pub use coverage::{
    parse_mosdepth_summary, parse_samtools_depth, parse_samtools_depth_with_uniformity,
    CoverageMetricsV1, CoverageUniformityV1, EffectiveCoverageV1,
};
pub use damage::{
    compare_damage_metrics, parse_damageprofiler_json, parse_mapdamage2_misincorporation,
    parse_pydamage_json, DamageComparisonV1, DamageMetricsV1,
};
pub use fragment::FragmentLengthSummaryV1;
pub use genotyping::GenotypingMetricsV1;
pub use idxstats::{parse_samtools_idxstats, IdxstatsContigV1, IdxstatsSummaryV1};
pub use mapq::MapqSummaryV1;
pub use sex::{parse_sex_json, SexConfidenceClass, SexInferenceV1};
pub use sufficiency::{
    ContaminationSufficiencyV1, CoverageSufficiencyV1, HaplogroupSufficiencyV1,
    KinshipSufficiencyV1, SexSufficiencyV1,
};
pub use verdict::{BamInvariantStatusV1, BamStageVerdictV1};

pub use crate::invariants::{
    evaluate_bam_invariants, BamInvariantEvaluation, BamInvariantThresholds,
};
