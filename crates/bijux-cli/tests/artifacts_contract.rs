use std::path::{Path, PathBuf};

struct StageCase {
    stage_dir: &'static str,
    command: &'static str,
    tool: &'static str,
    retention: bool,
    paired: bool,
}

fn repo_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent().and_then(|p| p.parent()) {
        Some(root) => Ok(root.to_path_buf()),
        None => Err("repo root not found".into()),
    }
}

fn assert_file_exists(path: &Path) {
    assert!(path.exists(), "expected file to exist: {}", path.display());
}

#[test]
#[allow(clippy::too_many_lines)]
fn fastq_bench_emits_contract_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::var("BIJUX_E2E").is_err() {
        return Ok(());
    }

    let root = repo_root()?;
    let artifacts = root.join("artifacts");
    let se_r1 = root.join("tests/data/fastq/ERR769587/ERR769587.fastq.gz");
    let pe_r1 = root.join("tests/data/fastq/canonical/BIJUX_PE_R1.fastq.gz");
    let pe_r2 = root.join("tests/data/fastq/canonical/BIJUX_PE_R2.fastq.gz");

    let cases = vec![
        StageCase {
            stage_dir: "trim",
            command: "trim",
            tool: "fastp",
            retention: true,
            paired: false,
        },
        StageCase {
            stage_dir: "filter",
            command: "filter",
            tool: "fastp",
            retention: true,
            paired: false,
        },
        StageCase {
            stage_dir: "validate_pre",
            command: "validate",
            tool: "fastqvalidator_official",
            retention: false,
            paired: false,
        },
        StageCase {
            stage_dir: "merge",
            command: "merge",
            tool: "pear",
            retention: true,
            paired: true,
        },
        StageCase {
            stage_dir: "correct",
            command: "correct",
            tool: "rcorrector",
            retention: true,
            paired: true,
        },
        StageCase {
            stage_dir: "umi",
            command: "umi",
            tool: "umi_tools",
            retention: true,
            paired: true,
        },
        StageCase {
            stage_dir: "screen",
            command: "screen",
            tool: "kraken2",
            retention: false,
            paired: false,
        },
        StageCase {
            stage_dir: "qc_post",
            command: "qc-post",
            tool: "multiqc",
            retention: false,
            paired: false,
        },
        StageCase {
            stage_dir: "stats",
            command: "stats",
            tool: "seqkit",
            retention: false,
            paired: false,
        },
    ];

    for case in cases {
        let sample_id = format!("contract_{}", case.stage_dir);
        let r1 = if case.paired { &pe_r1 } else { &se_r1 };
        let mut cmd = assert_cmd::cargo_bin_cmd!("bijux");
        cmd.current_dir(&root).args([
            "bench",
            "fastq",
            case.command,
            "--sample-id",
            &sample_id,
            "--r1",
            r1.to_str().ok_or("invalid utf-8 path")?,
            "--out",
            artifacts.to_str().ok_or("invalid utf-8 path")?,
            "--tools",
            case.tool,
        ]);
        if case.paired {
            cmd.args(["--r2", pe_r2.to_str().ok_or("invalid utf-8 path")?]);
        }
        cmd.assert().success();

        let out_dir = artifacts
            .join("bench")
            .join(case.stage_dir)
            .join(&sample_id)
            .join(case.tool);
        assert_file_exists(&out_dir.join("engine_execution.json"));

        let run_artifacts = out_dir.join("run_artifacts");
        assert_file_exists(&run_artifacts.join("plan.json"));
        assert_file_exists(&run_artifacts.join("effective_config.json"));
        assert_file_exists(&run_artifacts.join("metrics_envelope.json"));
        assert_file_exists(&run_artifacts.join("stage_report.json"));
        if case.retention {
            let stage_id = match case.stage_dir {
                "stats" => "fastq.stats_neutral".to_string(),
                "qc_post" => "fastq.qc_post".to_string(),
                "validate_pre" => "fastq.validate_pre".to_string(),
                other => format!("fastq.{other}"),
            };
            assert_file_exists(
                &run_artifacts
                    .join("reports")
                    .join(format!("{stage_id}.retention.json")),
            );
        }
    }

    Ok(())
}
