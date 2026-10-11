//! Owner: bijux-dna-bench-workflows
//! Amplicon observations truth.
use super::super::{
    anyhow, bail, count_gz_fastq_reads, fs, AmpliconAbundanceTruthRow,
    AmpliconCorpusFixtureManifest, AmpliconExpectedChimeraTruthRow, AmpliconMicroPipelineHandoff,
    AmpliconMicroPipelineRow, AmpliconMicroPipelineRowStatus, AmpliconPrimerTruthRow, BTreeSet,
    BufRead, BufReader, Context, FastaTruth, NormalizePrimersEvidence, NormalizedAbundanceTruthRow,
    OtuTableRow, Path, Result, Value,
};
use bijux_dna_core::id_catalog as stage;

pub(crate) fn passes_behavior_test(
    rows: &[AmpliconMicroPipelineRow],
    handoffs: &[AmpliconMicroPipelineHandoff],
) -> bool {
    let expected_stage_ids = BTreeSet::from([
        "benchmark.amplicon_corpus_fixture",
        "benchmark.amplicon_truth_fixture",
        stage::FASTQ_NORMALIZE_PRIMERS,
        stage::FASTQ_INFER_ASVS,
        stage::FASTQ_REMOVE_CHIMERAS,
        stage::FASTQ_CLUSTER_OTUS,
        stage::FASTQ_NORMALIZE_ABUNDANCE,
        "benchmark.amplicon_output_judgment",
    ]);
    let observed_stage_ids = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    let no_bam_or_vcf = rows.iter().all(|row| row.domain != "bam" && row.domain != "vcf");
    let all_succeeded =
        rows.iter().all(|row| row.status == AmpliconMicroPipelineRowStatus::Succeeded);
    let all_handoffs_accepted = handoffs.iter().all(|handoff| handoff.accepted);
    let judgment_valid = rows.iter().any(|row| {
        row.stage_id == "benchmark.amplicon_output_judgment"
            && row.metrics.get("valid").and_then(Value::as_bool) == Some(true)
    });
    observed_stage_ids == expected_stage_ids
        && no_bam_or_vcf
        && all_succeeded
        && all_handoffs_accepted
        && judgment_valid
}

pub(crate) fn validate_primer_case(
    repo_root: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    primer_rows: &[AmpliconPrimerTruthRow],
    report: &NormalizePrimersEvidence,
    normalized_reads_r1: &Path,
) -> Result<bool> {
    let primer_row = primer_rows
        .iter()
        .find(|row| row.primer_id == manifest.primer_set_id)
        .ok_or_else(|| {
            anyhow!("amplicon primer truth rows are missing primer set {}", manifest.primer_set_id)
        })?;
    let observed_read_count = count_gz_fastq_reads(normalized_reads_r1)?;
    Ok(report.primer_set_id == manifest.primer_set_id
        && report.marker_id == manifest.marker_id
        && report.orientation_policy == primer_row.orientation
        && report.input_reads == report.matched_reads + report.unmatched_reads
        && report.output_reads == observed_read_count
        && repo_root.join(&report.report_json).is_file()
        && repo_root.join(&report.primer_orientation_report).is_file()
        && repo_root.join(&report.primer_stats_json).is_file())
}

pub(crate) fn validate_asv_table(rows: &[Vec<String>], representatives: &[FastaTruth]) -> bool {
    if rows.is_empty() {
        return false;
    }
    let expected_ids = representatives.iter().map(|row| row.id.as_str()).collect::<BTreeSet<_>>();
    let observed_ids =
        rows.iter().filter_map(|row| row.get(1)).map(String::as_str).collect::<BTreeSet<_>>();
    expected_ids == observed_ids && rows.iter().all(|row| row.len() == 3 && row[2] != "0")
}

pub(crate) fn validate_chimera_table(
    rows: &[Vec<String>],
    expected_chimeras: &[AmpliconExpectedChimeraTruthRow],
) -> bool {
    let expected_ids = expected_chimeras
        .iter()
        .filter(|row| row.expected_presence == "present")
        .map(|row| row.chimera_id.as_str())
        .collect::<BTreeSet<_>>();
    let observed_ids =
        rows.iter().filter_map(|row| row.first()).map(String::as_str).collect::<BTreeSet<_>>();
    expected_ids == observed_ids
}

pub(crate) fn validate_otu_table(
    repo_root: &Path,
    rows: &[OtuTableRow],
    representatives: &[FastaTruth],
) -> bool {
    let expected_ids = representatives.iter().map(|row| row.id.as_str()).collect::<BTreeSet<_>>();
    let observed_ids = rows.iter().map(|row| row.otu_id.as_str()).collect::<BTreeSet<_>>();
    expected_ids == observed_ids
        && rows.len() == representatives.len()
        && rows.iter().all(|row| repo_root.join(&row.representative_fasta).is_file())
}

pub(crate) fn build_three_column_table(
    header: &str,
    rows: impl IntoIterator<Item = String>,
) -> String {
    let mut rendered = String::from(header);
    for row in rows {
        rendered.push_str(&row);
    }
    rendered
}

pub(crate) fn load_fasta_records(path: &Path) -> Result<Vec<FastaTruth>> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut current_id = None::<String>;
    let mut current_sequence = String::new();
    for line in reader.lines() {
        let line = line.with_context(|| format!("read {}", path.display()))?;
        if let Some(rest) = line.strip_prefix('>') {
            if let Some(id) = current_id.take() {
                records.push(FastaTruth { id, sequence: current_sequence.clone() });
                current_sequence.clear();
            }
            current_id = Some(rest.to_string());
        } else if !line.trim().is_empty() {
            current_sequence.push_str(line.trim());
        }
    }
    if let Some(id) = current_id {
        records.push(FastaTruth { id, sequence: current_sequence });
    }
    if records.is_empty() {
        bail!("fasta is empty: {}", path.display());
    }
    Ok(records)
}

pub(crate) fn load_tsv_rows(path: &Path) -> Result<Vec<Vec<String>>> {
    let file = fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("read {}", path.display()))?;
        if index == 0 {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        rows.push(line.split('\t').map(std::string::ToString::to_string).collect());
    }
    Ok(rows)
}

pub(crate) fn load_otu_table_rows(path: &Path) -> Result<Vec<OtuTableRow>> {
    let rows = load_tsv_rows(path)?;
    rows.into_iter()
        .map(|row| {
            if row.len() != 5 {
                bail!("otu table row has unexpected width in {}", path.display());
            }
            Ok(OtuTableRow {
                sample_id: row[0].clone(),
                otu_id: row[1].clone(),
                abundance: row[2].clone(),
                representative_id: row[3].clone(),
                representative_fasta: row[4].clone(),
            })
        })
        .collect()
}

pub(crate) fn load_abundance_rows(path: &Path) -> Result<Vec<AmpliconAbundanceTruthRow>> {
    let rows = load_tsv_rows(path)?;
    rows.into_iter()
        .map(|row| {
            if row.len() != 3 {
                bail!("abundance table row has unexpected width in {}", path.display());
            }
            Ok(AmpliconAbundanceTruthRow {
                sample_id: row[0].clone(),
                feature_id: row[1].clone(),
                abundance: row[2]
                    .parse::<u64>()
                    .with_context(|| format!("parse abundance in {}", path.display()))?,
            })
        })
        .collect()
}

pub(crate) fn load_normalized_abundance_rows(
    path: &Path,
) -> Result<Vec<NormalizedAbundanceTruthRow>> {
    let rows = load_tsv_rows(path)?;
    rows.into_iter()
        .map(|row| {
            if row.len() != 3 {
                bail!("normalized abundance row has unexpected width in {}", path.display());
            }
            Ok(NormalizedAbundanceTruthRow {
                sample_id: row[0].clone(),
                feature_id: row[1].clone(),
                normalized_abundance: row[2].clone(),
            })
        })
        .collect()
}
