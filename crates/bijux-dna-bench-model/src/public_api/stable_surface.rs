//! Owner: bijux-dna-bench-model
//! Stable crate-root re-exports.

pub use crate::compare;
pub use crate::contract;
pub use crate::diagnostics::BenchError;
pub use crate::model::decision::DecisionRationale;
pub use crate::model::graph::{BenchmarkGraphNode, BenchmarkGraphNodeKind, BenchmarkStageEdge};
pub use crate::model::observation::MetricsEnvelope;
pub use crate::model::{
    BenchmarkCorpusManifest, CorpusDatasetSpec, CorpusDomain, CorpusScale, TruthSetHook,
    TruthSetStatus,
};
pub use crate::model::suite::{
    AnalysisRequirements, BenchmarkParamBinding, BenchmarkStageSpec, DatasetSpec,
    DiversityRequirements, ReplicatePolicy, StratificationRequirement,
};
pub use crate::model::summary::{MetricSummary, SummaryRow, SummaryStratum};
pub use crate::model::{
    BenchmarkDecision, BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary,
};
pub use crate::policy::{GateDecision, GatePolicy, GatePolicyOverrides, GateViolation};
pub use crate::stats::robust_estimators::robust_stats;
