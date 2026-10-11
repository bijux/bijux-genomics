//! Owner: bijux-dna-bench-workflows
//! amplicon benchmark workflow composition.
use crate::execution::paths::path_relative_to_repo;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_corpus::amplicon::load_amplicon_corpus_fixture_manifest_path;
use bijux_dna_bench_corpus::amplicon::load_validated_amplicon_abundance_rows;
use bijux_dna_bench_corpus::amplicon::load_validated_amplicon_expected_asv_rows;
use bijux_dna_bench_corpus::amplicon::load_validated_amplicon_expected_chimera_rows;
use bijux_dna_bench_corpus::amplicon::load_validated_amplicon_primer_rows;
use bijux_dna_bench_corpus::amplicon::validate_amplicon_corpus_fixture_manifest_path;
use bijux_dna_bench_corpus::amplicon::validate_amplicon_truth_manifest_path;
use bijux_dna_bench_corpus::amplicon::AmpliconAbundanceTruthRow;
use bijux_dna_bench_corpus::amplicon::AmpliconCorpusFixtureManifest;
use bijux_dna_bench_corpus::amplicon::AmpliconExpectedChimeraTruthRow;
use bijux_dna_bench_corpus::amplicon::AmpliconPrimerTruthRow;
use bijux_dna_bench_corpus::amplicon::DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH;
use flate2::read::MultiGzDecoder;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
mod model;
use model::{
    AmpliconCorpusStageArtifacts, AmpliconOutputJudgmentEvidence, AmpliconTruthBundle,
    AmpliconTruthStageArtifacts, ClusterOtusStageArtifacts, ClusterOtusStageEvidence, FastaTruth,
    InferAsvsStageArtifacts, InferAsvsStageEvidence, NormalizeAbundanceStageArtifacts,
    NormalizeAbundanceStageEvidence, NormalizePrimersEvidence, NormalizePrimersLocalSmokeReport,
    NormalizePrimersStageArtifacts, NormalizedAbundanceTruthRow, OtuTableRow,
    RemoveChimerasStageArtifacts, RemoveChimerasStageEvidence, AMPLICON_MICRO_PIPELINE_COMMAND,
    AMPLICON_MICRO_PIPELINE_ID, AMPLICON_MICRO_PIPELINE_SCHEMA_VERSION,
    CLUSTER_OTUS_STAGE_SCHEMA_VERSION, DEFAULT_AMPLICON_ASV_REPRESENTATIVES_PATH,
    DEFAULT_AMPLICON_CHIMERA_CONTROL_FASTQ_PATH, DEFAULT_AMPLICON_CLUSTER_OTUS_FASTQ_PATH,
    DEFAULT_AMPLICON_INFER_ASVS_FASTQ_PATH, DEFAULT_AMPLICON_NON_CHIMERIC_REPRESENTATIVES_PATH,
    DEFAULT_AMPLICON_NORMALIZED_ABUNDANCE_PATH, DEFAULT_AMPLICON_OTU_REPRESENTATIVES_PATH,
    DEFAULT_AMPLICON_SINGLE_END_NORMALIZED_FASTQ_PATH, DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH,
    DEFAULT_AMPLICON_TRUTH_MANIFEST_PATH, GOVERNED_MICRO_ELAPSED_SECONDS,
    GOVERNED_MICRO_FINISHED_AT, GOVERNED_MICRO_STARTED_AT, INFER_ASVS_STAGE_SCHEMA_VERSION,
    NORMALIZE_ABUNDANCE_STAGE_SCHEMA_VERSION, NORMALIZE_PRIMERS_STAGE_SCHEMA_VERSION,
    OTU_ABUNDANCE_TABLE_KIND, REMOVE_CHIMERAS_STAGE_SCHEMA_VERSION,
};
pub use model::{
    AmpliconMicroPipelineHandoff, AmpliconMicroPipelineReport, AmpliconMicroPipelineRow,
    AmpliconMicroPipelineRowStatus, DEFAULT_AMPLICON_MICRO_PIPELINE_PATH,
};

mod workflow;
pub use workflow::render_amplicon_micro_pipeline;
mod handoffs;
pub(crate) use handoffs::{build_handoffs, row};
mod stages;
use stages::{
    run_amplicon_corpus_fixture_stage, run_amplicon_output_judgment_stage,
    run_amplicon_truth_fixture_stage, run_cluster_otus_stage, run_infer_asvs_stage,
    run_normalize_abundance_stage, run_normalize_primers_stage, run_remove_chimeras_stage,
};
mod observations;
use observations::{
    build_three_column_table, count_gz_fastq_reads, load_abundance_rows, load_fasta_records,
    load_normalized_abundance_rows, load_otu_table_rows, load_tsv_rows, passes_behavior_test,
    validate_asv_table, validate_chimera_table, validate_otu_table, validate_primer_case,
    write_fastq_records,
};
mod execution;
use execution::{
    copy_absolute_file, copy_repo_relative_file, copy_repo_relative_file_with_absolute_fallback,
    is_nonempty_file, load_json, stage_root, timestamp_marker, write_json_file, write_text_file,
};
