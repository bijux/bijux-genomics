use anyhow::Result;
use bijux_dna_bench_bindings::{
    observations::{AssignmentKind, CorpusAssignment, Domain, OutputContract},
    ports::ExpectedEvidence,
    ExpectedContext,
};
use bijux_dna_core::id_catalog::{
    BAM_DAMAGE, BAM_KINSHIP, FASTQ_INDEX_REFERENCE, FASTQ_SCREEN, FASTQ_STATS_NEUTRAL,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;
pub(crate) struct ExpectedFixture {
    pub fastq_outputs: Vec<OutputContract>,
    pub bam_outputs: Vec<OutputContract>,
    pub fastq_corpus: Vec<CorpusAssignment>,
    pub bam_corpus: Vec<CorpusAssignment>,
    pub scopes: BTreeMap<String, String>,
    pub kinds: BTreeMap<String, String>,
    pub calls: RefCell<Vec<String>>,
}
impl ExpectedFixture {
    pub fn new() -> Self {
        let mut fixture = Self {
            fastq_outputs: Vec::new(),
            bam_outputs: Vec::new(),
            fastq_corpus: Vec::new(),
            bam_corpus: Vec::new(),
            scopes: BTreeMap::new(),
            kinds: BTreeMap::new(),
            calls: RefCell::default(),
        };
        for (domain, stage, tool, corpus, scope, normalized) in [
            (
                Domain::Fastq,
                FASTQ_SCREEN,
                "kraken2",
                "corpus-02-edna-mini",
                "sample-set",
                "classification_report_json",
            ),
            (
                Domain::Fastq,
                FASTQ_INDEX_REFERENCE,
                "bowtie2_build",
                "reference-index-assets",
                "asset-set",
                "report_json",
            ),
            (
                Domain::Fastq,
                FASTQ_STATS_NEUTRAL,
                "seqkit_stats",
                "corpus-01-mini",
                "sample-set",
                "qc_json",
            ),
            (
                Domain::Bam,
                BAM_DAMAGE,
                "mapdamage2",
                "corpus-01-adna-damage-mini",
                "adna_damage_non_udg",
                "damage_report",
            ),
            (
                Domain::Bam,
                BAM_KINSHIP,
                "king",
                "corpus-01-kinship-mini",
                "sample-set",
                "kinship_report",
            ),
        ] {
            let output = OutputContract {
                stage_id: stage.into(),
                tool_id: tool.into(),
                complete: true,
                execution_expected_output_ids: vec!["native_output".into(), normalized.into()],
                raw_output_artifact_ids: vec!["native_output".into()],
                normalized_metrics_output_id: Some(normalized.into()),
            };
            let asset = scope == "asset-set";
            let assignment = CorpusAssignment {
                stage_id: stage.into(),
                tool_id: tool.into(),
                benchmark_status: "benchmark_ready".into(),
                assignment_status: if asset {
                    AssignmentKind::AssetBacked
                } else {
                    AssignmentKind::Assigned
                },
                corpus_family_id: if asset { None } else { Some("fixture-family".into()) },
                fixture_id: if asset { None } else { Some(corpus.into()) },
                benchmark_scope_id: if asset { Some(corpus.into()) } else { None },
            };
            if domain == Domain::Fastq {
                fixture.fastq_outputs.push(output);
                fixture.fastq_corpus.push(assignment);
            } else {
                fixture.bam_outputs.push(output);
                fixture.bam_corpus.push(assignment);
            }
            fixture.scopes.insert(corpus.into(), scope.into());
            fixture.kinds.insert(stage.into(), "benchmark".into());
        }
        fixture
    }
    pub fn context<'a>(&'a self, root: &'a Path) -> ExpectedContext<'a> {
        ExpectedContext {
            repo_root: root,
            evidence: self,
            result_root: "chosen/results",
            run_id: "declared-run",
        }
    }
    fn record(&self, name: &str) {
        self.calls.borrow_mut().push(name.to_string());
    }
}
impl ExpectedEvidence for ExpectedFixture {
    fn fixture_sample_scopes(&self, _root: &Path) -> Result<BTreeMap<String, String>> {
        self.record("scopes");
        Ok(self.scopes.clone())
    }
    fn output_contracts(&self, _root: &Path, domain: Domain) -> Result<Vec<OutputContract>> {
        self.record(&format!("{} outputs", domain.as_str()));
        Ok(if domain == Domain::Fastq {
            self.fastq_outputs.clone()
        } else {
            self.bam_outputs.clone()
        })
    }
    fn corpus_assignments(&self, _root: &Path, domain: Domain) -> Result<Vec<CorpusAssignment>> {
        self.record(&format!("{} corpus", domain.as_str()));
        Ok(if domain == Domain::Fastq {
            self.fastq_corpus.clone()
        } else {
            self.bam_corpus.clone()
        })
    }
    fn readiness_kinds(&self, _root: &Path, domain: Domain) -> Result<BTreeMap<String, String>> {
        self.record(&format!("{} kinds", domain.as_str()));
        Ok(self.kinds.clone())
    }
}
