use anyhow::Result;
use bijux_dna_bench_fastq::probes::{generate_correct_errors_probe, CorrectErrorsProbeInput};
use bijux_dna_stage_contract::StagePlanV1;
use std::path::Path;
fn plan(stage: &str, tool: &str) -> Result<StagePlanV1> {
    Ok(serde_json::from_value(
        serde_json::json!({"stage_id":stage,"stage_version":1,"tool_id":tool,"tool_version":"fixture","image":{"image":"fixture","digest":null},"command":{"template":[]},"resources":{"runtime":"local","mem_gb":1,"tmp_gb":1,"threads":1},"io":{"inputs":[],"outputs":[]},"out_dir":"fixture","params":{},"effective_params":{}}),
    )?)
}
#[test]
fn foreign_stage_and_unknown_correction_tool_refuse_before_creating_probe_outputs() -> Result<()> {
    let root = tempfile::tempdir()?;
    let input = root.path().join("reads.fastq");
    std::fs::write(&input, b"@read\nAC\n+\nII\n")?;
    let output = root.path().join("probes");
    for (stage, tool) in [("fastq.trim_reads", "rcorrector"), ("fastq.correct_errors", "foreign")] {
        let plan = plan(stage, tool)?;
        let request = CorrectErrorsProbeInput {
            plan: &plan,
            r1: &input,
            r2: None,
            quality_encoding: bijux_dna_domain_fastq::params::correct::QualityEncoding::Phred33,
            conservative_mode: false,
        };
        assert!(generate_correct_errors_probe(root.path(), &output, &request).is_err());
        assert!(!output.exists());
    }
    Ok(())
}
#[test]
fn malformed_fastq_refuses_before_copying_or_replacing_existing_evidence() -> Result<()> {
    let root = tempfile::tempdir()?;
    let input = root.path().join("reads.fastq");
    std::fs::write(&input, b"@read\nAC\n+\nI\n")?;
    let output = root.path().join("probes");
    let retained = output.join("error_correction/corrected_R1.fastq.gz");
    std::fs::create_dir_all(output.join("error_correction"))?;
    std::fs::write(&retained, b"retained evidence")?;
    let plan = plan("fastq.correct_errors", "rcorrector")?;
    let request = CorrectErrorsProbeInput {
        plan: &plan,
        r1: &input,
        r2: None,
        quality_encoding: bijux_dna_domain_fastq::params::correct::QualityEncoding::Phred33,
        conservative_mode: false,
    };
    assert!(generate_correct_errors_probe(root.path(), &output, &request).is_err());
    assert_eq!(std::fs::read(&retained)?, b"retained evidence");
    assert!(!output.join("error_correction/correction_report.json").exists());
    Ok(())
}
#[test]
fn synthetic_correction_publishes_the_typed_domain_report_without_claiming_changes() -> Result<()> {
    let root = tempfile::tempdir()?;
    let input = root.path().join("reads.fastq");
    std::fs::write(&input, b"@read\nAC\n+\nII\n")?;
    let plan = plan("fastq.correct_errors", "rcorrector")?;
    let request = CorrectErrorsProbeInput {
        plan: &plan,
        r1: Path::new("reads.fastq"),
        r2: None,
        quality_encoding: bijux_dna_domain_fastq::params::correct::QualityEncoding::Phred33,
        conservative_mode: false,
    };
    let proof = generate_correct_errors_probe(root.path(), &root.path().join("probes"), &request)?;
    let report = bijux_dna_bench_fastq::probes::reports::parse_report(
        &proof.proof_path,
        bijux_dna_bench_fastq::probes::reports::FastqReportKind::CorrectErrors,
    )?;
    assert_eq!(report["changed_reads"], 0);
    assert_eq!(report["unchanged_reads"], 1);
    assert_eq!(report["bases_in"], 2);
    assert_eq!(proof.observed_tool_id, "rcorrector");
    let output = root.path().join("probes/error_correction/corrected_R1.fastq.gz");
    let decoded =
        std::io::read_to_string(flate2::read::MultiGzDecoder::new(std::fs::File::open(output)?))?;
    assert_eq!(decoded, "@read\nAC\n+\nII\n");
    Ok(())
}
