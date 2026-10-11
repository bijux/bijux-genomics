//! Owner: bijux-dna-bench-corpus
//! Admission of AMPLICON corpus declarations and their local payload observations.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::fastq::{
    count_fastq_gz_reads, validate_fastq_fixture_path, FastqCorpusFixtureCompression,
};
use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path};

mod abundance;
pub use abundance::load_validated_amplicon_abundance_rows;
use abundance::validate_amplicon_abundance_table;
mod admission;
pub use admission::validate_amplicon_corpus_fixture_manifest_path;
mod manifest;
use manifest::validate_amplicon_governance_contract;
pub use manifest::{
    load_amplicon_corpus_fixture_manifest_path, validate_amplicon_corpus_fixture_manifest_contract,
};
mod model;
use model::{
    AmpliconAbundanceTableMetrics, AmpliconExpectedAsvRow, AmpliconExpectedChimeraRow,
    AmpliconExpectedPresence, AmpliconGovernanceDocument, AmpliconPrimerTableRow,
};
pub use model::{
    AmpliconAbundanceTruthRow, AmpliconCorpusAbundanceTable,
    AmpliconCorpusAbundanceTableValidationReport, AmpliconCorpusControl,
    AmpliconCorpusControlValidationReport, AmpliconCorpusFixtureManifest,
    AmpliconCorpusFixtureSample, AmpliconCorpusFixtureSampleValidationReport,
    AmpliconCorpusFixtureValidationReport, AmpliconCorpusSampleKind, AmpliconExpectedAsvTruthRow,
    AmpliconExpectedChimeraTruthRow, AmpliconPrimerTruthRow,
    AMPLICON_CORPUS_FIXTURE_SCHEMA_VERSION, AMPLICON_CORPUS_FIXTURE_VALIDATION_SCHEMA_VERSION,
    DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH,
};
mod samples;
use samples::validate_amplicon_corpus_fixture_sample;
mod truth;
use truth::{
    load_fasta_records, validate_chimera_expectations_contract, validate_expected_asvs_contract,
    validate_primer_fasta_headers, validate_primer_table_contract,
};
pub use truth::{
    load_validated_amplicon_expected_asv_rows, load_validated_amplicon_expected_chimera_rows,
    load_validated_amplicon_primer_rows,
};

mod bundle;
pub use bundle::{
    validate_amplicon_truth_manifest_path, AmpliconTruthValidationReport,
    AMPLICON_TRUTH_FIXTURE_ID, AMPLICON_TRUTH_MANIFEST_SCHEMA_VERSION,
};
