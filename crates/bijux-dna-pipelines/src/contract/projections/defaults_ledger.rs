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
                citations: vec![default_citation_for_stage(stage.as_str()).to_string()],
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

fn default_citation_for_stage(stage_id: &str) -> &'static str {
    if stage_id.starts_with("bam.") {
        "docs/20-science/bam/GOLD_PIPELINE_SPEC.md"
    } else if stage_id.starts_with("vcf.") {
        "docs/20-science/vcf/GOLD_PIPELINE_SPEC.md"
    } else if stage_id.starts_with("core.") || stage_id.starts_with("cross.") {
        "docs/20-science/cross/GOLD_PIPELINE_SPEC.md"
    } else {
        "docs/20-science/fastq/GOLD_PIPELINE_SPEC.md"
    }
}
