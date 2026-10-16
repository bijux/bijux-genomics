use super::{
    fixture_id_from_manifest_path, load_manifest_schema_version, path_relative_to_repo,
    validate_fastq_duplicates_truth_manifest_path, validate_fastq_trimming_truth_manifest_path,
    validate_manifest_row_by_schema, BenchmarkFixtureRootValidationRow,
    FASTQ_DUPLICATES_TRUTH_MANIFEST_SCHEMA_VERSION, FASTQ_TRIMMING_TRUTH_MANIFEST_SCHEMA_VERSION,
};
use crate::commands::fixtures::expected::{
    fastq_duplicates::FastqDuplicatesTruthValidationReport,
    fastq_trimming::FastqTrimmingTruthValidationReport,
};
use anyhow::Result;
use std::path::Path;

/// A successful observation belongs only to this manifest in this request.
/// Failed or differently selected validators retain their independent invocation.
pub(super) enum FastqTruthObservation {
    None,
    Trimming(BenchmarkFixtureRootValidationRow),
    Duplicates(BenchmarkFixtureRootValidationRow),
}

impl FastqTruthObservation {
    pub(super) fn take_trimming_or_else(
        &mut self,
        fallback: impl FnOnce() -> BenchmarkFixtureRootValidationRow,
    ) -> BenchmarkFixtureRootValidationRow {
        match std::mem::replace(self, Self::None) {
            Self::Trimming(row) => row,
            Self::None | Self::Duplicates(_) => fallback(),
        }
    }

    pub(super) fn take_duplicates_or_else(
        &mut self,
        fallback: impl FnOnce() -> BenchmarkFixtureRootValidationRow,
    ) -> BenchmarkFixtureRootValidationRow {
        match std::mem::replace(self, Self::None) {
            Self::Duplicates(row) => row,
            Self::None | Self::Trimming(_) => fallback(),
        }
    }
}

pub(super) fn observe_manifest(
    repo_root: &Path,
    manifest_path: &Path,
) -> (BenchmarkFixtureRootValidationRow, FastqTruthObservation) {
    let schema_version = match load_manifest_schema_version(manifest_path) {
        Ok(value) => value,
        Err(error) => {
            return (
                manifest_row(repo_root, manifest_path, None, Err(error)),
                FastqTruthObservation::None,
            )
        }
    };
    match schema_version.as_str() {
        FASTQ_TRIMMING_TRUTH_MANIFEST_SCHEMA_VERSION => observe_fastq_manifest(
            repo_root,
            manifest_path,
            schema_version,
            FastqTruthObservation::Trimming,
            || {
                validate_fastq_trimming_truth_manifest_path(repo_root, manifest_path)
                    .map(|report| trimming_truth_row(repo_root, manifest_path, report))
            },
        ),
        FASTQ_DUPLICATES_TRUTH_MANIFEST_SCHEMA_VERSION => observe_fastq_manifest(
            repo_root,
            manifest_path,
            schema_version,
            FastqTruthObservation::Duplicates,
            || {
                validate_fastq_duplicates_truth_manifest_path(repo_root, manifest_path)
                    .map(|report| duplicates_truth_row(repo_root, manifest_path, report))
            },
        ),
        _ => {
            let result = validate_manifest_row_by_schema(repo_root, manifest_path, &schema_version);
            (
                manifest_row(repo_root, manifest_path, Some(schema_version), result),
                FastqTruthObservation::None,
            )
        }
    }
}

fn observe_fastq_manifest(
    repo_root: &Path,
    manifest_path: &Path,
    schema_version: String,
    observation: impl FnOnce(BenchmarkFixtureRootValidationRow) -> FastqTruthObservation,
    validate: impl FnOnce() -> Result<BenchmarkFixtureRootValidationRow>,
) -> (BenchmarkFixtureRootValidationRow, FastqTruthObservation) {
    match validate() {
        Ok(truth) => (
            manifest_row(
                repo_root,
                manifest_path,
                Some(schema_version),
                Ok(("science_fixture".to_string(), fixture_id_from_manifest_path(manifest_path))),
            ),
            observation(truth),
        ),
        Err(error) => (
            manifest_row(repo_root, manifest_path, Some(schema_version), Err(error)),
            FastqTruthObservation::None,
        ),
    }
}

fn manifest_row(
    repo_root: &Path,
    manifest_path: &Path,
    schema_version: Option<String>,
    result: Result<(String, String)>,
) -> BenchmarkFixtureRootValidationRow {
    match result {
        Ok((fixture_kind, fixture_id)) => BenchmarkFixtureRootValidationRow {
            fixture_kind,
            fixture_id,
            manifest_path: Some(path_relative_to_repo(repo_root, manifest_path)),
            detail_path: None,
            schema_version,
            valid: true,
            detail: "valid".to_string(),
        },
        Err(error) => BenchmarkFixtureRootValidationRow {
            fixture_kind: "manifest".to_string(),
            fixture_id: fixture_id_from_manifest_path(manifest_path),
            manifest_path: Some(path_relative_to_repo(repo_root, manifest_path)),
            detail_path: None,
            schema_version,
            valid: false,
            detail: error.to_string(),
        },
    }
}

pub(super) fn trimming_truth_row(
    repo_root: &Path,
    manifest_path: &Path,
    report: FastqTrimmingTruthValidationReport,
) -> BenchmarkFixtureRootValidationRow {
    BenchmarkFixtureRootValidationRow {
        fixture_kind: "expected_truth".to_string(),
        fixture_id: report.fixture_id,
        manifest_path: Some(path_relative_to_repo(repo_root, manifest_path)),
        detail_path: Some(report.expected_path),
        schema_version: Some(report.schema_version.to_string()),
        valid: report.valid,
        detail: format!("validated_cases={}", report.validated_case_count),
    }
}

pub(super) fn duplicates_truth_row(
    repo_root: &Path,
    manifest_path: &Path,
    report: FastqDuplicatesTruthValidationReport,
) -> BenchmarkFixtureRootValidationRow {
    BenchmarkFixtureRootValidationRow {
        fixture_kind: "expected_truth".to_string(),
        fixture_id: report.fixture_id,
        manifest_path: Some(path_relative_to_repo(repo_root, manifest_path)),
        detail_path: Some(report.expected_path),
        schema_version: Some(report.schema_version.to_string()),
        valid: report.valid,
        detail: format!("validated_cases={}", report.validated_case_count),
    }
}

#[cfg(test)]
mod contracts {
    use super::*;
    use std::cell::Cell;

    fn truth(valid: bool) -> BenchmarkFixtureRootValidationRow {
        BenchmarkFixtureRootValidationRow {
            fixture_kind: "expected_truth".to_string(),
            fixture_id: "declared-fixture".to_string(),
            manifest_path: Some("science/path-fixture/manifest.toml".to_string()),
            detail_path: Some("science/path-fixture/expected.json".to_string()),
            schema_version: Some("observed-schema".to_string()),
            valid,
            detail: "validated_cases=7".to_string(),
        }
    }

    #[test]
    fn successful_observation_is_consumed_once_with_distinct_report_projections() {
        let calls = Cell::new(0);
        let (manifest, mut observed) = observe_fastq_manifest(
            Path::new("/repo"),
            Path::new("/repo/science/path-fixture/manifest.toml"),
            "manifest-schema".to_string(),
            FastqTruthObservation::Trimming,
            || {
                calls.set(calls.get() + 1);
                Ok(truth(false))
            },
        );
        let row = observed.take_trimming_or_else(|| {
            calls.set(calls.get() + 1);
            truth(true)
        });
        assert_eq!(calls.get(), 1);
        assert!(manifest.valid);
        assert_eq!(manifest.fixture_id, "path-fixture");
        assert_eq!(manifest.schema_version.as_deref(), Some("manifest-schema"));
        assert_eq!(manifest.detail, "valid");
        assert_eq!(row.fixture_id, "declared-fixture");
        assert_eq!(row.schema_version.as_deref(), Some("observed-schema"));
        assert_eq!(row.detail, "validated_cases=7");
        assert!(!row.valid);
        let _ = observed.take_trimming_or_else(|| {
            calls.set(calls.get() + 1);
            truth(true)
        });
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn failed_observation_preserves_independent_named_validator_invocation() {
        let calls = Cell::new(0);
        let (manifest, mut observed) = observe_fastq_manifest(
            Path::new("/repo"),
            Path::new("/repo/science/path-fixture/manifest.toml"),
            "manifest-schema".to_string(),
            FastqTruthObservation::Trimming,
            || {
                calls.set(calls.get() + 1);
                Err(anyhow::anyhow!("first acquisition refused"))
            },
        );
        let row = observed.take_trimming_or_else(|| {
            calls.set(calls.get() + 1);
            truth(true)
        });
        assert_eq!(calls.get(), 2);
        assert!(!manifest.valid);
        assert_eq!(manifest.fixture_kind, "manifest");
        assert_eq!(manifest.detail, "first acquisition refused");
        assert_eq!(manifest.schema_version.as_deref(), Some("manifest-schema"));
        assert!(row.valid);
    }

    #[test]
    fn different_schema_selected_validator_cannot_reuse_another_family_observation() {
        let calls = Cell::new(0);
        let (_, mut observed) = observe_fastq_manifest(
            Path::new("/repo"),
            Path::new("/repo/science/fastq-trimming-truth/manifest.toml"),
            FASTQ_DUPLICATES_TRUTH_MANIFEST_SCHEMA_VERSION.to_string(),
            FastqTruthObservation::Duplicates,
            || {
                calls.set(calls.get() + 1);
                Ok(truth(false))
            },
        );
        let row = observed.take_trimming_or_else(|| {
            calls.set(calls.get() + 1);
            truth(true)
        });
        assert_eq!(calls.get(), 2);
        assert!(row.valid);
        let mut observed = FastqTruthObservation::Duplicates(truth(false));
        let row = observed.take_duplicates_or_else(|| {
            calls.set(calls.get() + 1);
            truth(true)
        });
        assert_eq!(calls.get(), 2);
        assert!(!row.valid);
    }

    #[test]
    fn schema_probe_failure_does_not_suppress_named_validation() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("manifest.toml");
        std::fs::write(&path, "schema_version = [")?;
        let (manifest, mut observed) = observe_manifest(temp.path(), &path);
        assert!(!manifest.valid);
        assert!(manifest.schema_version.is_none());
        assert_eq!(manifest.fixture_kind, "manifest");
        assert!(manifest.detail.starts_with("parse "));
        let calls = Cell::new(0);
        let row = observed.take_duplicates_or_else(|| {
            calls.set(calls.get() + 1);
            truth(true)
        });
        assert_eq!(calls.get(), 1);
        assert!(row.valid);
        Ok(())
    }
}
