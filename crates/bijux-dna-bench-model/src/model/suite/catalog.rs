//! Owner: bijux-dna-bench-model
//! Benchmark suite catalog helpers.

use super::BenchmarkSuiteSpec;

impl BenchmarkSuiteSpec {
    #[must_use]
    pub fn stage_ids(&self) -> Vec<&str> {
        self.stages.iter().map(|stage| stage.stage.as_str()).collect()
    }

    #[must_use]
    pub fn tool_ids(&self) -> Vec<&str> {
        let mut tool_ids = std::collections::BTreeSet::new();
        for stage in &self.stages {
            for tool in &stage.tools {
                tool_ids.insert(tool.as_str());
            }
        }
        tool_ids.into_iter().collect()
    }

    #[must_use]
    pub fn params(&self) -> Vec<&str> {
        let mut params = std::collections::BTreeSet::new();
        for stage in &self.stages {
            for value in &stage.params {
                params.insert(value.as_str());
            }
            for binding in &stage.param_bindings {
                for key in binding.values.keys() {
                    params.insert(key.as_str());
                }
            }
        }
        params.into_iter().collect()
    }
}
