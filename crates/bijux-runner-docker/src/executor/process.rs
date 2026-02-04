use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ExecutionAssessment {
    pub success: bool,
    pub missing_outputs: Vec<PathBuf>,
    pub reason: Option<String>,
}

#[must_use]
pub fn assess_execution(exit_code: i32, expected_outputs: &[PathBuf]) -> ExecutionAssessment {
    if exit_code != 0 {
        return ExecutionAssessment {
            success: false,
            missing_outputs: Vec::new(),
            reason: Some(format!("exit_code={exit_code}")),
        };
    }
    let missing: Vec<PathBuf> = expected_outputs
        .iter()
        .filter(|path| !path.exists())
        .cloned()
        .collect();
    if !missing.is_empty() {
        return ExecutionAssessment {
            success: false,
            missing_outputs: missing,
            reason: Some("missing_outputs".to_string()),
        };
    }
    ExecutionAssessment {
        success: true,
        missing_outputs: Vec::new(),
        reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::assess_execution;
    use std::path::PathBuf;

    #[test]
    fn assess_execution_success() -> anyhow::Result<()> {
        let dir = bijux_infra::temp_dir("bijux")?;
        let output = dir.path().join("out.fastq");
        bijux_infra::atomic_write_bytes(&output, b"ok")?;
        let assessment = assess_execution(0, &[output]);
        assert!(assessment.success);
        Ok(())
    }

    #[test]
    fn assess_execution_missing_outputs() {
        let missing = PathBuf::from("/tmp/missing.fastq");
        let assessment = assess_execution(0, &[missing]);
        assert!(!assessment.success);
        assert_eq!(assessment.reason.as_deref(), Some("missing_outputs"));
    }

    #[test]
    fn assess_execution_partial_outputs() -> anyhow::Result<()> {
        let dir = bijux_infra::temp_dir("bijux")?;
        let present = dir.path().join("present.fastq");
        bijux_infra::atomic_write_bytes(&present, b"ok")?;
        let missing = dir.path().join("missing.fastq");
        let assessment = assess_execution(0, &[present, missing]);
        assert!(!assessment.success);
        assert_eq!(assessment.reason.as_deref(), Some("missing_outputs"));
        Ok(())
    }

    #[test]
    fn assess_execution_bad_exit_code() {
        let assessment = assess_execution(1, &[]);
        assert!(!assessment.success);
        assert_eq!(assessment.reason.as_deref(), Some("exit_code=1"));
    }
}
