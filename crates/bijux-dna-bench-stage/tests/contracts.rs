#[path = "contracts/local_stage_commands.rs"]
mod local_stage_commands;
#[path = "contracts/local_stage_inventory.rs"]
mod local_stage_inventory;

#[cfg(feature = "execution")]
#[path = "execution/commands.rs"]
mod execution_commands;
#[cfg(feature = "repository")]
#[path = "repository/admission.rs"]
mod repository_admission;
