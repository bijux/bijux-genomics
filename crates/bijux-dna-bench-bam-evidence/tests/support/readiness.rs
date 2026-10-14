use anyhow::{bail, Result};
use bijux_dna_bench_bam_evidence::{
    command_coverage::BamCommandAdapterCoverageReport,
    output_contract::BamAdapterOutputContractReport,
    parser_coverage::BamParserFixtureCoverageReport,
    readiness::{
        ports::{DeclarationEvidence, LocalProofKind, LocalProofWriter},
        ReadinessContext, ReadinessPaths,
    },
    report_map::BamReportMapReport,
};
use bijux_dna_bench_bindings::{
    expected::ExpectedBenchmarkResultsReport, serving::ToolServingMapReport,
};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct RejectedSource {
    pub calls: RefCell<Vec<&'static str>>,
}
impl RejectedSource {
    pub fn context<'a>(&'a self, root: &'a Path) -> ReadinessContext<'a> {
        ReadinessContext {
            repo_root: root,
            declarations: self,
            native: self,
            paths: ReadinessPaths {
                serving: "serving",
                commands: "commands",
                outputs: "outputs",
                parsers: "parsers",
                expected: "expected",
                report_map: "report-map",
                schema: "schema",
            },
        }
    }
    fn reject<T>(&self, surface: &'static str) -> Result<T> {
        self.calls.borrow_mut().push(surface);
        bail!("captured {surface} source rejection")
    }
}
impl DeclarationEvidence for RejectedSource {
    fn serving(&self, _: &Path) -> Result<ToolServingMapReport> {
        self.reject("serving")
    }
    fn commands(&self, _: &Path) -> Result<BamCommandAdapterCoverageReport> {
        self.reject("commands")
    }
    fn outputs(&self, _: &Path) -> Result<BamAdapterOutputContractReport> {
        self.reject("outputs")
    }
    fn parsers(&self, _: &Path) -> Result<BamParserFixtureCoverageReport> {
        self.reject("parsers")
    }
    fn expected(&self, _: &Path) -> Result<ExpectedBenchmarkResultsReport> {
        self.reject("expected")
    }
    fn report_map(&self, _: &Path) -> Result<BamReportMapReport> {
        self.reject("report-map")
    }
    fn schema(&self) -> Result<serde_json::Value> {
        self.reject("schema")
    }
}
impl LocalProofWriter for RejectedSource {
    fn write(&self, _: &Path, _: LocalProofKind) -> Result<PathBuf> {
        self.reject("native")
    }
}

// Acquired declaration fixtures distinguish content admission from source refusal.
pub mod preflight;
