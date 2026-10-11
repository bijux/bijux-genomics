use super::bindings::ExpectedBindingObservation;
/// Nonserialized canonical observations consumed by retained BAM assessment.
#[derive(Debug, Clone)]
pub struct ExpectedResultsObservation {
    pub rows: Vec<ExpectedBindingObservation>,
}
