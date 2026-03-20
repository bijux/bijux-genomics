use std::sync::{Mutex, OnceLock};

use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::{CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1};

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvGuard {
    key: &'static str,
    value: Option<String>,
}

impl EnvGuard {
    fn capture(key: &'static str) -> Self {
        Self {
            key,
            value: std::env::var(key).ok(),
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

#[test]
fn experimental_registry_alias_extends_planner_stage_selection() {
    let _lock = env_lock().lock().expect("lock env mutation tests");
    let _include_guard = EnvGuard::capture("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
    let _api_guard = EnvGuard::capture("BIJUX_EXPERIMENTAL_TOOLS");
    std::env::remove_var("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
    std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");

    let stage_id = StageId::from_static("fastq.trim_reads");
    let default_tools = bijux_dna_planner_fastq::stage_api::allowed_tools_for_stage(&stage_id);
    assert!(
        !default_tools.iter().any(|tool| tool.as_str() == "prinseq"),
        "experimental trim backend must stay out of planner defaults"
    );

    std::env::set_var("BIJUX_EXPERIMENTAL_TOOLS", "1");
    let experimental_tools = bijux_dna_planner_fastq::stage_api::allowed_tools_for_stage(&stage_id);
    assert!(
        experimental_tools.iter().any(|tool| tool.as_str() == "prinseq"),
        "planner stage selection must honor the experimental registry alias"
    );
}

#[test]
fn correct_errors_planning_honors_include_experimental_alias() {
    let _lock = env_lock().lock().expect("lock env mutation tests");
    let _include_guard = EnvGuard::capture("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
    let _api_guard = EnvGuard::capture("BIJUX_EXPERIMENTAL_TOOLS");
    std::env::remove_var("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS");
    std::env::remove_var("BIJUX_EXPERIMENTAL_TOOLS");

    let tool = ToolExecutionSpecV1 {
        tool_id: ToolId::new("musket"),
        tool_version: "99.99.99+fixture".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec!["musket".to_string()],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 1,
        },
    };

    assert!(
        bijux_dna_planner_fastq::tool_adapters::fastq::correct_errors::plan_correct(
            &tool,
            std::path::Path::new("reads_R1.fastq.gz"),
            std::path::Path::new("reads_R2.fastq.gz"),
            std::path::Path::new("out"),
        )
        .is_err(),
        "experimental corrector must stay blocked without the experimental alias"
    );

    std::env::set_var("BIJUX_INCLUDE_EXPERIMENTAL_TOOLS", "1");
    let plan = bijux_dna_planner_fastq::tool_adapters::fastq::correct_errors::plan_correct(
        &tool,
        std::path::Path::new("reads_R1.fastq.gz"),
        std::path::Path::new("reads_R2.fastq.gz"),
        std::path::Path::new("out"),
    )
    .expect("include-experimental alias must unlock planning for experimental correctors");
    assert_eq!(plan.tool_id.as_str(), "musket");
}
