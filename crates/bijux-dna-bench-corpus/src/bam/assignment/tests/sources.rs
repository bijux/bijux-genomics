#![cfg(test)]
use super::super::{
    ports::{
        CompatibilityEvidence, Coverage, CoverageEvidence, InventoryEvidence, StageCompatibility,
    },
    render_bam_corpus_assignment, AssignmentContext,
};
use anyhow::{bail, Context, Result};
use std::{cell::RefCell, fs, path::Path};

struct Evidence {
    fail_inventory: bool,
    calls: RefCell<Vec<&'static str>>,
}
impl InventoryEvidence for Evidence {
    fn stage_count(&self, _: &Path) -> Result<usize> {
        self.calls.borrow_mut().push("inventory");
        if self.fail_inventory {
            bail!("captured inventory failure");
        }
        Ok(24)
    }
}
impl CoverageEvidence for Evidence {
    fn coverage(&self, _: &Path) -> Result<Coverage> {
        self.calls.borrow_mut().push("coverage");
        bail!("coverage must not run before source admission")
    }
}
impl CompatibilityEvidence for Evidence {
    fn compatibility(&self, _: &Path) -> Result<Vec<StageCompatibility>> {
        self.calls.borrow_mut().push("compatibility");
        Ok((0..2)
            .map(|_| StageCompatibility {
                stage_id: bijux_dna_core::id_catalog::BAM_DAMAGE.to_string(),
                corpus_family_id: Some("family".to_string()),
                fixture_id: Some("fixture".to_string()),
            })
            .collect())
    }
}
impl Evidence {
    fn context<'a>(&'a self, root: &'a Path) -> AssignmentContext<'a> {
        AssignmentContext { repo_root: root, inventory: self, coverage: self, compatibility: self }
    }
}

#[test]
fn inventory_failure_preserves_existing_report_and_prevents_later_acquisition() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("report.tsv"), "previous report")?;
    let source = Evidence { fail_inventory: true, calls: RefCell::new(vec![]) };
    let error = render_bam_corpus_assignment(&source.context(root.path()), Path::new("report.tsv"))
        .err()
        .context("inventory must reject")?;
    assert_eq!(error.to_string(), "captured inventory failure");
    assert_eq!(*source.calls.borrow(), ["inventory"]);
    assert_eq!(fs::read_to_string(root.path().join("report.tsv"))?, "previous report");
    Ok(())
}

#[test]
fn duplicate_compatibility_rejects_before_fixture_reads_and_report_replacement() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("report.tsv"), "previous report")?;
    let source = Evidence { fail_inventory: false, calls: RefCell::new(vec![]) };
    let error = render_bam_corpus_assignment(&source.context(root.path()), Path::new("report.tsv"))
        .err()
        .context("duplicate source must reject")?;
    assert!(error.to_string().contains("BAM corpus compatibility repeats stage"));
    assert_eq!(*source.calls.borrow(), ["inventory", "compatibility"]);
    assert_eq!(fs::read_to_string(root.path().join("report.tsv"))?, "previous report");
    Ok(())
}
