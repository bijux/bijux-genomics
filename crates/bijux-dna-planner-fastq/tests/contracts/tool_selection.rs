use std::sync::{Mutex, OnceLock};

use bijux_dna_core::ids::StageId;

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
