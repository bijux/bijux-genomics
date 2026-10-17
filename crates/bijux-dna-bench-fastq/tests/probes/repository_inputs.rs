use anyhow::Result;
use bijux_dna_bench_fastq::probes::{
    generate_deplete_host_probe, generate_deplete_rrna_probe, generate_index_reference_probe,
    generate_screen_taxonomy_probe,
    reports::{parse_report, FastqReportKind},
    MaterializedFastqProof,
};
use bijux_dna_stage_contract::StagePlanV1;
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};

type Probe = fn(&Path, &Path, &StagePlanV1) -> Result<MaterializedFastqProof>;

fn plan(stage: &str, tool: &str, inputs: &[Value]) -> Result<StagePlanV1> {
    Ok(serde_json::from_value(json!({
        "stage_id":stage,"stage_version":1,"tool_id":tool,"tool_version":"fixture",
        "image":{"image":"fixture","digest":null},"command":{"template":[]},
        "resources":{"runtime":"local","mem_gb":1,"tmp_gb":1,"threads":1},
        "io":{"inputs":inputs,"outputs":[]},"out_dir":"fixture","params":{},
        "effective_params":{
            "schema_version":bijux_dna_domain_fastq::params::quality::screen::SCREEN_TAXONOMY_SCHEMA_VERSION,"paired_mode":"paired_end",
            "threads":1,"database_catalog_id":"fixture","database_artifact_id":"fixture",
            "database_scope":"read_screening","classifier":"kraken2","report_format":"kraken_report",
            "assignment_format":"kraken_assignments","emit_unclassified":true,
            "interpretation_boundary":"screening_only"
        }
    }))?)
}

fn input(name: &str, path: &Path, role: &str) -> Value {
    json!({"name":name,"path":path,"role":role,"optional":false})
}

#[test]
fn reference_probe_resolves_relative_inputs_and_preserves_absolute_input_semantics() -> Result<()> {
    let root = tempfile::tempdir()?;
    let external = tempfile::tempdir()?;
    let cwd = std::env::current_dir()?;
    fs::write(root.path().join("reference.fa"), b">one\nACGT\n")?;
    fs::write(external.path().join("reference.fa"), b">external\nACGTACGT\n")?;
    let output = root.path().join("probes");
    let mut relative_report = None;
    for reference in [PathBuf::from("reference.fa"), root.path().join("reference.fa")] {
        let plan = plan(
            "fastq.index_reference",
            "bowtie2_build",
            &[input("reference_fasta", &reference, "reference")],
        )?;
        let proof = generate_index_reference_probe(root.path(), &output, &plan)?;
        let report = parse_report(&proof.proof_path, FastqReportKind::IndexReference)?;
        assert_eq!(report["reference_fasta"], "reference.fa");
        assert_eq!(report["reference_bytes"], 10);
        if let Some(previous) = &relative_report {
            assert_eq!(&report, previous);
        }
        relative_report = Some(report);
    }
    let absolute = external.path().join("reference.fa");
    let plan = plan(
        "fastq.index_reference",
        "bowtie2_build",
        &[input("reference_fasta", &absolute, "reference")],
    )?;
    let proof = generate_index_reference_probe(root.path(), &output, &plan)?;
    let report = parse_report(&proof.proof_path, FastqReportKind::IndexReference)?;
    assert_eq!(report["reference_fasta"], absolute.to_string_lossy().as_ref());
    assert_eq!(report["reference_bytes"], 19);
    assert_eq!(std::env::current_dir()?, cwd);
    Ok(())
}

fn paired_inputs(stage: &str, tool: &str, probe: Probe, kind: FastqReportKind) -> Result<()> {
    let root = tempfile::tempdir()?;
    let cwd = std::env::current_dir()?;
    fs::write(root.path().join("left.fastq"), b"@read/1\nAC\n+\nII\n")?;
    fs::write(root.path().join("right.fastq"), b"@read/2\nGT\n+\nII\n")?;
    let output = root.path().join("probes");
    let mut relative_report = None;
    for absolute in [false, true] {
        let path = |name| if absolute { root.path().join(name) } else { PathBuf::from(name) };
        let plan = plan(
            stage,
            tool,
            &[
                input("reads_r1", &path("left.fastq"), "reads"),
                input("reads_r2", &path("right.fastq"), "reads"),
            ],
        )?;
        let proof = probe(root.path(), &output, &plan)?;
        let report = parse_report(&proof.proof_path, kind)?;
        assert_eq!(report["input_r1"], "left.fastq");
        assert_eq!(report["input_r2"], "right.fastq");
        assert_eq!(report["reads_in"], 2);
        assert_eq!(report["bases_in"], 4);
        if let Some(previous) = &relative_report {
            assert_eq!(&report, previous);
        }
        relative_report = Some(report);
    }
    let plan = plan(
        stage,
        tool,
        &[
            input("reads_r1", Path::new("left.fastq"), "reads"),
            input("reads_r2", Path::new("missing.fastq"), "reads"),
        ],
    )?;
    let absent = root.path().join("refused");
    assert!(probe(root.path(), &absent, &plan).is_err());
    assert!(!absent.exists());
    assert_eq!(std::env::current_dir()?, cwd);
    Ok(())
}

#[test]
fn host_probe_resolves_both_reads_before_publication() -> Result<()> {
    paired_inputs(
        "fastq.deplete_host",
        "bowtie2",
        generate_deplete_host_probe,
        FastqReportKind::DepleteHost,
    )
}
#[test]
fn rrna_probe_resolves_both_reads_before_publication() -> Result<()> {
    paired_inputs(
        "fastq.deplete_rrna",
        "sortmerna",
        generate_deplete_rrna_probe,
        FastqReportKind::DepleteRrna,
    )
}
#[test]
fn taxonomy_probe_resolves_both_reads_before_publication() -> Result<()> {
    paired_inputs(
        "fastq.screen_taxonomy",
        "kraken2",
        generate_screen_taxonomy_probe,
        FastqReportKind::ScreenTaxonomy,
    )
}
