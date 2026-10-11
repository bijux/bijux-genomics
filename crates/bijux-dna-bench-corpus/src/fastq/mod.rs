//! Owner: bijux-dna-bench-corpus
//! Admission of FASTQ corpus declarations and their local payload observations.
use std::collections::BTreeSet;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use flate2::read::MultiGzDecoder;
use serde::{Deserialize, Serialize};

use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path};

mod admission;
pub use admission::validate_fastq_corpus_fixture_manifest_path;
mod manifest;
pub use manifest::load_fastq_corpus_fixture_manifest_path;
use manifest::validate_fastq_corpus_fixture_manifest_contract;
mod model;
pub use model::{
    FastqCorpusFixtureCompression, FastqCorpusFixtureLayout, FastqCorpusFixtureManifest,
    FastqCorpusFixtureSample, FastqCorpusFixtureSampleValidationReport,
    FastqCorpusFixtureValidationReport, DEFAULT_CORPUS_01_MINI_MANIFEST_PATH,
    DEFAULT_CORPUS_01_VCF_COHORT_FASTQ_MINI_MANIFEST_PATH, FASTQ_CORPUS_FIXTURE_SCHEMA_VERSION,
    FASTQ_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
};
mod reads;
use reads::validate_fastq_corpus_fixture_sample;
pub(crate) use reads::{count_fastq_gz_reads, validate_fastq_fixture_path};
