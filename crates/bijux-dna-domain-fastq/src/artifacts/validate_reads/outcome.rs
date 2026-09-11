use super::{ValidateFailureClass, ValidationReportV1};
use crate::params::validate::ValidationMode;
impl ValidationReportV1 {
    /// Whether failure classification, strict acceptance and exit policy agree.
    /// Report-only mode can retain a scientific failure while returning exit code zero.
    #[must_use]
    pub fn has_consistent_outcome(&self) -> bool {
        let no_failure = self.failure_class == ValidateFailureClass::None;
        let expected_pass = self.status_r1 == 0
            && self.status_r2 == 0
            && no_failure
            && self.pair_sync_pass.unwrap_or(true)
            && self.pair_count_match.unwrap_or(true);
        let exits_successfully =
            self.strict_pass || self.validation_mode == ValidationMode::ReportOnly;
        if self.status_r1 < 0
            || self.status_r2 < 0
            || self.exit_code < 0
            || self.strict_pass != expected_pass
            || no_failure != self.strict_pass
            || (self.exit_code == 0) != exits_successfully
        {
            return false;
        }
        match self.failure_class {
            ValidateFailureClass::PairCountMismatch => self.pair_count_match == Some(false),
            ValidateFailureClass::HeaderSyncMismatch => {
                self.pair_sync_checked
                    && self.pair_sync_pass == Some(false)
                    && self.pair_count_match != Some(false)
            }
            ValidateFailureClass::EmptyInput => {
                self.validated_reads_r1 == 0 || self.validated_reads_r2 == Some(0)
            }
            ValidateFailureClass::None => {
                self.validated_reads_r1 > 0 && self.validated_reads_r2 != Some(0)
            }
            ValidateFailureClass::UnsupportedCompression
            | ValidateFailureClass::MalformedRecord
            | ValidateFailureClass::InvalidQualityEncoding
            | ValidateFailureClass::ValidatorError => true,
        }
    }
}
