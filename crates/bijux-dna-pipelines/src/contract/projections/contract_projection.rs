use crate::PipelineProfile;

use super::PipelineContract;

impl PipelineProfile {
    #[must_use]
    pub fn contract(&self) -> PipelineContract {
        PipelineContract {
            pipeline_id: self.id.clone(),
            required_stages: self
                .capabilities
                .required_stages
                .iter()
                .map(|stage| (*stage).to_string())
                .collect(),
            required_artifacts: self
                .capabilities
                .required_artifacts
                .iter()
                .map(|artifact| (*artifact).to_string())
                .collect(),
            required_metrics_bundles: self.capabilities.required_metrics_bundles.clone(),
            required_report_sections: self.capabilities.required_report_sections.clone(),
        }
    }
}
