//! Owner: bijux-dna-bench-corpus
//! Complete amplicon truth-bundle admission and observed fixture comparison.
use crate::amplicon::load_amplicon_corpus_fixture_manifest_path;
use crate::amplicon::load_validated_amplicon_abundance_rows;
use crate::amplicon::load_validated_amplicon_expected_asv_rows;
use crate::amplicon::load_validated_amplicon_expected_chimera_rows;
use crate::amplicon::load_validated_amplicon_primer_rows;
use crate::amplicon::validate_amplicon_corpus_fixture_manifest_contract;
use crate::amplicon::validate_amplicon_corpus_fixture_manifest_path;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
mod model;
use model::{
    AmpliconTruthBundle, AmpliconTruthManifest, FastaSequenceTruth, NormalizedAbundanceTruthRow,
    AMPLICON_TRUTH_BUNDLE_SCHEMA_VERSION, AMPLICON_TRUTH_VALIDATION_SCHEMA_VERSION,
    OTU_ABUNDANCE_TABLE_KIND,
};
pub use model::{
    AmpliconTruthValidationReport, AMPLICON_TRUTH_FIXTURE_ID,
    AMPLICON_TRUTH_MANIFEST_SCHEMA_VERSION,
};
mod admission;

mod observations;
pub(crate) use observations::build_actual_truth_bundle;
mod io;
pub use admission::validate_amplicon_truth_manifest_path;
pub(crate) use io::{
    load_amplicon_truth_bundle, load_amplicon_truth_manifest_path, path_relative_to_repo,
    resolve_fixture_path, resolve_repo_relative_path,
};
