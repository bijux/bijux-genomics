#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::os::unix::fs::PermissionsExt;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> (support::RepoSandbox, std::process::Output) {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let sandbox = support::RepoSandbox::new("output-contract-audit-").expect("repo sandbox");
    let home = tempfile::tempdir().expect("tempdir");
    let unavailable_tools = tempfile::tempdir().expect("unavailable tool shims");
    for tool_id in ["plink", "plink2"] {
        let shim = unavailable_tools.path().join(tool_id);
        std::fs::write(&shim, "#!/bin/sh\nexit 127\n").expect("write unavailable tool shim");
        let mut permissions = std::fs::metadata(&shim).expect("tool shim metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&shim, permissions).expect("make tool shim executable");
    }
    let path =
        format!("{}:{}", unavailable_tools.path().display(), std::env::var("PATH").expect("PATH"));

    let output = sandbox
        .bijux_dna_command()
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .env("PATH", path)
        .args(args)
        .output()
        .expect("run cli");
    (sandbox, output)
}

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let (_sandbox, output) = run_cli(args);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_readiness_output_contract_audit_report_governs_all_retained_bindings() {
    let payload = run_cli_json(&["bench", "readiness", "render-output-contract-tests", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.output_contract_audit.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("artifacts/benchmark-smoke/readiness/tools/output-contract-tests.json")
    );
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(
        payload.get("row_count").and_then(serde_json::Value::as_u64),
        Some(rows.len() as u64)
    );
    assert_eq!(
        payload.get("passed_row_count").and_then(serde_json::Value::as_u64),
        Some(rows.len() as u64 - 139)
    );
    assert_eq!(payload.get("blocked_row_count").and_then(serde_json::Value::as_u64), Some(139));
    assert_eq!(payload.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(
        payload.get("blocked_tool_ids").and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::Value::String("adapterremoval".to_string()),
            serde_json::Value::String("addeam".to_string()),
            serde_json::Value::String("alientrimmer".to_string()),
            serde_json::Value::String("angsd".to_string()),
            serde_json::Value::String("atropos".to_string()),
            serde_json::Value::String("authenticct".to_string()),
            serde_json::Value::String("bamtools".to_string()),
            serde_json::Value::String("bamutil".to_string()),
            serde_json::Value::String("bayeshammer".to_string()),
            serde_json::Value::String("bbduk".to_string()),
            serde_json::Value::String("bbmerge".to_string()),
            serde_json::Value::String("bcftools".to_string()),
            serde_json::Value::String("beagle".to_string()),
            serde_json::Value::String("bedtools".to_string()),
            serde_json::Value::String("bijux_dna".to_string()),
            serde_json::Value::String("bowtie2".to_string()),
            serde_json::Value::String("bowtie2_build".to_string()),
            serde_json::Value::String("bwa".to_string()),
            serde_json::Value::String("centrifuge".to_string()),
            serde_json::Value::String("clumpify".to_string()),
            serde_json::Value::String("contammix".to_string()),
            serde_json::Value::String("cutadapt".to_string()),
            serde_json::Value::String("dada2".to_string()),
            serde_json::Value::String("damageprofiler".to_string()),
            serde_json::Value::String("eagle".to_string()),
            serde_json::Value::String("eigensoft".to_string()),
            serde_json::Value::String("fastp".to_string()),
            serde_json::Value::String("fastq_scan".to_string()),
            serde_json::Value::String("fastqc".to_string()),
            serde_json::Value::String("fastqvalidator".to_string()),
            serde_json::Value::String("fastuniq".to_string()),
            serde_json::Value::String("fastx_clipper".to_string()),
            serde_json::Value::String("flash2".to_string()),
            serde_json::Value::String("fqtools".to_string()),
            serde_json::Value::String("gatk".to_string()),
            serde_json::Value::String("germline".to_string()),
            serde_json::Value::String("ibdhap".to_string()),
            serde_json::Value::String("ibdne".to_string()),
            serde_json::Value::String("ibdseq".to_string()),
            serde_json::Value::String("impute5".to_string()),
            serde_json::Value::String("kaiju".to_string()),
            serde_json::Value::String("king".to_string()),
            serde_json::Value::String("kraken2".to_string()),
            serde_json::Value::String("krakenuniq".to_string()),
            serde_json::Value::String("leehom".to_string()),
            serde_json::Value::String("lighter".to_string()),
            serde_json::Value::String("mapdamage2".to_string()),
            serde_json::Value::String("mosdepth".to_string()),
            serde_json::Value::String("multiqc".to_string()),
            serde_json::Value::String("musket".to_string()),
            serde_json::Value::String("ngsbriggs".to_string()),
            serde_json::Value::String("pear".to_string()),
            serde_json::Value::String("picard".to_string()),
            serde_json::Value::String("plink".to_string()),
            serde_json::Value::String("plink2".to_string()),
            serde_json::Value::String("pmdtools".to_string()),
            serde_json::Value::String("preseq".to_string()),
            serde_json::Value::String("prinseq".to_string()),
            serde_json::Value::String("pydamage".to_string()),
            serde_json::Value::String("rcorrector".to_string()),
            serde_json::Value::String("rxy".to_string()),
            serde_json::Value::String("samtools".to_string()),
            serde_json::Value::String("schmutzi".to_string()),
            serde_json::Value::String("seqfu".to_string()),
            serde_json::Value::String("seqkit".to_string()),
            serde_json::Value::String("seqkit_stats".to_string()),
            serde_json::Value::String("seqprep".to_string()),
            serde_json::Value::String("seqtk".to_string()),
            serde_json::Value::String("shapeit".to_string()),
            serde_json::Value::String("shapeit5".to_string()),
            serde_json::Value::String("skewer".to_string()),
            serde_json::Value::String("sortmerna".to_string()),
            serde_json::Value::String("star".to_string()),
            serde_json::Value::String("trim_galore".to_string()),
            serde_json::Value::String("trimmomatic".to_string()),
            serde_json::Value::String("umi_tools".to_string()),
            serde_json::Value::String("verifybamid2".to_string()),
            serde_json::Value::String("vsearch".to_string()),
            serde_json::Value::String("yleaf".to_string()),
        ])
    );
    assert!(payload
        .get("failed_tool_ids")
        .and_then(serde_json::Value::as_array)
        .is_some_and(Vec::is_empty));

    let synthetic = rows
        .iter()
        .filter(|row| {
            matches!(
                row["output_proof_surface"].as_str(),
                Some("shared_stage_smoke" | "direct_declared_output_proof")
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        synthetic.len(),
        116,
        "retain every output-contract fixture without inventing backend execution"
    );
    for row in synthetic {
        assert_eq!(row["passed"], false);
        assert_eq!(row["independent_execution_proven"], false);
        assert_eq!(row["proof_tool_id"], serde_json::Value::Null);
        assert!(row["execution_blocker"]
            .as_str()
            .is_some_and(|reason| reason.starts_with("synthetic_output_contract_only:")));
    }
    for row in rows.iter().filter(|row| row["passed"] == true) {
        assert_eq!(row["proof_tool_id"], row["tool_id"]);
        assert_eq!(row["execution_blocker"], serde_json::Value::Null);
        assert_eq!(row["independent_execution_proven"], true);
    }

    let proof_surface_counts = payload
        .get("output_proof_surface_counts")
        .and_then(serde_json::Value::as_object)
        .expect("proof surface counts");
    let proof_count =
        proof_surface_counts.values().filter_map(serde_json::Value::as_u64).sum::<u64>();
    assert_eq!(proof_count, rows.len() as u64);
    for (tool_id, stage_id) in [
        ("ngsbriggs", "bam.damage"),
        ("angsd", "bam.kinship"),
        ("angsd", "bam.sex"),
        ("fastuniq", "fastq.remove_duplicates"),
        ("angsd", "vcf.call_gl"),
        ("angsd", "vcf.call_pseudohaploid"),
        ("impute5", "vcf.imputation_metrics"),
        ("impute5", "vcf.impute"),
    ] {
        assert!(rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                && row.get("passed").and_then(serde_json::Value::as_bool) == Some(false)
                && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                    == Some(false)
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("external_artifact_admission")
                && row
                    .get("execution_blocker")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|reason| reason.starts_with("blocked_external_access:"))
        }));
    }

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.haplogroups")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("yleaf")
            && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                == Some("direct_declared_output_proof")
            && row
                .get("observed_normalized_metric_ids")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some("haplogroups")))
    }));
    for blocked_tool_id in ["germline", "ibdne"] {
        assert!(rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(blocked_tool_id)
                && row.get("passed").and_then(serde_json::Value::as_bool) == Some(false)
                && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                    == Some(false)
                && row
                    .get("execution_blocker")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|reason| reason.contains("was not executed"))
        }));
    }
    for blocked_tool_id in ["ibdhap", "ibdseq"] {
        assert!(rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some(blocked_tool_id)
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.ibd")
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("direct_tool_smoke")
                && row.get("passed").and_then(serde_json::Value::as_bool) == Some(false)
                && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                    == Some(false)
                && row
                    .get("execution_blocker")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|reason| reason.contains("was not executed"))
        }));
    }
    for tool_id in ["eigensoft", "plink2"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.pca")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("direct_tool_smoke")
                && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                    == Some(false)
                && row
                    .get("execution_blocker")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|reason| reason.contains("tool_ok=false"))
        }));
    }
    for tool_id in ["beagle", "eagle"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("tool_contract_planning_refusal")
                && row.get("execution_blocker").and_then(serde_json::Value::as_str).is_some_and(
                    |reason| {
                        reason.starts_with("invalid_tool_contract:")
                            && reason.contains("map hsapiens_grch38_glimpse_chr22")
                            && reason.contains(&format!("not compatible with tool {tool_id}"))
                    },
                )
        }));
    }
    for tool_id in ["shapeit", "shapeit5"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.phasing")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("direct_tool_smoke")
                && row
                    .get("execution_blocker")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|reason| reason.contains("backend_execution_attempted=false"))
        }));
    }
    for stage_id in ["vcf.impute", "vcf.imputation_metrics"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("glimpse")
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("direct_tool_smoke")
                && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                    == Some(true)
                && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
        }));
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some(stage_id)
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("minimac4")
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("direct_tool_smoke")
                && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                    == Some(true)
                && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
        }));
    }
    for tool_id in ["plink", "plink2"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.admixture")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("direct_tool_smoke")
                && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                    == Some(false)
                && row
                    .get("execution_blocker")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|reason| reason.contains("tool_ok=false"))
        }));
    }
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.population_structure")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink")
            && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                == Some("direct_tool_smoke")
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(false)
            && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                == Some(false)
            && row.get("execution_blocker").and_then(serde_json::Value::as_str).is_some_and(
                |reason| {
                    reason.contains("population-structure component plan")
                        && reason.contains("MDS `plink` ok=false")
                },
            )
    }));

    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.postprocess")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
            && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                == Some("direct_tool_smoke")
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));
    assert!(rows.iter().any(|row| {
        row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
            && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.roh")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("plink2")
            && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                == Some("direct_tool_smoke")
            && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
    }));
    for tool_id in ["bcftools", "plink", "plink2"] {
        assert!(rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("vcf")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("vcf.qc")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("direct_tool_smoke")
                && row.get("independent_execution_proven").and_then(serde_json::Value::as_bool)
                    == Some(true)
                && row.get("passed").and_then(serde_json::Value::as_bool) == Some(true)
        }));
    }
}

#[test]
fn bench_readiness_output_contract_audit_write_governed_json_file() {
    let (sandbox, output) = run_cli(&["bench", "readiness", "render-output-contract-tests"]);
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
        "artifacts/benchmark-smoke/readiness/tools/output-contract-tests.json"
    );

    let report_path = sandbox.path().join(rendered_path.trim());
    assert!(report_path.is_file(), "output contract report JSON must exist");

    let payload: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&report_path).expect("read report"))
            .expect("parse report json");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.output_contract_audit.v1")
    );
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(
        payload.get("row_count").and_then(serde_json::Value::as_u64),
        Some(rows.len() as u64)
    );
    assert_eq!(payload.get("failed_row_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("blocked_row_count").and_then(serde_json::Value::as_u64), Some(139));
    assert!(
        rows.iter().any(|row| {
            row.get("tool_id").and_then(serde_json::Value::as_str) == Some("mapdamage2")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("bam.bias_mitigation")
                && row.get("output_proof_surface").and_then(serde_json::Value::as_str)
                    == Some("direct_declared_output_proof")
        }),
        "report file must retain governed BAM downstream direct-proof coverage"
    );
}
