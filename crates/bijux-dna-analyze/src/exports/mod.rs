//! Owner: bijux-dna-analyze
//! Facts export and summary helpers.

mod dashboard_facts;
mod evidence_bundle;
mod facts_summary;
mod facts_support;
mod run_summary;
mod stage_summary;

pub use dashboard_facts::write_dashboard_facts_jsonl;
pub use evidence_bundle::{
    build_evidence_bundle, compare_evidence_bundles, list_reviewer_challenges,
    submit_reviewer_challenge, validate_evidence_bundle_profile, verify_evidence_bundle,
    verify_profile_bundle, write_evidence_bundle_json, write_methods_summary_json,
    write_profile_bundle_json, EvidenceArchiveMigrationV1, EvidenceArtifactV1,
    EvidenceBundleFileDigestV1, EvidenceBundleProfileV1, EvidenceBundleV1, EvidenceCheckV1,
    EvidenceCitationTypeV1, EvidenceCitationV1, EvidenceCompactSummaryV1, EvidenceComparisonV1,
    EvidenceEdgeV1, EvidenceGapV1, EvidenceHealthV1, EvidenceMethodsSummaryV1,
    EvidenceMethodsToolV1, EvidenceMetricsV1, EvidenceNodeV1, EvidenceProfileBundleV1,
    EvidenceProfileBundleVerificationV1, EvidenceProfileCheckV1, EvidenceProfileValidationV1,
    EvidenceProvenanceGraphV1, EvidenceSeverityV1, EvidenceSourcesV1, EvidenceTimelineCategoryV1,
    EvidenceTimelineEventV1, EvidenceVerificationV1, ReviewerChallengeRecordV1,
    ReviewerChallengeRequestV1,
};
pub use facts_summary::summarize_facts;
pub use run_summary::write_run_summary_json;
pub use stage_summary::write_stage_summary_csv;
