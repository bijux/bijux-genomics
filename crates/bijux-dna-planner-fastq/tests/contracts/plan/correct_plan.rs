use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_planner_fastq::tool_adapters::fastq::correct_errors::{
    plan_correct_with_options, CorrectPlanOptions,
};
use bijux_dna_testkit::tempdir_for;
use flate2::read::GzDecoder;

fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id.to_string()),
        tool_version: "fixture".to_string(),
        image: ContainerImageRefV1 {
            image: "bijux/test:latest".to_string(),
            digest: None,
        },
        command: CommandSpecV1 {
            template: vec![tool_id.to_string(), "{{reads_r1}}".to_string()],
        },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 2,
        },
    }
}

#[test]
#[allow(non_snake_case)]
fn slow__bayeshammer_reconstruction_preserves_paired_record_count() -> Result<()> {
    let tempdir = tempdir_for("slow__bayeshammer_reconstruction_preserves_paired_record_count");
    let input_r1 = tempdir.path().join("reads_R1.fastq");
    let input_r2 = tempdir.path().join("reads_R2.fastq");
    let out_dir = tempdir.path().join("out");
    let bin_dir = tempdir.path().join("bin");
    fs::create_dir_all(&bin_dir)?;

    fs::write(
        &input_r1,
        concat!(
            "@read1/1\n",
            "AAAAAA\n",
            "+\n",
            "IIIIII\n",
            "@read2/1\n",
            "TTTTTT\n",
            "+\n",
            "IIIIII\n",
        ),
    )?;
    fs::write(
        &input_r2,
        concat!(
            "@read1/2\n",
            "CCCCCC\n",
            "+\n",
            "IIIIII\n",
            "@read2/2\n",
            "GGGGGG\n",
            "+\n",
            "IIIIII\n",
        ),
    )?;

    let fake_bayeshammer = bin_dir.join("bayeshammer");
    fs::write(
        &fake_bayeshammer,
        concat!(
            "#!/bin/sh\n",
            "set -eu\n",
            "out_dir=\n",
            "while [ \"$#\" -gt 0 ]; do\n",
            "  case \"$1\" in\n",
            "    -o) out_dir=\"$2\"; shift 2 ;;\n",
            "    -1|-2|-s|--threads|--phred-offset|-m) shift 2 ;;\n",
            "    *) shift ;;\n",
            "  esac\n",
            "done\n",
            "mkdir -p \"$out_dir/corrected\"\n",
            "cat > \"$out_dir/corrected/sample_R1.cor.fastq\" <<'EOF'\n",
            "@read1/1\n",
            "AACCAA\n",
            "+\n",
            "IIIIII\n",
            "EOF\n",
            "cat > \"$out_dir/corrected/sample_R2.cor.fastq\" <<'EOF'\n",
            "@read1/2\n",
            "CCGGCC\n",
            "+\n",
            "IIIIII\n",
            "EOF\n",
            "cat > \"$out_dir/corrected/sample_R_unpaired.cor.fastq\" <<'EOF'\n",
            "@read2/1\n",
            "TTTTAA\n",
            "+\n",
            "IIIIII\n",
            "EOF\n",
        ),
    )?;
    let mut permissions = fs::metadata(&fake_bayeshammer)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_bayeshammer, permissions)?;

    let plan = plan_correct_with_options(
        &tool("bayeshammer"),
        &input_r1,
        Some(&input_r2),
        &out_dir,
        &CorrectPlanOptions::baseline(),
    )?;

    let status = Command::new(&plan.command.template[0])
        .arg(&plan.command.template[1])
        .arg(&plan.command.template[2])
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin_dir.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .status()?;
    assert!(status.success(), "fake bayeshammer plan should succeed");

    let output_r1 = out_dir.join("reads_r1.fastq.gz");
    let output_r2 = out_dir.join("reads_r2.fastq.gz");
    assert!(output_r1.is_file(), "expected reconstructed R1 output");
    assert!(output_r2.is_file(), "expected reconstructed R2 output");

    let decoded_r1 = read_gzip_text(&output_r1)?;
    let decoded_r2 = read_gzip_text(&output_r2)?;
    assert_eq!(decoded_r1.matches('\n').count() / 4, 2);
    assert_eq!(decoded_r2.matches('\n').count() / 4, 2);
    assert!(decoded_r1.contains("@read1/1\nAACCAA\n+\nIIIIII\n"));
    assert!(decoded_r1.contains("@read2/1\nTTTTAA\n+\nIIIIII\n"));
    assert!(decoded_r2.contains("@read1/2\nCCGGCC\n+\nIIIIII\n"));
    assert!(decoded_r2.contains("@read2/2\nGGGGGG\n+\nIIIIII\n"));

    Ok(())
}

fn read_gzip_text(path: &Path) -> Result<String> {
    let mut decoded = String::new();
    let mut reader = GzDecoder::new(fs::File::open(path)?);
    reader.read_to_string(&mut decoded)?;
    Ok(decoded)
}
