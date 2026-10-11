mod apptainer;
mod behavioral;
mod contracts;
mod datasets;
mod facade;
mod fs;
mod logging;
mod records;
mod runner;
mod static_qa;
mod support;
mod validation;

pub(crate) use contracts::{QaDataset, QaStage};
pub use facade::run_image_qa;
pub use support::{
    hash_file_sha256, image_qa_base_dir, image_qa_jsonl_path, image_qa_sqlite_path,
    validate_execution_outputs,
};
pub use validation::{ensure_image_qa_passed, ensure_tool_qa_passed};
