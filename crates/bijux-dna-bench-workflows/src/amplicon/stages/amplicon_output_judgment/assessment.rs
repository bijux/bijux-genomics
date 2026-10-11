//! Owner: bijux-dna-bench-workflows
//! Compare typed stage observations with admitted amplicon truth.
use super::super::super::{
    anyhow, load_abundance_rows, load_amplicon_corpus_fixture_manifest_path, load_fasta_records,
    load_json, load_normalized_abundance_rows, load_otu_table_rows, load_tsv_rows,
    load_validated_amplicon_abundance_rows, load_validated_amplicon_expected_asv_rows,
    load_validated_amplicon_expected_chimera_rows, load_validated_amplicon_primer_rows,
    validate_asv_table, validate_chimera_table, validate_otu_table, validate_primer_case,
    AmpliconMicroPipelineRow, AmpliconOutputJudgmentEvidence, AmpliconTruthBundle,
    ClusterOtusStageEvidence, InferAsvsStageEvidence, NormalizeAbundanceStageEvidence,
    NormalizePrimersEvidence, Path, RemoveChimerasStageEvidence, Result,
    DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH, OTU_ABUNDANCE_TABLE_KIND,
};

pub(super) fn assess_outputs(
    repo_root: &Path,
    stages: &super::super::super::execution::AmpliconStageArtifacts,
) -> Result<AmpliconOutputJudgmentEvidence> {
    let corpus_manifest_path = &stages.corpus.manifest_path;
    let manifest = load_amplicon_corpus_fixture_manifest_path(corpus_manifest_path)?;
    let primer_rows =
        load_validated_amplicon_primer_rows(repo_root, corpus_manifest_path, &manifest)?;
    let expected_asv_rows =
        load_validated_amplicon_expected_asv_rows(repo_root, corpus_manifest_path, &manifest)?;
    let expected_chimera_rows =
        load_validated_amplicon_expected_chimera_rows(repo_root, corpus_manifest_path, &manifest)?;
    let expected_abundance_rows = load_validated_amplicon_abundance_rows(
        repo_root,
        corpus_manifest_path,
        &manifest,
        OTU_ABUNDANCE_TABLE_KIND,
    )?;
    let truth_bundle: AmpliconTruthBundle =
        load_json(&repo_root.join(DEFAULT_AMPLICON_TRUTH_EXPECTED_PATH))?;

    let normalize_report: NormalizePrimersEvidence =
        load_stage_evidence(repo_root, &stages.normalize.row)?;
    let infer_report: InferAsvsStageEvidence = load_stage_evidence(repo_root, &stages.infer.row)?;
    let chimera_report: RemoveChimerasStageEvidence =
        load_stage_evidence(repo_root, &stages.chimera.row)?;
    let otu_report: ClusterOtusStageEvidence = load_stage_evidence(repo_root, &stages.otu.row)?;
    let abundance_report: NormalizeAbundanceStageEvidence =
        load_stage_evidence(repo_root, &stages.abundance.row)?;

    let primer_case_valid = validate_primer_case(
        repo_root,
        &manifest,
        &primer_rows,
        &normalize_report,
        &stages.normalize.normalized_reads_r1,
    )?;
    let RepresentativeJudgment {
        table_valid: asv_table_valid,
        representatives_valid: asv_representatives_valid,
    } = assess_asv(stages, &truth_bundle, &infer_report)?;
    let RepresentativeJudgment {
        table_valid: chimera_table_valid,
        representatives_valid: non_chimeric_representatives_valid,
    } = assess_chimera(stages, &truth_bundle, &chimera_report, &expected_chimera_rows)?;
    let RepresentativeJudgment {
        table_valid: otu_table_valid,
        representatives_valid: otu_representatives_valid,
    } = assess_otu(repo_root, stages, &truth_bundle, &otu_report)?;
    let otu_abundance_rows = load_abundance_rows(&stages.abundance.otu_abundance_table_tsv)?;
    let otu_abundance_table_valid = otu_abundance_rows == expected_abundance_rows
        && expected_abundance_rows == truth_bundle.otu_abundances;
    let normalized_abundance_rows =
        load_normalized_abundance_rows(&stages.abundance.normalized_abundance_tsv)?;
    let normalized_abundance_valid = normalized_abundance_rows
        == truth_bundle.normalized_abundances
        && abundance_report.numeric_output_valid
        && abundance_report.sample_totals.iter().all(|(_, total)| (*total - 1.0).abs() < 1e-9);

    let valid = primer_case_valid
        && asv_table_valid
        && asv_representatives_valid
        && chimera_table_valid
        && non_chimeric_representatives_valid
        && otu_table_valid
        && otu_representatives_valid
        && otu_abundance_table_valid
        && normalized_abundance_valid;

    Ok(AmpliconOutputJudgmentEvidence {
        schema_version: "bijux.bench.local_amplicon_output_judgment.v1",
        stage_id: "benchmark.amplicon_output_judgment".to_string(),
        valid,
        checked_section_count: 9,
        checked_row_count: primer_rows.len()
            + expected_asv_rows.len()
            + expected_chimera_rows.len()
            + truth_bundle.asv_representatives.len()
            + truth_bundle.non_chimeric_representatives.len()
            + truth_bundle.otu_representatives.len()
            + expected_abundance_rows.len()
            + truth_bundle.normalized_abundances.len(),
        primer_case_valid,
        asv_table_valid,
        asv_representatives_valid,
        chimera_table_valid,
        non_chimeric_representatives_valid,
        otu_table_valid,
        otu_representatives_valid,
        otu_abundance_table_valid,
        normalized_abundance_valid,
    })
}

fn load_stage_evidence<T: serde::de::DeserializeOwned>(
    repo_root: &Path,
    row: &AmpliconMicroPipelineRow,
) -> Result<T> {
    let path = row
        .evidence_path
        .as_deref()
        .ok_or_else(|| anyhow!("{} row is missing evidence path", row.stage_id))?;
    load_json(&repo_root.join(path))
}

struct RepresentativeJudgment {
    table_valid: bool,
    representatives_valid: bool,
}

fn assess_asv(
    stages: &super::super::super::execution::AmpliconStageArtifacts,
    truth_bundle: &AmpliconTruthBundle,
    infer_report: &InferAsvsStageEvidence,
) -> Result<RepresentativeJudgment> {
    let asv_representatives = load_fasta_records(&stages.infer.representatives_fasta)?;
    let asv_representatives_valid = asv_representatives == truth_bundle.asv_representatives
        && usize::try_from(infer_report.representative_sequence_count)
            .ok()
            .is_some_and(|count| count == truth_bundle.asv_representatives.len());
    let asv_table_rows = load_tsv_rows(&stages.infer.asv_table_tsv)?;
    let asv_table_valid = validate_asv_table(&asv_table_rows, &truth_bundle.asv_representatives)
        && usize::try_from(infer_report.asv_count)
            .ok()
            .is_some_and(|count| count == truth_bundle.asv_representatives.len())
        && infer_report.sample_count == 1;

    Ok(RepresentativeJudgment {
        table_valid: asv_table_valid,
        representatives_valid: asv_representatives_valid,
    })
}

fn assess_chimera(
    stages: &super::super::super::execution::AmpliconStageArtifacts,
    truth_bundle: &AmpliconTruthBundle,
    chimera_report: &RemoveChimerasStageEvidence,
    expected_chimera_rows: &[bijux_dna_bench_corpus::amplicon::AmpliconExpectedChimeraTruthRow],
) -> Result<RepresentativeJudgment> {
    let chimera_table_rows = load_tsv_rows(&stages.chimera.chimeras_tsv)?;
    let chimera_table_valid = validate_chimera_table(&chimera_table_rows, expected_chimera_rows)
        && usize::try_from(chimera_report.chimera_count).ok().is_some_and(|count| {
            count
                == expected_chimera_rows
                    .iter()
                    .filter(|row| row.expected_presence == "present")
                    .count()
        });
    let non_chimeric_representatives = load_fasta_records(&stages.chimera.non_chimeric_fasta)?;
    let non_chimeric_representatives_valid = non_chimeric_representatives
        == truth_bundle.non_chimeric_representatives
        && usize::try_from(chimera_report.non_chimera_count)
            .ok()
            .is_some_and(|count| count == truth_bundle.non_chimeric_representatives.len());

    Ok(RepresentativeJudgment {
        table_valid: chimera_table_valid,
        representatives_valid: non_chimeric_representatives_valid,
    })
}

fn assess_otu(
    repo_root: &Path,
    stages: &super::super::super::execution::AmpliconStageArtifacts,
    truth_bundle: &AmpliconTruthBundle,
    otu_report: &ClusterOtusStageEvidence,
) -> Result<RepresentativeJudgment> {
    let otu_table_rows = load_otu_table_rows(&stages.otu.otu_table_tsv)?;
    let otu_table_valid =
        validate_otu_table(repo_root, &otu_table_rows, &truth_bundle.otu_representatives)
            && usize::try_from(otu_report.otu_count)
                .ok()
                .is_some_and(|count| count == truth_bundle.otu_representatives.len())
            && otu_report.sample_count == 1;
    let otu_representatives = load_fasta_records(&stages.otu.otu_representatives_fasta)?;
    let otu_representatives_valid = otu_representatives == truth_bundle.otu_representatives
        && usize::try_from(otu_report.representative_sequence_count)
            .ok()
            .is_some_and(|count| count == truth_bundle.otu_representatives.len());

    Ok(RepresentativeJudgment {
        table_valid: otu_table_valid,
        representatives_valid: otu_representatives_valid,
    })
}
