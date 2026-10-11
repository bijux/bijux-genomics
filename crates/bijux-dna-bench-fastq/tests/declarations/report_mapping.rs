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
