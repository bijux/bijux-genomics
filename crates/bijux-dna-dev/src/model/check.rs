#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Primary,
    Alias,
}

#[derive(Debug, Clone)]
pub struct CheckDefinition {
    pub id: &'static str,
    pub version: u32,
    pub summary: String,
    pub aliases: &'static [&'static str],
    pub execution_mode: ExecutionMode,
    pub command: CommandSpec,
}

#[derive(Debug, Clone)]
pub enum CommandSpec {
    CargoTest { package: &'static str, test_bin: &'static str, filter: &'static str },
    Process { program: &'static str, args: &'static [&'static str] },
    Native { key: NativeCheckKey },
    Composite { members: &'static [&'static str] },
}

#[derive(Debug, Clone, Copy)]
pub enum NativeCheckKey {
    AuditAllowlist,
    DenyPolicyDeviations,
    ArtifactEnvContract,
    ArtifactsLayout,
    ArtifactsTracked,
    AssetsReferenceSchema,
    BenchKnobDisciplineDownstream,
    BenchKnobs,
    BenchmarkIntegrityPolicy,
    CargoConfigPolicy,
    CertificationSchemaDocs,
    CiAutomationSurface,
    ClippyAllowlistExpiry,
    ClippyAllowlistGrowth,
    ConfigSchema,
    DocsBuildContract,
    DocsRequirementsLock,
    ExamplesRunnerContract,
    AutomationExitCodes,
    FrontendMiniDomainValidation,
    GeneratedConfigs,
    GitignoreContract,
    HiddenTmpUsage,
    HpcSafety,
    HpcRsyncDocsParity,
    AutomationBoundary,
    LoggingContract,
    MakeHelpSync,
    AutomationNetworkUsage,
    NoFakeArtifacts,
    LegacyAutomationReferences,
    AutomationParallelism,
    NoRawCargoInMakes,
    NoRawCargoInAutomation,
    NoTargetPathsInTests,
    AutomationTempDiscipline,
    NoUserPathLiterals,
    OutputRoots,
    ReadmeLinks,
    RootLayout,
    RuntimeExecutionKernelConfig,
    RustflagsConsistency,
    AutomationArgStyle,
    AutomationDependencies,
    AutomationEntrypoints,
    AutomationHelp,
    AutomationInterface,
    AutomationWrites,
    AutomationPortability,
    SsotGuardrails,
    SpeciesAliases,
    LegacyAutomationRemoved,
    ToolRegistryLock,
    AutomationIntent,
    VcfCompatibilityMatrix,
}

#[derive(Debug, Clone)]
pub enum CheckSelection {
    All,
    Single(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    Passed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct CheckOutcome {
    pub id: String,
    pub status: CheckStatus,
    pub detail: String,
    pub children: Vec<CheckOutcome>,
}

impl CheckOutcome {
    #[must_use]
    pub fn leaf(id: &str, status: CheckStatus, detail: String) -> Self {
        Self { id: id.to_string(), status, detail, children: Vec::new() }
    }

    #[must_use]
    pub fn composite(id: &str, status: CheckStatus, children: Vec<CheckOutcome>) -> Self {
        Self { id: id.to_string(), status, detail: String::new(), children }
    }
}
