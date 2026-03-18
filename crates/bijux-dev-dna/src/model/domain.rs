#[derive(Debug, Clone)]
pub struct DomainCommandDefinition {
    pub id: String,
    pub summary: String,
    pub rel_path: String,
}

#[derive(Debug, Clone)]
pub struct DomainCommandOutcome {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl DomainCommandOutcome {
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
