use bijux_dna_core::contract::RunId;

#[must_use]
pub fn new_run_id() -> RunId {
    RunId(format!("run-{}", uuid::Uuid::new_v4()))
}
