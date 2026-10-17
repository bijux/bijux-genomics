use anyhow::{bail, Result};
use bijux_dna_bench_fastq::readiness::{
    declarations::{
        observations::{
            ExpectedResultBinding, InventoryReadinessKind, InventoryStage, StageAdmission,
            StageInventory,
        },
        ports::ReportEvidence,
        ReportContext,
    },
    report_map::collect_fastq_report_stage_metadata,
};
use std::{collections::BTreeMap, path::Path};
struct Evidence {
    stage: &'static str,
    admissions: Vec<StageAdmission>,
}
impl ReportEvidence for Evidence {
    fn expected(&self, _: &Path) -> Result<Vec<ExpectedResultBinding>> {
        bail!("metadata must not acquire expected result rows")
    }
    fn inventory(&self, _: &Path) -> Result<StageInventory> {
        Ok(StageInventory {
            stages: vec![InventoryStage {
                stage_id: self.stage.into(),
                readiness_kind: InventoryReadinessKind::DryOrSmoke,
            }],
        })
    }
    fn admissions(&self, _: &Path) -> Result<BTreeMap<String, Vec<StageAdmission>>> {
        Ok(BTreeMap::from([(self.stage.into(), self.admissions.clone())]))
    }
}
fn admission(tool: &str, status: &str) -> StageAdmission {
    StageAdmission { tool_id: tool.into(), support_status: status.into() }
}
#[test]
fn preferred_stage_anchor_retains_its_declared_support_status() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence {
        stage: "fastq.report_qc",
        admissions: vec![admission("other", "supported"), admission("multiqc", "planned")],
    };
    let rows = collect_fastq_report_stage_metadata(&ReportContext {
        repo_root: root.path(),
        evidence: &evidence,
    })?;
    let row = &rows["fastq.report_qc"];
    assert_eq!(row.anchor_tool_id, "multiqc");
    assert_eq!(row.anchor_support_status, "planned");
    assert!(row.produces_reports_only);
    assert_eq!(row.report_section_id, "quality_profiling");
    Ok(())
}
#[test]
fn absent_preferred_anchor_selects_support_rank_before_tool_name() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence = Evidence {
        stage: "fastq.report_qc",
        admissions: vec![
            admission("a", "planned"),
            admission("z", "supported"),
            admission("y", "supported"),
        ],
    };
    let rows = collect_fastq_report_stage_metadata(&ReportContext {
        repo_root: root.path(),
        evidence: &evidence,
    })?;
    assert_eq!(rows["fastq.report_qc"].anchor_tool_id, "y");
    Ok(())
}
#[test]
fn ungoverned_stage_refuses_report_placement() -> Result<()> {
    let root = tempfile::tempdir()?;
    let evidence =
        Evidence { stage: "fastq.unknown", admissions: vec![admission("tool", "supported")] };
    assert!(collect_fastq_report_stage_metadata(&ReportContext {
        repo_root: root.path(),
        evidence: &evidence
    })
    .err()
    .is_some_and(|error| error.to_string().contains("missing stage placement")));
    Ok(())
}

#[test]
fn published_report_uses_one_expected_result_and_admission_observation() -> Result<()> {
    use bijux_dna_bench_fastq::readiness::report_map::render_fastq_report_map;
    use std::cell::Cell;

    const BINDINGS: [(&str, &str); 7] = [
        ("fastq.index_reference", "bowtie2_build"),
        ("fastq.validate_reads", "fastqvalidator"),
        ("fastq.profile_reads", "seqkit_stats"),
        ("fastq.estimate_library_complexity_prealign", "bijux_dna"),
        ("fastq.trim_reads", "fastp"),
        ("fastq.screen_taxonomy", "kraken2"),
        ("fastq.infer_asvs", "dada2"),
    ];
    struct Observations {
        expected_calls: Cell<usize>,
        admission_calls: Cell<usize>,
    }
    impl ReportEvidence for Observations {
        fn expected(&self, _: &Path) -> Result<Vec<ExpectedResultBinding>> {
            let call = self.expected_calls.get();
            self.expected_calls.set(call + 1);
            let mut rows = BINDINGS
                .iter()
                .map(|(stage, tool)| ExpectedResultBinding {
                    domain: "fastq".into(),
                    result_row_id: format!("{stage}:{tool}:fixture"),
                    stage_id: (*stage).into(),
                    tool_id: (*tool).into(),
                    corpus_family_id: "fixture-family".into(),
                    fixture_id: "fixture".into(),
                    sample_scope: "sample-set".into(),
                })
                .collect::<Vec<_>>();
            if call > 0 {
                let mut extra = rows[0].clone();
                extra.result_row_id.push_str(":changed");
                extra.fixture_id = "changed-fixture".into();
                rows.push(extra);
            }
            Ok(rows)
        }
        fn inventory(&self, _: &Path) -> Result<StageInventory> {
            Ok(StageInventory {
                stages: BINDINGS
                    .iter()
                    .map(|(stage, _)| InventoryStage {
                        stage_id: (*stage).into(),
                        readiness_kind: InventoryReadinessKind::DryOrSmoke,
                    })
                    .collect(),
            })
        }
        fn admissions(&self, _: &Path) -> Result<BTreeMap<String, Vec<StageAdmission>>> {
            self.admission_calls.set(self.admission_calls.get() + 1);
            Ok(BINDINGS
                .iter()
                .map(|(stage, tool)| ((*stage).into(), vec![admission(tool, "supported")]))
                .collect())
        }
    }
    let root = tempfile::tempdir()?;
    let observations = Observations { expected_calls: Cell::new(0), admission_calls: Cell::new(0) };
    let output = root.path().join("report-map.tsv");
    let report = render_fastq_report_map(
        &ReportContext { repo_root: root.path(), evidence: &observations },
        &output,
    )?;
    assert!(output.is_file(), "the regression must reach real publication");
    assert_eq!(report.expected_result_row_count, report.row_count);
    assert_eq!(report.rows.len(), BINDINGS.len());
    assert_eq!(observations.expected_calls.get(), 1);
    assert_eq!(observations.admission_calls.get(), 1);
    Ok(())
}
