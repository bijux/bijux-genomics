use anyhow::Result;
use bijux_dna_bench_bindings::{
    observations::{
        BamStageAssessment, BamToolAssessment, CorpusDeclaration, Domain, FastqAssessment,
        StageInventory,
    },
    ports::ServingEvidence,
    ServingContext,
};
use bijux_dna_core::id_catalog::{
    BAM_DAMAGE, FASTQ_CLUSTER_OTUS, FASTQ_INFER_ASVS, FASTQ_NORMALIZE_ABUNDANCE,
    FASTQ_NORMALIZE_PRIMERS, FASTQ_REMOVE_CHIMERAS,
};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::path::Path;
pub(crate) struct ServingFixture {
    pub declarations: Vec<CorpusDeclaration>,
    pub tools: Vec<String>,
    pub integrated: BTreeSet<String>,
    pub flags: (bool, bool, bool),
    pub extra: Option<FastqAssessment>,
    pub calls: RefCell<Vec<String>>,
}
impl ServingFixture {
    pub fn new() -> Self {
        let declarations = [
            BAM_DAMAGE,
            FASTQ_NORMALIZE_PRIMERS,
            FASTQ_REMOVE_CHIMERAS,
            FASTQ_INFER_ASVS,
            FASTQ_CLUSTER_OTUS,
            FASTQ_NORMALIZE_ABUNDANCE,
        ]
        .into_iter()
        .map(|stage| CorpusDeclaration {
            stage_id: stage.into(),
            compatibility_kind: "fixture".into(),
            fixture_id: Some("corpus-03-amplicon-mini".into()),
            benchmark_scope_id: None,
        })
        .collect();
        Self {
            declarations,
            tools: vec!["zeta".into(), "alpha".into(), "unintegrated".into()],
            integrated: ["zeta", "alpha", "cutadapt", "vsearch", "dada2", "seqkit"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            flags: (true, true, true),
            extra: None,
            calls: RefCell::default(),
        }
    }
    pub fn context<'a>(&'a self, root: &'a Path) -> ServingContext<'a> {
        ServingContext { repo_root: root, evidence: self }
    }
    fn record(&self, name: &str) {
        self.calls.borrow_mut().push(name.to_string());
    }
}
impl ServingEvidence for ServingFixture {
    fn corpus_declarations(&self, _root: &Path) -> Result<Vec<CorpusDeclaration>> {
        self.record("corpus");
        Ok(self.declarations.clone())
    }
    fn integrated_tools(&self, _root: &Path) -> Result<BTreeSet<String>> {
        self.record("integrated");
        Ok(self.integrated.clone())
    }
    fn stage_inventory(&self, _root: &Path, domain: Domain) -> Result<StageInventory> {
        self.record("inventory");
        let stages = if domain == Domain::Bam {
            vec![BAM_DAMAGE.into()]
        } else {
            self.declarations
                .iter()
                .filter(|row| row.stage_id != BAM_DAMAGE)
                .map(|row| row.stage_id.clone())
                .collect()
        };
        Ok(StageInventory { stage_count: if domain == Domain::Bam { 1 } else { 5 }, stages })
    }
    fn fastq_tools(&self, _root: &Path, stage: &str) -> Result<Vec<String>> {
        let tool = match stage {
            FASTQ_NORMALIZE_PRIMERS => "cutadapt",
            FASTQ_REMOVE_CHIMERAS | FASTQ_CLUSTER_OTUS => "vsearch",
            FASTQ_INFER_ASVS => "dada2",
            _ => "seqkit",
        };
        let mut tools = vec![tool.into()];
        if self.extra.is_some() && stage == FASTQ_NORMALIZE_PRIMERS {
            tools.push("alpha".into());
        }
        Ok(tools)
    }
    fn fastq_assessment(&self, _root: &Path, _stage: &str, tool: &str) -> Result<FastqAssessment> {
        if tool == "alpha" {
            if let Some(extra) = &self.extra {
                return Ok(extra.clone());
            }
        }
        Ok(FastqAssessment {
            readiness: if tool == "dada2" {
                "governed_execution"
            } else {
                "governed_benchmark_cohort"
            },
            active: true,
            runnable: true,
            plannable: true,
            declared: true,
            comparable: false,
            benchmark_normalized: tool != "dada2",
            parse_normalized: true,
        })
    }
    fn bam_stage(&self, _root: &Path, _stage: &str) -> Result<BamStageAssessment> {
        Ok(BamStageAssessment {
            tools: self.tools.clone(),
            has_parser_fixtures: self.flags.0,
            has_scientific_report_contract: self.flags.1,
            has_artifact_contract: self.flags.2,
        })
    }
    fn bam_assessment(&self, _root: &Path, _stage: &str, tool: &str) -> Result<BamToolAssessment> {
        self.record(tool);
        Ok(BamToolAssessment { support: "declared_supported", adapter: "runnable" })
    }
}
