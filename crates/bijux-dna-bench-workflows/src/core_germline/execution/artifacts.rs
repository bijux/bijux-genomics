//! Owner: bijux-dna-bench-workflows
//! Core germline execution artifacts.
use super::super::{fs, Context, Path, Result, Value};

pub(crate) fn read_json_document(path: &Path) -> Result<Value> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}
