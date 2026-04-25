#[derive(Debug, Clone)]
pub struct ContainerCommandDefinition {
    pub id: String,
    pub summary: String,
    pub command: ContainerCommandSpec,
}

#[derive(Debug, Clone)]
pub enum ContainerCommandSpec {
    Native { key: NativeContainerCommandKey },
}

#[derive(Debug, Clone, Copy)]
pub enum NativeContainerCommandKey {
    Lint,
    RegistryTools,
    EnsureImages,
    ContainerDoctor,
    ReleaseGate,
    VulnScanHook,
    ApptainerBuildAll,
    BuildApptainerAll,
    BuildApptainerHpcFrontend,
    DockerBuildAll,
    SmokeApptainer,
    SmokeDockerAmd64,
    SmokeDockerArm64,
    RunApptainerFrontendSmoke,
    RunApptainerFrontendSecurity,
    RunApptainerFrontendReproducibility,
    ContainerRuntimeCheck,
    GenerateToolIds,
    CheckToolIdManifest,
    GenerateToolNameMap,
    CheckToolNameMapGenerated,
    GenerateContainerIndex,
    CheckContainerIndex,
    GenerateGhcrPublishMatrix,
    GenerateGhcrApptainerPublishMatrix,
    GenerateLicenseMetadata,
    CheckLicenseMetadata,
    CheckLicenseIndexGenerated,
    GenerateQaMatrix,
    CheckQaMatrixGenerated,
    GenerateToolDocs,
    CheckToolDocsGenerated,
    GenerateNetworkUsage,
    CheckNetworkDisclosure,
    ExtractVersionMap,
    GenerateVersionLock,
    CheckVersionLock,
    CheckVersionAuthority,
    GenerateVersionsIndexSha,
    CheckVersionsIndexSha,
    CheckLockChangeDiscipline,
    CheckLockDrift,
    CheckLockSchema,
    CheckVersionCompleteness,
    CheckVersionHashPin,
    CheckVersionImmutability,
    CheckVersionDeprecations,
    CheckPromotionPolicy,
    CheckPromotionLockIntegrity,
    Promote,
    Demote,
    DeprecateVersion,
    ToolLifecycle,
    CheckApptainerCachePolicy,
    CheckApptainerFrontendReproducibility,
    CheckApptainerFrontendSecurity,
    CheckApptainerFrontendSmokeProof,
    CheckApptainerFrontendVersionOutputLock,
    CheckApptainerHardening,
    CheckApptainerPostPins,
    CheckApptainerVersionLabelSync,
    CheckBijuxApptainerBuilt,
    GenerateLocalApptainerDigests,
    CompareFrontendLocalSifHash,
    CheckMissingImages,
    CheckNonBijuxSources,
    CheckOwners,
    CheckRegistryVsDefs,
    CheckToolNameCollision,
    CheckToolContainerCoverage,
    CheckToolkitBundles,
    CheckHpcImageNaming,
    CheckPlannedActionability,
    CheckBijuxTemplateMarkers,
    CheckToolIdContract,
    CheckDockerArchPolicy,
    CheckDockerArm64Completeness,
    CheckDockerContext,
    CheckDockerHardening,
    CheckDockerLabels,
    CheckDockerUnpinnedApt,
    CheckDockerVersionSync,
    CheckDockerfilesBuilt,
    CheckNoSecrets,
    CheckRuntimeDownloads,
    CheckVulnAllowlist,
    CheckVulnHook,
    CheckSbomArtifacts,
    CheckTimeLocaleDeterminism,
    CheckToolInvocationNormalization,
    CheckSmokeInputsPolicy,
    CheckCrossRuntimeRepresentative,
    CheckCrossRuntimeSmoke,
    CheckSmokeFailureClassification,
    CheckSmokeContract,
    CheckSmokeContractLock,
    CheckVcfImputationToolchain,
    CheckImputationRuntimeConstraints,
    CheckImputationNetworkPolicy,
    CheckImputationHardening,
    CheckImputationReleaseSmoke,
    CheckImputationCrossRuntimeParity,
    CheckBuildProvenance,
    CheckDigestChangesOnVersionChange,
    CheckDigestOutputPolicy,
    CheckRuntimeToolDigestRecording,
    CheckRebuildRepro,
    CheckApptainerRebuildRepro,
    CheckApptainerBijuxHeader,
    CheckHpcFrontendPolicyEnforcement,
    CheckImageSizeRegression,
    CheckLockMatchesBuiltOutput,
    CheckReleaseChecklist,
    CheckToolkitBundleBuildable,
    CheckVcfDownstreamBundleCoverage,
    Summary,
    EnvPrep,
    EnvSmoke,
    ContainerSmoke,
    ContainersSmoke,
    SmokeContainersDockerArm64,
    SmokeContainersDockerAmd64,
    SmokeContainersApptainer,
    SmokeContainersApptainerBijuxRun,
    SmokeContainersApptainerApptainerRun,
    SmokeContainersApptainerVerify,
    SmokeCrossRuntimeVerify,
    SmokeToolkitDockerArm64,
    SmokeToolkitApptainer,
    BuildImages,
    BuildTool,
    BuildAll,
    BuildBundle,
    TestImages,
    TestImagesStage,
    TestImagesTool,
    ImageSmokeVcf,
    ImageQa,
    ApptainerEnsure,
    ApptainerEnsureStage,
}

#[derive(Debug, Clone)]
pub struct ContainerCommandOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ContainerCommandOutcome {
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
