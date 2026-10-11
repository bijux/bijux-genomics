#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn retained_registry_bindings(repo_root: &Path) -> BTreeSet<(String, String)> {
    let mut bindings = BTreeSet::new();
    for relative_path in [
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let raw = std::fs::read_to_string(repo_root.join(relative_path)).expect("read registry");
        let parsed: toml::Value = toml::from_str(&raw).expect("parse registry");
        for tool in parsed["tools"].as_array().expect("registry tools") {
            if tool["status"].as_str() == Some("unsupported") {
                continue;
            }
            let tool_id = tool["id"].as_str().expect("tool id");
            for stage_id in tool["stage_ids"].as_array().expect("tool stage ids") {
                bindings.insert((
                    stage_id.as_str().expect("stage id").to_string(),
                    tool_id.to_string(),
                ));
            }
        }
    }
    bindings
}

#[test]
fn bench_readiness_vcf_active_stage_tool_matrix_writes_governed_tsv_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-vcf-active-stage-tool-matrix"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        rendered_path.trim(),
        "artifacts/benchmark-smoke/readiness/vcf/vcf-active-stage-tool-matrix.tsv"
    );

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF active-stage-tool matrix TSV");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\ttool_id\ttool_status\tstage_support_status\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tscope_state\tscope_detail\tscope_proof_path\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    let actual_bindings = rows
        .iter()
        .map(|row| {
            let mut columns = row.split('\t');
            (
                columns.next().expect("row stage id").to_string(),
                columns.next().expect("row tool id").to_string(),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual_bindings.len(), rows.len());
    assert_eq!(actual_bindings, retained_registry_bindings(&repo_root));
    assert!(rows.iter().any(|row| {
        row == &"vcf.call\tbcftools\tproduction\tsupported\tvcf_production_regression\tbam_bundle\tvcf.adapter.calling\tvcf.parser.call_summary\tbijux.schemas.bench.vcf-normalized-metrics.call.v1\tactive\tactive\tartifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.call` / `bcftools` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().all(|row| !row.contains("\tunsupported\t")));
    assert!(rows.iter().any(|row| {
        row == &"vcf.impute\tbeagle\tproduction\tsupported\tvcf_production_regression\tvcf_cohort_with_panel\tvcf.adapter.panel_workflow\tvcf.parser.vcf_output\tbijux.schemas.bench.vcf-normalized-metrics.impute.v1\tactive\tactive\tartifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.impute` / `beagle` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.admixture\tplink2\tproduction\tsupported\tvcf_production_regression\tvcf_cohort\tvcf.adapter.population_structure\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.admixture.v1\tactive\tactive\tartifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.admixture` / `plink2` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.pca\teigensoft\tproduction\tsupported\tvcf_production_regression\tvcf_cohort\tvcf.adapter.population_structure\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.pca.v1\tactive\tactive\tartifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.pca` / `eigensoft` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.pca\tplink2\tproduction\tsupported\tvcf_production_regression\tvcf_cohort\tvcf.adapter.population_structure\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.pca.v1\tactive\tactive\tartifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.pca` / `plink2` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.imputation_metrics\tbeagle\tproduction\tsupported\tvcf_production_regression\tvcf_cohort_with_panel\tvcf.adapter.panel_workflow\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.imputation-metrics.v1\tactive\tactive\tartifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.imputation_metrics` / `beagle` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.prepare_reference_panel\tbcftools\tproduction\tsupported\tvcf_production_regression\tvcf_reference_panel\tvcf.adapter.reference_panel\tvcf.parser.vcf_output\tbijux.schemas.bench.vcf-normalized-metrics.prepare-reference-panel.v1\tactive\tactive\tartifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.prepare_reference_panel` / `bcftools` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.qc\tbcftools\tproduction\tsupported\tvcf_production_regression\tvcf_cohort\tvcf.adapter.quality_control\tvcf.parser.qc_report\tbijux.schemas.bench.vcf-normalized-metrics.qc.v1\tactive\tactive\tartifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.qc` / `bcftools` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.phasing\teagle\tplanned\tsupported\tvcf_production_regression\tvcf_cohort_with_panel\tvcf.adapter.panel_workflow\tvcf.parser.vcf_output\tbijux.schemas.bench.vcf-normalized-metrics.phasing.v1\tremoved_from_scope\tbenchmark_not_ready\tartifacts/benchmark-smoke/readiness/all-domains/no-not-benchmark-ready-rows.json\tbinding `vcf.phasing` / `eagle` is retained for a supported stage but remains outside active scope because it is not benchmark ready"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.roh\tplink2\tproduction\tsupported\tvcf_production_regression\tvcf_cohort\tvcf.adapter.roh\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.roh.v1\tactive\tactive\tartifacts/benchmark-smoke/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.roh` / `plink2` is part of the governed all-domain active benchmark matrix"
    }));
}
