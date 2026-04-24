use std::collections::BTreeMap;

use crate::{DefaultProvenanceV1, DefaultsLedgerV1, PipelineProfile};

impl PipelineProfile {
    #[must_use]
    pub fn defaults_ledger(&self) -> DefaultsLedgerV1 {
        let mut tool_provenance = BTreeMap::new();
        let mut param_provenance = BTreeMap::new();
        for (stage, rationale) in &self.defaults.rationales {
            let provenance = DefaultProvenanceV1 {
                rationale: rationale.clone(),
                assumptions: vec![
                    "defaults chosen for pre-HPC deterministic baseline comparisons".to_string()
                ],
                comparability_implications: vec![
                    "changing this default can shift cross-run comparability baselines".to_string(),
                ],
                citations: vec!["docs/20-science/fastq/GOLD_PIPELINE_SPEC.md".to_string()],
            };
            if self.defaults.tools.contains_key(stage) {
                tool_provenance.insert(stage.clone(), provenance.clone());
            }
            if self.defaults.params.contains_key(stage) {
                param_provenance.insert(stage.clone(), provenance);
            }
        }
        for stage in self.defaults.tools.keys() {
            assert!(
                tool_provenance.contains_key(stage),
                "missing tool provenance rationale for stage {} in pipeline {}",
                stage.as_str(),
                self.id.as_str()
            );
        }
        for stage in self.defaults.params.keys() {
            assert!(
                param_provenance.contains_key(stage),
                "missing parameter provenance rationale for stage {} in pipeline {}",
                stage.as_str(),
                self.id.as_str()
            );
        }
        DefaultsLedgerV1 {
            pipeline_id: self.id.clone(),
            tools: self.defaults.tools.clone(),
            params: self.defaults.params.clone(),
            thresholds: BTreeMap::new(),
            tool_provenance,
            param_provenance,
            assumptions: Vec::new(),
            citations: BTreeMap::new(),
        }
    }
}
