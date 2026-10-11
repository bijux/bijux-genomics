//! Owner: bijux-dna-bench-model
//! Persisted local VCF smoke-suite readiness records.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::BENCHMARK_LOCAL_VCF_SMOKE_SUITE_SCHEMA_V1;

pub const BENCHMARK_LOCAL_VCF_SMOKE_OUTPUT_PATH: &str =
    "artifacts/runs/bench/local-smoke/VCF_SMOKE_SUITE_READY.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkLocalVcfSmokeGoalContract {
    pub goal_id: u32,
    pub surface: &'static str,
    pub output_path: &'static str,
}

pub const BENCHMARK_LOCAL_VCF_SMOKE_GOALS: [BenchmarkLocalVcfSmokeGoalContract; 19] = [
    goal(211, "vcf.call smoke", "artifacts/runs/bench/local-smoke/vcf.call/bcftools/calls.vcf.gz"),
    goal(
        212,
        "vcf.call_diploid smoke",
        "artifacts/runs/bench/local-smoke/vcf.call_diploid/bcftools/diploid.vcf.gz",
    ),
    goal(
        213,
        "vcf.call_pseudohaploid smoke",
        "artifacts/runs/bench/local-smoke/vcf.call_pseudohaploid/bcftools/pseudohaploid.vcf.gz",
    ),
    goal(214, "vcf.call_gl smoke", "artifacts/runs/bench/local-smoke/vcf.call_gl/bcftools/gl.vcf.gz"),
    goal(
        215,
        "vcf.damage_filter smoke",
        "artifacts/runs/bench/local-smoke/vcf.damage_filter/bcftools/damage_filter_summary.json",
    ),
    goal(
        216,
        "vcf.gl_propagation smoke",
        "artifacts/runs/bench/local-smoke/vcf.gl_propagation/bcftools/gl_propagation_report.json",
    ),
    goal(217, "vcf.filter smoke", "artifacts/runs/bench/local-smoke/vcf.filter/bcftools/filter_explain.json"),
    goal(218, "vcf.stats smoke", "artifacts/runs/bench/local-smoke/vcf.stats/bcftools/stats.json"),
    goal(219, "vcf.qc smoke", "artifacts/runs/bench/local-smoke/vcf.qc/plink2/qc.json"),
    goal(
        220,
        "vcf.prepare_reference_panel smoke",
        "artifacts/runs/bench/local-smoke/vcf.prepare_reference_panel/bcftools/panel.vcf.gz",
    ),
    goal(221, "vcf.phasing smoke", "artifacts/runs/bench/local-smoke/vcf.phasing/shapeit5/phased.vcf.gz"),
    goal(222, "vcf.impute smoke", "artifacts/runs/bench/local-smoke/vcf.impute/beagle/imputed.vcf.gz"),
    goal(
        223,
        "vcf.imputation_metrics smoke",
        "artifacts/runs/bench/local-smoke/vcf.imputation_metrics/beagle/imputation_metrics.json",
    ),
    goal(224, "vcf.pca smoke", "artifacts/runs/bench/local-smoke/vcf.pca/plink2/pca.json"),
    goal(225, "vcf.admixture smoke", "artifacts/runs/bench/local-smoke/vcf.admixture/plink2/admixture.json"),
    goal(
        226,
        "vcf.population_structure smoke",
        "artifacts/runs/bench/local-smoke/vcf.population_structure/plink2/population_structure.json",
    ),
    goal(227, "vcf.roh smoke", "artifacts/runs/bench/local-smoke/vcf.roh/plink2/roh.json"),
    goal(228, "vcf.ibd smoke", "artifacts/runs/bench/local-smoke/vcf.ibd/germline/ibd.json"),
    goal(
        229,
        "vcf.demography smoke",
        "artifacts/runs/bench/local-smoke/vcf.demography/ibdne/demography.json",
    ),
];

const fn goal(
    goal_id: u32,
    surface: &'static str,
    output_path: &'static str,
) -> BenchmarkLocalVcfSmokeGoalContract {
    BenchmarkLocalVcfSmokeGoalContract { goal_id, surface, output_path }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalVcfSmokeGoalCheck {
    pub goal_id: u32,
    pub surface: String,
    pub output_path: Option<String>,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalVcfSmokeSuiteReport {
    #[schemars(schema_with = "local_vcf_smoke_suite_schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub checked_goal_count: usize,
    pub passed_goal_count: usize,
    pub failed_goal_count: usize,
    pub failing_goal_ids: Vec<u32>,
    pub ok: bool,
    pub checks: Vec<BenchmarkLocalVcfSmokeGoalCheck>,
}

#[must_use]
pub fn benchmark_local_vcf_smoke_suite_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalVcfSmokeSuiteReport)
}

fn local_vcf_smoke_suite_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": BENCHMARK_LOCAL_VCF_SMOKE_SUITE_SCHEMA_V1})
}
