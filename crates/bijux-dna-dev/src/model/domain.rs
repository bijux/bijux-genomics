#[derive(Debug, Clone)]
pub struct DomainCommandDefinition {
    pub id: String,
    pub summary: String,
    pub command: DomainCommandSpec,
}

#[derive(Debug, Clone)]
pub enum DomainCommandSpec {
    Native { key: NativeDomainCommandKey },
}

#[derive(Debug, Clone, Copy)]
pub enum NativeDomainCommandKey {
    CheckDefaultSettingsDocs,
    CheckDocLinks,
    CheckDomainIndex,
    CheckDomainLayout,
    CheckDomainSchema,
    CheckDomainToolMetadata,
    CheckExternalToolPolicy,
    CheckFixtureContracts,
    CheckInventory,
    CheckOrphanFiles,
    CheckPlannerFixtureCoverage,
    CheckPlannerStageCoverage,
    CheckReferenceBundleLock,
    CheckRustStageCatalogParity,
    CheckSharedTools,
    CheckSsotAuthority,
    CheckToolContainerParity,
    GenerateIndex,
    GenerateInventory,
    InventoryDrift,
    LockRegistry,
    Validate,
}

#[derive(Debug, Clone)]
pub struct DomainCommandOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl DomainCommandOutcome {
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
        let std::process::Output {
            status,
            stdout,
            stderr,
        } = output;
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
