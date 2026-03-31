use super::{
    AnalysisRequirements, BenchmarkStageSpec, BenchmarkSuiteSpec, DatasetSpec,
    DiversityRequirements, ReplicatePolicy, StratificationRequirement,
};

impl BenchmarkSuiteSpec {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        suite_id: String,
        datasets: Vec<DatasetSpec>,
        stages: &[String],
        tools: &[String],
        params: &[String],
        replicate_policy: ReplicatePolicy,
        diversity: DiversityRequirements,
        stratifications: Vec<StratificationRequirement>,
        analysis_requirements: AnalysisRequirements,
    ) -> Self {
        let stage_matrix = stages
            .iter()
            .cloned()
            .map(|stage| BenchmarkStageSpec {
                stage,
                stage_instance_id: None,
                tools: tools.to_vec(),
                params: params.to_vec(),
                param_bindings: Vec::new(),
                upstream_stage_instance_ids: Vec::new(),
            })
            .collect();
        Self::v1_stage_matrix(
            suite_id,
            datasets,
            stage_matrix,
            replicate_policy,
            diversity,
            stratifications,
            analysis_requirements,
        )
    }

    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn v1_stage_matrix(
        suite_id: String,
        datasets: Vec<DatasetSpec>,
        stages: Vec<BenchmarkStageSpec>,
        replicate_policy: ReplicatePolicy,
        diversity: DiversityRequirements,
        stratifications: Vec<StratificationRequirement>,
        analysis_requirements: AnalysisRequirements,
    ) -> Self {
        Self {
            schema_version: "bijux.bench.suite.v1".to_string(),
            suite_id,
            datasets,
            stages,
            edges: Vec::new(),
            replicate_policy,
            diversity,
            stratifications,
            analysis_requirements,
        }
    }
}
