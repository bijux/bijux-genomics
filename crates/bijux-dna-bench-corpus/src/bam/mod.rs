//! Owner: bijux-dna-bench-corpus
//! Admission of BAM corpus declarations and their local payload observations.
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_domain_bam::inspect_tiny_alignment;
use serde::{Deserialize, Serialize};

use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path};

mod admission;
pub use admission::validate_bam_corpus_fixture_manifest_path;
mod alignment;
use alignment::{parse_reference_contigs, validate_bam_corpus_fixture_sample};
mod genotyping;
use genotyping::validate_bam_corpus_genotyping_contract;
mod kinship;
use kinship::validate_bam_corpus_kinship_contract;
mod manifest;
pub use manifest::load_bam_corpus_fixture_manifest_path;
use manifest::{validate_bam_corpus_fixture_manifest_contract, validate_expected_output_contract};
mod model;
pub use model::{
    BamCorpusFixtureGenotypingContract, BamCorpusFixtureGenotypingContractReport,
    BamCorpusFixtureKinshipCase, BamCorpusFixtureKinshipCaseReport,
    BamCorpusFixtureKinshipContract, BamCorpusFixtureKinshipContractReport,
    BamCorpusFixtureManifest, BamCorpusFixtureSample, BamCorpusFixtureSampleValidationReport,
    BamCorpusFixtureValidationReport, BAM_CORPUS_FIXTURE_SCHEMA_VERSION,
    BAM_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION, DEFAULT_CORPUS_01_ADNA_BAM_MINI_MANIFEST_PATH,
    DEFAULT_CORPUS_01_BAM_MINI_MANIFEST_PATH, DEFAULT_CORPUS_01_GENOTYPING_MINI_MANIFEST_PATH,
    DEFAULT_CORPUS_01_KINSHIP_MINI_MANIFEST_PATH,
    DEFAULT_CORPUS_01_VCF_COHORT_BAM_MINI_MANIFEST_PATH,
};

pub mod assignment;
