use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_domain_bam::{
    align_fastq_to_bam_bowtie2_style, align_fastq_to_bam_bwa_style, params::ReadGroupSpec,
};

const BAM_ALIGNMENT_FIXTURE_SCHEMA_VERSION: &str = "bijux.fixture.bam_alignment_metrics.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AlignmentFixtureBackend {
    Bwa,
    Bowtie2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BamAlignmentFixtureCase {
    tool_id: &'static str,
    backend: AlignmentFixtureBackend,
    sample_id: &'static str,
    reference_file: &'static str,
    reads_r1_file: &'static str,
    reads_r2_file: Option<&'static str>,
    expected_file: &'static str,
}

const BAM_ALIGNMENT_FIXTURE_CASES: &[BamAlignmentFixtureCase] = &[
    BamAlignmentFixtureCase {
        tool_id: "bwa",
        backend: AlignmentFixtureBackend::Bwa,
        sample_id: "sample1",
        reference_file: "reference.fa",
        reads_r1_file: "reads_R1.fastq",
        reads_r2_file: Some("reads_R2.fastq"),
        expected_file: "expected.align-metrics.json",
    },
    BamAlignmentFixtureCase {
        tool_id: "bowtie2",
        backend: AlignmentFixtureBackend::Bowtie2,
        sample_id: "sample2",
        reference_file: "reference.fa",
        reads_r1_file: "reads_R1.fastq",
        reads_r2_file: None,
        expected_file: "expected.align-metrics.json",
    },
];

#[test]
fn bam_alignment_fixture_bank_covers_governed_aligner_cases() {
    assert_eq!(BAM_ALIGNMENT_FIXTURE_CASES.len(), 2);
    for case in BAM_ALIGNMENT_FIXTURE_CASES {
        let dir = fixture_dir(case);
        assert!(dir.exists(), "missing fixture directory: {}", dir.display());
        assert!(
            dir.join(case.reference_file).exists(),
            "missing reference fixture `{}` for {}",
            case.reference_file,
            case.tool_id
        );
        assert!(
            dir.join(case.reads_r1_file).exists(),
            "missing R1 fixture `{}` for {}",
            case.reads_r1_file,
            case.tool_id
        );
        if let Some(reads_r2_file) = case.reads_r2_file {
            assert!(
                dir.join(reads_r2_file).exists(),
                "missing R2 fixture `{}` for {}",
                reads_r2_file,
                case.tool_id
            );
        }
        assert!(
            dir.join(case.expected_file).exists(),
            "missing expected fixture `{}` for {}",
            case.expected_file,
            case.tool_id
        );
    }
}

#[test]
fn bam_alignment_fixture_bank_matches_expected_normalized_json() -> Result<()> {
    for case in BAM_ALIGNMENT_FIXTURE_CASES {
        let expected = read_expected_json(case)?;
        let observed = render_case(case)?;
        assert_json_matches(&observed, &expected, case.tool_id);
    }
    Ok(())
}

fn render_case(case: &BamAlignmentFixtureCase) -> Result<serde_json::Value> {
    let dir = fixture_dir(case);
    let reference = dir.join(case.reference_file);
    let reads_r1 = dir.join(case.reads_r1_file);
    let reads_r2 = case.reads_r2_file.map(|path| dir.join(path));
    let output_root = repo_root().join("artifacts/bench-readiness/bam-align-fixtures").join(case.tool_id);
    std::fs::create_dir_all(&output_root)
        .with_context(|| format!("create {}", output_root.display()))?;
    let read_group = ReadGroupSpec::with_defaults(case.sample_id);

    let (provenance, summary) = match case.backend {
        AlignmentFixtureBackend::Bwa => align_fastq_to_bam_bwa_style(
            &reference,
            &reads_r1,
            reads_r2.as_deref(),
            &output_root,
            case.sample_id,
            &read_group,
            Some("default"),
            Some(12),
        )?,
        AlignmentFixtureBackend::Bowtie2 => align_fastq_to_bam_bowtie2_style(
            &reference,
            &reads_r1,
            None,
            &output_root,
            case.sample_id,
            &read_group,
            Some("very_sensitive_local"),
        )?,
    };

    let mut output_names = provenance
        .outputs
        .outputs
        .iter()
        .map(|entry| entry.name.clone())
        .collect::<Vec<_>>();
    output_names.sort();

    Ok(serde_json::json!({
        "schema_version": BAM_ALIGNMENT_FIXTURE_SCHEMA_VERSION,
        "stage_id": "bam.align",
        "tool_id": case.tool_id,
        "normalized": {
            "provenance": {
                "backend_tool_id": provenance.backend_tool_id,
                "strategy_id": provenance.strategy_id,
                "preset": provenance.preset,
                "mode": provenance.mode,
                "sensitivity_profile": provenance.sensitivity_profile,
                "seed_length": provenance.seed_length,
                "sample_id": provenance.sample_identity.sample_id,
                "read_group_id": provenance.read_group.id,
                "read_group_sample": provenance.read_group.sample,
                "output_names": output_names,
            },
            "mapping_summary": {
                "total_reads": summary.flagstat.total_reads,
                "mapped_reads": summary.flagstat.mapped_reads,
                "duplicate_reads": summary.flagstat.duplicate_reads,
                "mapped_fraction": summary.flagstat.mapped_fraction,
                "proper_pair_reads": summary.proper_pair_reads,
                "secondary_reads": summary.secondary_reads,
                "supplementary_reads": summary.supplementary_reads,
                "mapq_status": summary.mapq_regime.as_ref().map(|value| value.status.clone()),
            }
        }
    }))
}

fn read_expected_json(case: &BamAlignmentFixtureCase) -> Result<serde_json::Value> {
    let path = fixture_dir(case).join(case.expected_file);
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read expected fixture {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse expected fixture {}", path.display()))
}

fn fixture_dir(case: &BamAlignmentFixtureCase) -> PathBuf {
    repo_root().join("tests/fixtures/bench/parsers/bam/bam.align").join(case.tool_id)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonicalize repo root")
}

fn assert_json_matches(observed: &serde_json::Value, expected: &serde_json::Value, tool_id: &str) {
    assert!(
        observed == expected,
        "BAM alignment fixture mismatch for {tool_id}\nobserved: {observed:#}\nexpected: {expected:#}"
    );
}
