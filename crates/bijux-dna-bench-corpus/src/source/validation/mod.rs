mod declaration;
mod evaluation;
mod policy;
mod values;
pub use declaration::validate_fastq_corpus_source_spec;
pub use evaluation::validate_corpus_evaluation_design;
pub use policy::validate_source_policy;
/// Invalid corpus source declarations retain their original diagnostic reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusSourceError {
    reason: String,
}
impl CorpusSourceError {
    /// Return the reason for adaptation to an existing public error taxonomy.
    #[must_use]
    pub fn into_reason(self) -> String {
        self.reason
    }
}
impl std::fmt::Display for CorpusSourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid policy: {}", self.reason)
    }
}
impl std::error::Error for CorpusSourceError {}
fn invalid(message: &str) -> CorpusSourceError {
    CorpusSourceError { reason: message.to_string() }
}
