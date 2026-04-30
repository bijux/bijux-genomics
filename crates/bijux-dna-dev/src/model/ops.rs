#[derive(Debug, Clone)]
pub struct OpsCommandDefinition {
    pub id: String,
    pub summary: String,
    pub command: OpsCommandSpec,
}

#[derive(Debug, Clone)]
pub enum OpsCommandSpec {
    Native { key: NativeOpsCommandKey },
}

#[derive(Debug, Clone, Copy)]
pub enum NativeOpsCommandKey {
    AssetsRefreshGolden,
    AssetsRefreshReference,
    AssetsRefreshToy,
    AssetsValidateReference,
    DocsCheckDocAssets,
    DocsCheckDocDepth,
    DocsCheckDocLinks,
    DocsCheckDocRootLayout,
    DocsCheckDocsGraph,
    DocsCheckDomainDocReferences,
    DocsCheckGeneratedDocs,
    DocsCheckNoPlaceholderLanguage,
    DocsCheckRootPollution,
    DocsCheckDocMajorDepth,
    ExamplesGenerateIndex,
    ExamplesCheckIndex,
    ExamplesRun,
    ExamplesCheckDrift,
    HpcValidateFrontendConstraints,
    HpcRunFrontendMiniE2e,
    HpcBenchmarkSyncPull,
    HpcBenchmarkSyncPush,
    LabRunBench,
    LabRunPipelines,
    SmokeRun,
    SmokeBam,
    SmokeFastq,
    TestControlPlaneSmoke,
    TestTriage,
    TestReproduceFailure,
    TestFastqGoldRepro,
    TestToyRuns,
    ToolingCargoTargets,
    ToolingGenerateCompatibilityMatrix,
    ToolingCheckConfigSnapshot,
    ToolingCheckConfigPaths,
    ToolingCiAudit,
    ToolingCiClippy,
    ToolingCiClippyExecutors,
    ToolingCiCoverage,
    ToolingCiFast,
    ToolingCiFmt,
    ToolingCiInstallTools,
    ToolingCiSlow,
    ToolingCiTest,
    ToolingCiTestSlow,
    ToolingCleanDocs,
    ToolingCertificationGate,
    ToolingCertifyLevel1,
    ToolingCertifyAll,
    ToolingCertifyBam,
    ToolingCertifyDomains,
    ToolingCertifyFastq,
    ToolingCertifyVcf,
    ToolingAcquireMaps,
    ToolingAcquirePanels,
    ToolingAcquireReference,
    ToolingReferenceExternalData,
    ToolingArchitectureReport,
    ToolingBenchmarkSmokeLevel1,
    ToolingBenchmarkIntegrityMini,
    ToolingConfigInventory,
    ToolingCoverageSummary,
    ToolingCrashTriage,
    ToolingDeprecateVcfKnob,
    ToolingDeprecateVcfPanel,
    ToolingDocsBuild,
    ToolingFlakeHunt,
    ToolingGenerateConfigs,
    ToolingGeneratePanelCompatibilityMatrix,
    ToolingGeneratePolicyIndex,
    ToolingGenerateDocs,
    ToolingGenerateDocsGraph,
    ToolingGenerateConfigTreeSnapshot,
    ToolingGenerateDomainCoverageDoc,
    ToolingGenerateRepoRootMap,
    ToolingGenerateToolIndex,
    ToolingImageQa,
    ToolingInventory,
    ToolingLintFast,
    ToolingMakeHelp,
    ToolingRepoDoctor,
    ToolingRunBijux,
    ToolingSetupDocsVenv,
    ToolingSimulateCoverageRegime,
    ToolingValidateFrontendMiniDomainStacks,
}

#[derive(Debug, Clone)]
pub struct OpsCommandOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl OpsCommandOutcome {
    #[must_use]
    pub fn success(stdout: impl Into<String>) -> Self {
        Self { exit_code: 0, stdout: stdout.into(), stderr: String::new() }
    }

    #[must_use]
    pub fn failure(stderr: impl Into<String>) -> Self {
        Self { exit_code: 1, stdout: String::new(), stderr: stderr.into() }
    }

    #[must_use]
    pub fn from_output(output: std::process::Output) -> Self {
        let std::process::Output { status, stdout, stderr } = output;
        Self {
            exit_code: status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&stdout).into_owned(),
            stderr: String::from_utf8_lossy(&stderr).into_owned(),
        }
    }

    #[must_use]
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}
