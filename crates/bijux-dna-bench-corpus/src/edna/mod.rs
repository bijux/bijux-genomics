//! Owner: bijux-dna-bench-corpus
//! Admission of EDNA corpus declarations and their local payload observations.
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::fastq::{
    count_fastq_gz_reads, validate_fastq_fixture_path, FastqCorpusFixtureCompression,
};
use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path};

mod admission;
pub use admission::validate_edna_corpus_fixture_manifest_path;
mod manifest;
pub use manifest::{
    load_edna_corpus_fixture_manifest_path, validate_edna_corpus_fixture_manifest_contract,
};
mod model;
pub use model::{
    EdnaCorpusFixtureManifest, EdnaCorpusFixtureSample, EdnaCorpusFixtureSampleValidationReport,
    EdnaCorpusFixtureValidationReport, EdnaExpectedPresence, EdnaExpectedTaxon,
    EdnaExpectedTaxonRow, DEFAULT_CORPUS_02_EDNA_MANIFEST_PATH, EDNA_CORPUS_FIXTURE_SCHEMA_VERSION,
    EDNA_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
};
mod reads;
use reads::validate_edna_corpus_fixture_sample;
mod taxonomy;
pub use taxonomy::{load_validated_edna_expected_taxa_rows, resolve_edna_expected_taxa_path};
