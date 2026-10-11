//! Owner: bijux-dna-bench-corpus
//! Admission of DAMAGE corpus declarations and their local payload observations.
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path};

mod admission;
pub use admission::validate_bam_damage_fixture_manifest_path;
mod alignment;
use alignment::{
    ensure_fixture_file, ensure_reference_fasta, parse_reference_contigs, parse_tiny_sam_header,
};
mod manifest;
use manifest::{
    load_bam_damage_fixture_expectation_path, load_bam_damage_fixture_manifest_path,
    validate_bam_damage_fixture_expectation_contract,
    validate_bam_damage_fixture_manifest_contract,
};
mod model;
pub use model::{
    BamDamageFixtureExpectation, BamDamageFixtureManifest, BamDamageFixtureValidationReport,
    BAM_DAMAGE_FIXTURE_SCHEMA_VERSION, BAM_DAMAGE_FIXTURE_VALIDATION_SCHEMA_VERSION,
    DEFAULT_CORPUS_01_ADNA_DAMAGE_MANIFEST_PATH,
};
use model::{TinySamHeaderSummary, BAM_DAMAGE_FIXTURE_EXPECTATION_SCHEMA_VERSION};
