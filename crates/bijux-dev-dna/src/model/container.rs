#[derive(Debug, Clone)]
pub struct ContainerCommandDefinition {
    pub id: String,
    pub summary: String,
    pub command: ContainerCommandSpec,
}

#[derive(Debug, Clone)]
pub enum ContainerCommandSpec {
    Native { key: NativeContainerCommandKey },
    Script { rel_path: String },
}

#[derive(Debug, Clone, Copy)]
pub enum NativeContainerCommandKey {
    ContainerRuntimeCheck,
    GenerateToolIds,
    CheckToolIdManifest,
    GenerateToolNameMap,
    CheckToolNameMapGenerated,
    GenerateContainerIndex,
    CheckContainerIndex,
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
    Summary,
    EnvPrep,
    EnvSmoke,
    ContainerSmoke,
    ContainersSmoke,
    SmokeContainersDockerArm64,
    SmokeContainersDockerAmd64,
    SmokeContainersApptainer,
    SmokeCntainersApptainerBijuxRun,
    SmokeCntainersApptainerApptainerRun,
    SmokeCntainersApptainerVerify,
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
        Self {
            exit_code: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    #[must_use]
    pub fn failure(stderr: impl Into<String>) -> Self {
        Self {
            exit_code: 1,
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }

    #[must_use]
    pub fn from_output(output: std::process::Output) -> Self {
        Self {
            exit_code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }

    #[must_use]
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}
