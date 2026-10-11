//! Owner: bijux-dna-bench-model
//! Persisted readiness evidence for the governed essential-pipeline portfolio.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::BENCHMARK_ESSENTIAL_PIPELINE_READINESS_SCHEMA_V1;

pub const BENCHMARK_ESSENTIAL_PIPELINE_READINESS_OUTPUT_PATH: &str =
    "artifacts/benchmark-smoke/readiness/pipelines/ESSENTIAL_PIPELINES_READY.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkEssentialPipelineReadinessGoalContract {
    pub goal_id: u32,
    pub surface: &'static str,
    pub output_path: &'static str,
}

pub const BENCHMARK_ESSENTIAL_PIPELINE_READINESS_GOALS:
    [BenchmarkEssentialPipelineReadinessGoalContract; 16] = [
    pipeline_goal(
        261,
        "core-germline-fastq-bam-vcf",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/core-germline-fastq-bam-vcf.json",
    ),
    pipeline_goal(
        262,
        "adna-pseudohaploid-fastq-bam-vcf",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/adna-pseudohaploid-fastq-bam-vcf.json",
    ),
    pipeline_goal(
        263,
        "adna-gl-fastq-bam-vcf",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/adna-gl-fastq-bam-vcf.json",
    ),
    pipeline_goal(
        264,
        "diploid-small-fastq-bam-vcf",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/diploid-small-fastq-bam-vcf.json",
    ),
    pipeline_goal(
        265,
        "reference-panel-imputation",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/reference-panel-imputation.json",
    ),
    pipeline_goal(
        266,
        "popgen-structure-vcf",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/popgen-structure-vcf.json",
    ),
    pipeline_goal(
        267,
        "relatedness-segments-vcf",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/relatedness-segments-vcf.json",
    ),
    pipeline_goal(
        268,
        "bam-genotyping-to-vcf-downstream",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/bam-genotyping-to-vcf-downstream.json",
    ),
    pipeline_goal(
        269,
        "edna-taxonomy-no-vcf",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/edna-taxonomy-no-vcf.json",
    ),
    pipeline_goal(
        270,
        "amplicon-asv-otu-no-vcf",
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/amplicon-asv-otu-no-vcf.json",
    ),
    goal(
        271,
        "essential pipeline corpus/assets",
        "artifacts/benchmark-smoke/readiness/pipelines/essential-pipeline-corpus-assets.tsv",
    ),
    goal(
        272,
        "essential pipeline commands",
        "artifacts/benchmark-smoke/readiness/pipelines/essential-pipelines-rendered-commands.sh",
    ),
    goal(
        273,
        "essential pipeline fake-runner",
        "artifacts/benchmark-smoke/simulated-runs/pipelines/essential",
    ),
    goal(
        274,
        "essential pipeline partial resume",
        "artifacts/benchmark-smoke/readiness/pipelines/essential-pipeline-partial-resume.json",
    ),
    goal(
        275,
        "essential pipeline failure isolation",
        "artifacts/benchmark-smoke/readiness/pipelines/essential-pipeline-failure-isolation.json",
    ),
    goal(
        276,
        "essential pipeline report map",
        "artifacts/benchmark-smoke/readiness/pipelines/essential-pipeline-report-map.tsv",
    ),
];

const fn pipeline_goal(
    goal_id: u32,
    pipeline_id: &'static str,
    output_path: &'static str,
) -> BenchmarkEssentialPipelineReadinessGoalContract {
    BenchmarkEssentialPipelineReadinessGoalContract { goal_id, surface: pipeline_id, output_path }
}

const fn goal(
    goal_id: u32,
    surface: &'static str,
    output_path: &'static str,
) -> BenchmarkEssentialPipelineReadinessGoalContract {
    BenchmarkEssentialPipelineReadinessGoalContract { goal_id, surface, output_path }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkEssentialPipelineReadinessGoalCheck {
    pub goal_id: u32,
    pub surface: String,
    pub output_path: Option<String>,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkEssentialPipelineReadinessReport {
    #[schemars(schema_with = "essential_pipeline_readiness_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub checked_goal_count: usize,
    pub passed_goal_count: usize,
    pub failed_goal_count: usize,
    pub failing_goal_ids: Vec<u32>,
    pub pipeline_count: usize,
    pub dag_node_count: usize,
    pub corpus_asset_row_count: usize,
    pub rendered_command_row_count: usize,
    pub fake_run_node_count: usize,
    pub fake_run_output_count: usize,
    pub report_map_row_count: usize,
    pub ok: bool,
    pub checks: Vec<BenchmarkEssentialPipelineReadinessGoalCheck>,
}

#[must_use]
pub fn benchmark_essential_pipeline_readiness_schema() -> Schema {
    schemars::schema_for!(BenchmarkEssentialPipelineReadinessReport)
}

fn essential_pipeline_readiness_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": BENCHMARK_ESSENTIAL_PIPELINE_READINESS_SCHEMA_V1})
}
