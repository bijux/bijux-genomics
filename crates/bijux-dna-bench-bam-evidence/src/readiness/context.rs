use super::ports::{DeclarationEvidence, LocalProofWriter};
use std::path::Path;
/// Canonical addresses retained in published readiness evidence.
pub struct ReadinessPaths<'a> {
    pub serving: &'a str,
    pub commands: &'a str,
    pub outputs: &'a str,
    pub parsers: &'a str,
    pub expected: &'a str,
    pub report_map: &'a str,
    pub schema: &'a str,
}
/// A readiness request binds all observations and effects to explicit authorities.
pub struct ReadinessContext<'a> {
    pub repo_root: &'a Path,
    pub declarations: &'a dyn DeclarationEvidence,
    pub native: &'a dyn LocalProofWriter,
    pub paths: ReadinessPaths<'a>,
}
