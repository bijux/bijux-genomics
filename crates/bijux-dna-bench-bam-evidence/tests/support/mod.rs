use anyhow::{anyhow, Result};
use bijux_dna_bench_bam_evidence::{
    observations::{
        BindingKey, BindingSurface, CommandSurfaces, ExpectedResultBinding, OutputAssessment,
        ReportStage, ServingBinding, ServingInventory, StageAdmissions, StageReadiness,
        ToolAdmission,
    },
    ports::{ConsistencyEvidence, OutputEvidence, ReportEvidence, ServingEvidence},
    EvidenceContext, EvidencePaths,
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    path::Path,
};
pub const STAGES: &[&str] = &[
    "bam.align",
    "bam.validate",
    "bam.qc_pre",
    "bam.mapping_summary",
    "bam.filter",
    "bam.mapq_filter",
    "bam.length_filter",
    "bam.overlap_correction",
    "bam.markdup",
    "bam.duplication_metrics",
    "bam.complexity",
    "bam.coverage",
    "bam.insert_size",
    "bam.gc_bias",
    "bam.endogenous_content",
    "bam.damage",
    "bam.authenticity",
    "bam.contamination",
    "bam.sex",
    "bam.haplogroups",
    "bam.kinship",
    "bam.bias_mitigation",
    "bam.recalibration",
    "bam.genotyping",
];
fn tool(stage: &str) -> &str {
    match stage {
        "bam.align" => "bwa",
        "bam.damage" => "mapdamage2",
        "bam.contamination" => "schmutzi",
        "bam.sex" => "rxy",
        "bam.recalibration" => "gatk",
        "bam.kinship" => "king",
        _ => "observed-tool",
    }
}
#[derive(Default)]
pub struct Evidence {
    pub calls: RefCell<Vec<&'static str>>,
    pub generation: Cell<usize>,
    pub duplicate_expected: Cell<bool>,
    pub missing_stage: Cell<bool>,
    pub fail_output: Cell<bool>,
    pub missing_output: Cell<bool>,
    pub command_drift: Cell<bool>,
    pub duplicate_active: Cell<bool>,
    pub serving_override: RefCell<Option<Vec<ServingBinding>>>,
}
impl Evidence {
    pub fn context<'a>(&'a self, repo_root: &'a Path) -> EvidenceContext<'a> {
        EvidenceContext {
            repo_root,
            serving: self,
            output: self,
            reporting: self,
            consistency: self,
            paths: EvidencePaths {
                output_contract: "reports/outputs.tsv",
                parser_coverage: "reports/parsers.tsv",
                report_map: "reports/map.tsv",
                slurm_run_id: "admitted-run",
            },
        }
    }
    pub fn bindings() -> Vec<BindingKey> {
        STAGES
            .iter()
            .map(|stage| BindingKey {
                stage_id: (*stage).to_string(),
                tool_id: tool(stage).to_string(),
            })
            .collect()
    }
    fn surface(path: &str) -> BindingSurface {
        let bindings = Self::bindings();
        BindingSurface {
            output_path: path.to_string(),
            row_count: bindings.len(),
            stage_count: 24,
            tool_count: 7,
            bindings,
        }
    }
    pub fn serving_rows() -> Vec<ServingBinding> {
        Self::bindings()
            .into_iter()
            .map(|key| ServingBinding {
                stage_id: key.stage_id,
                tool_id: key.tool_id,
                support_status: "supported".to_string(),
                adapter_status: "runnable".to_string(),
                parser_status: "parser_fixture_validated".to_string(),
                corpus_status: "fixture:observed-corpus".to_string(),
            })
            .collect()
    }
    fn serving_inventory(&self) -> ServingInventory {
        let rows = self.serving_override.borrow().clone().unwrap_or_else(Self::serving_rows);
        ServingInventory { stage_count: 24, tool_count: 7, rows }
    }
}
impl ServingEvidence for Evidence {
    fn serving(&self, _: &Path) -> Result<ServingInventory> {
        self.calls.borrow_mut().push("serving");
        Ok(self.serving_inventory())
    }
    fn publish_serving(&self, _: &Path) -> Result<ServingInventory> {
        self.calls.borrow_mut().push("publish_serving");
        Ok(self.serving_inventory())
    }
}
impl OutputEvidence for Evidence {
    fn stage_output_ids(&self, _: &Path, _: &str) -> Vec<String> {
        vec!["declared.metrics".to_string()]
    }
    fn assess_output(&self, _: &Path, _: &str, _: &str) -> Result<OutputAssessment> {
        self.calls.borrow_mut().push("assess_output");
        if self.fail_output.get() {
            return Err(anyhow!("captured planner rejection"));
        }
        Ok(OutputAssessment {
            complete: !self.missing_output.get(),
            declared_output_ids: vec!["declared.metrics".to_string()],
            execution_expected_output_ids: vec!["declared.metrics".to_string()],
            stage_expected_artifact_ids: vec!["declared.metrics".to_string()],
            normalized_metrics_output_id: Some("declared.metrics".to_string()),
            raw_output_artifact_ids: Vec::new(),
            missing_declarations: if self.missing_output.get() {
                vec!["execution_contract.raw".to_string()]
            } else {
                Vec::new()
            },
        })
    }
}
impl ReportEvidence for Evidence {
    fn stage_inventory(&self, _: &Path) -> Result<Vec<StageReadiness>> {
        self.calls.borrow_mut().push("stage_inventory");
        Ok(STAGES
            .iter()
            .map(|stage| StageReadiness {
                stage_id: (*stage).to_string(),
                readiness_kind: "dry_run",
            })
            .collect())
    }
    fn stage_admissions(&self, _: &Path) -> Result<StageAdmissions> {
        self.calls.borrow_mut().push("stage_admissions");
        Ok(STAGES
            .iter()
            .map(|stage| {
                (
                    (*stage).to_string(),
                    vec![ToolAdmission {
                        tool_id: tool(stage).to_string(),
                        support_status: "supported".to_string(),
                    }],
                )
            })
            .collect::<BTreeMap<_, _>>())
    }
    fn report_stage(&self, _: &Path, stage: &str) -> Result<ReportStage> {
        self.calls.borrow_mut().push("report_stage");
        if self.missing_stage.get() {
            return Err(anyhow!("missing domain stage observation"));
        }
        let branch = match stage {
            "bam.damage" | "bam.contamination" => Some("ancient_dna_authenticity"),
            "bam.sex" | "bam.kinship" => Some("sample_identity"),
            "bam.recalibration" => Some("variant_readiness"),
            _ => None,
        };
        Ok(ReportStage {
            canonical_rank: STAGES.iter().position(|item| *item == stage).map(|rank| rank + 1),
            criticality: Some(if stage == "bam.align" { "optional" } else { "essential" }),
            preferred_tool_id: tool(stage).to_string(),
            workflow_branch_id: branch.map(str::to_string),
            scientific_context_required: vec![format!(
                "reference-generation-{}",
                self.generation.get()
            )],
        })
    }
    fn expected_results(&self, _: &Path) -> Result<Vec<ExpectedResultBinding>> {
        self.calls.borrow_mut().push("expected_results");
        let mut rows = STAGES
            .iter()
            .map(|stage| ExpectedResultBinding {
                result_row_id: format!("{stage}:{}", tool(stage)),
                domain: "bam".to_string(),
                stage_id: (*stage).to_string(),
                tool_id: tool(stage).to_string(),
                corpus_family_id: "observed-corpus".to_string(),
                fixture_id: "fixture".to_string(),
                sample_scope: "sample".to_string(),
            })
            .collect::<Vec<_>>();
        if self.duplicate_expected.get() {
            rows.push(rows[0].clone());
        }
        rows.reverse();
        Ok(rows)
    }
}
impl ConsistencyEvidence for Evidence {
    fn publish_active_bindings(&self, _: &Path) -> Result<BindingSurface> {
        self.calls.borrow_mut().push("publish_active");
        let mut surface = Self::surface("observed-active.json");
        if self.duplicate_active.get() {
            surface.bindings.push(surface.bindings[0].clone());
            surface.row_count += 1;
        }
        Ok(surface)
    }
    fn publish_commands(&self, _: &Path) -> Result<CommandSurfaces> {
        self.calls.borrow_mut().push("publish_commands");
        let mut commands = Self::surface("observed-commands.tsv");
        if self.command_drift.get() {
            commands.bindings[0].tool_id = "unadmitted-tool".to_string();
        }
        Ok(CommandSurfaces { commands, argv_output_path: "observed-argv.json".to_string() })
    }
    fn publish_expected_results(&self, _: &Path) -> Result<BindingSurface> {
        self.calls.borrow_mut().push("publish_expected");
        Ok(Self::surface("observed-expected.tsv"))
    }
}

pub mod readiness;
