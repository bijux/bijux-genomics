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
mod mapq;
mod sex;
mod sufficiency;

pub use alignment::AlignmentCountsV1;
pub use authenticity::{
    AuthenticityEvidenceV1, AuthenticityScoreV1, LibraryTypeInferenceV1, TrimSuggestionV1,
};
pub use bundle::{BamMetricsBundleV1, BamMetricsV1};
pub use complexity::ComplexityMetricsV1;
pub use contamination::{ContaminationMetricsV1, ContaminationReconciliationV1};
pub use coverage::{CoverageMetricsV1, CoverageUniformityV1, EffectiveCoverageV1};
pub use damage::DamageMetricsV1;
pub use fragment::FragmentLengthSummaryV1;
pub use genotyping::GenotypingMetricsV1;
pub use mapq::MapqSummaryV1;
pub use sex::{SexConfidenceClass, SexInferenceV1};
pub use sufficiency::{
    ContaminationSufficiencyV1, CoverageSufficiencyV1, HaplogroupSufficiencyV1,
    KinshipSufficiencyV1, SexSufficiencyV1,
};
