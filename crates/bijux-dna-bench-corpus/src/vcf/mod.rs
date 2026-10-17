//! Owner: bijux-dna-bench-corpus
//! Admission of VCF corpus declarations and their local payload observations.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_domain_vcf::execute_vcf_validation;
use serde::{Deserialize, Serialize};

use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path};

mod admission;
pub use admission::validate_vcf_corpus_fixture_manifest_path;
mod manifest;
pub use manifest::{
    load_vcf_corpus_fixture_manifest_path, validate_vcf_corpus_fixture_manifest_contract,
};
mod model;
use model::{FastaIndexRow, ReferenceContig, ReferenceDictRow, VcfDocumentSummary, VcfVariantRole};
pub use model::{
    SampleMetadataRow, VcfCorpusFixtureManifest, VcfCorpusFixtureValidationReport,
    VcfVariantSetValidationReport, DEFAULT_VCF_MINI_MANIFEST_PATH,
    VCF_CORPUS_FIXTURE_SCHEMA_VERSION, VCF_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
};
mod reference;
use reference::{load_validated_reference_contigs, validate_target_sites_bed};
mod samples;
pub use samples::load_sample_metadata;
use samples::{
    load_population_metadata, validate_manifest_sample_coverage,
    validate_sample_metadata_population_links,
};
mod variants;
use variants::validate_variant_set;

mod filter_truth;
pub use filter_truth::{
    validate_vcf_filter_truth_manifest_path, VcfFilterTruthValidationReport,
    VCF_FILTER_TRUTH_FIXTURE_ID, VCF_FILTER_TRUTH_MANIFEST_SCHEMA_VERSION,
};

mod genotype_truth;
pub use genotype_truth::{
    validate_vcf_genotype_truth_manifest_path, VcfGenotypeTruthValidationReport,
    VCF_GENOTYPE_TRUTH_FIXTURE_ID, VCF_GENOTYPE_TRUTH_MANIFEST_SCHEMA_VERSION,
};

mod phasing_imputation_truth;
pub use phasing_imputation_truth::{
    validate_phasing_imputation_truth_manifest_path, PhasingImputationTruthValidationReport,
    PHASING_IMPUTATION_TRUTH_FIXTURE_ID, PHASING_IMPUTATION_TRUTH_MANIFEST_SCHEMA_VERSION,
};

mod population_structure_truth;
pub use population_structure_truth::{
    validate_population_structure_truth_manifest_path, PopulationStructureTruthValidationReport,
    POPULATION_STRUCTURE_TRUTH_FIXTURE_ID, POPULATION_STRUCTURE_TRUTH_MANIFEST_SCHEMA_VERSION,
};

mod segments_demography_truth;
pub use segments_demography_truth::{
    validate_segments_demography_truth_manifest_path, SegmentsDemographyTruthValidationReport,
    SEGMENTS_DEMOGRAPHY_TRUTH_FIXTURE_ID, SEGMENTS_DEMOGRAPHY_TRUTH_MANIFEST_SCHEMA_VERSION,
};
