//! Admission invariants for stage-tool benchmark asset contracts.

use std::collections::{BTreeMap, BTreeSet};

use bijux_dna_core::id_catalog::{
    BAM_CONTAMINATION, BAM_GENOTYPING, BAM_HAPLOGROUPS, BAM_KINSHIP, BAM_RECALIBRATION, BAM_SEX,
    FASTQ_DEPLETE_HOST, FASTQ_DEPLETE_REFERENCE_CONTAMINANTS, FASTQ_DEPLETE_RRNA,
    FASTQ_INDEX_REFERENCE, FASTQ_SCREEN,
};

use crate::{
    BenchError, StageToolAssetRow, StageToolAssetsConfig, StageToolAssetsReport,
    LOCAL_STAGE_TOOL_ASSETS_SCHEMA_V1, STAGE_TOOL_ASSETS_DECLARATION_ORIGIN,
    STAGE_TOOL_ASSETS_REPORT_SCHEMA_V1, STAGE_TOOL_ASSETS_SCOPE,
};

/// Validates a persisted stage-tool asset configuration.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] for invalid identity, paths, ordering, or semantics.
pub fn validate_stage_tool_assets_config(config: &StageToolAssetsConfig) -> Result<(), BenchError> {
    if config.schema_version != LOCAL_STAGE_TOOL_ASSETS_SCHEMA_V1
        || config.classification_scope != STAGE_TOOL_ASSETS_SCOPE
    {
        return invalid("stage-tool asset config identity is invalid");
    }
    validate_rows(&config.rows)
}

/// Validates a persisted stage-tool asset readiness report.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] for invalid identity, paths, rows, or aggregates.
pub fn validate_stage_tool_assets_report(report: &StageToolAssetsReport) -> Result<(), BenchError> {
    if report.schema_version != STAGE_TOOL_ASSETS_REPORT_SCHEMA_V1
        || report.classification_scope != STAGE_TOOL_ASSETS_SCOPE
        || !safe_relative_path(&report.config_path)
    {
        return invalid("stage-tool asset report identity or path is invalid");
    }
    validate_rows(&report.rows)?;
    let stage_tools = report
        .rows
        .iter()
        .map(|row| (row.domain.as_str(), row.stage_id.as_str(), row.tool_id.as_str()))
        .collect::<BTreeSet<_>>();
    let asset_ids = report.rows.iter().map(|row| row.asset_id.as_str()).collect::<BTreeSet<_>>();
    if report.row_count != report.rows.len()
        || report.declared_stage_tool_row_count != stage_tools.len()
        || report.asset_id_row_count != report.rows.len()
        || report.unique_asset_id_count != asset_ids.len()
        || report.domain_counts != count_by(&report.rows, |row| row.domain.as_str())
        || report.asset_role_counts != count_by(&report.rows, |row| row.asset_role.as_str())
    {
        return invalid("stage-tool asset report aggregates are inconsistent");
    }
    Ok(())
}

/// Validates the required asset identities and paths for governed FASTQ/BAM benchmark commands.
///
/// This checks already-acquired declaration rows and performs no repository or filesystem IO.
/// Generic configuration/report admission remains a separate operation.
///
/// # Errors
/// Returns [`BenchError::InvalidObservation`] for missing required coverage or rebound assets.
pub fn validate_governed_stage_tool_asset_coverage(
    rows: &[StageToolAssetRow],
) -> Result<(), BenchError> {
    validate_taxonomy_database_roots(rows)?;
    for tool_id in TAXONOMY_TOOL_IDS {
        require_asset_binding(
            rows,
            &RequiredAssetBinding {
                stage_id: FASTQ_SCREEN,
                tool_id,
                asset_role: "database_artifact_id",
                asset_id: "taxonomy_db",
                asset_path: "assets/reference/taxonomy/references/mock_community_taxonomy",
                label: "FASTQ taxonomy",
            },
        )?;
    }
    for required in FASTQ_REFERENCE_ASSETS.iter().chain(BAM_REFERENCE_ASSETS) {
        require_asset_binding(rows, required)?;
    }
    Ok(())
}

const TAXONOMY_TOOL_IDS: [&str; 4] = ["centrifuge", "kaiju", "kraken2", "krakenuniq"];

struct RequiredAssetBinding {
    stage_id: &'static str,
    tool_id: &'static str,
    asset_role: &'static str,
    asset_id: &'static str,
    asset_path: &'static str,
    label: &'static str,
}

const FASTQ_REFERENCE_ASSETS: &[RequiredAssetBinding] = &[
    RequiredAssetBinding {
        stage_id: FASTQ_DEPLETE_RRNA,
        tool_id: "sortmerna",
        asset_role: "rrna_reference",
        asset_id: "sortmerna_common_rrna_reference",
        asset_path: "assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta",
        label: "FASTQ rRNA depletion",
    },
    RequiredAssetBinding {
        stage_id: FASTQ_DEPLETE_RRNA,
        tool_id: "sortmerna",
        asset_role: "database_artifact_id",
        asset_id: "sortmerna_common_rrna_reference",
        asset_path: "assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta",
        label: "FASTQ rRNA depletion",
    },
    RequiredAssetBinding {
        stage_id: FASTQ_DEPLETE_HOST,
        tool_id: "bowtie2",
        asset_role: "reference_catalog_id",
        asset_id: "host_reference",
        asset_path: "assets/reference/host/references/toy_host_reference",
        label: "FASTQ host depletion",
    },
    RequiredAssetBinding {
        stage_id: FASTQ_DEPLETE_HOST,
        tool_id: "bowtie2",
        asset_role: "reference_index_artifact_id",
        asset_id: "reference_index",
        asset_path: "assets/reference/host/references/toy_host_reference",
        label: "FASTQ host depletion",
    },
    RequiredAssetBinding {
        stage_id: FASTQ_DEPLETE_REFERENCE_CONTAMINANTS,
        tool_id: "bowtie2",
        asset_role: "reference_catalog_id",
        asset_id: "contaminant_reference",
        asset_path: "assets/reference/contaminants/references/toy_contaminant_reference",
        label: "FASTQ contaminant depletion",
    },
    RequiredAssetBinding {
        stage_id: FASTQ_DEPLETE_REFERENCE_CONTAMINANTS,
        tool_id: "bowtie2",
        asset_role: "reference_index_artifact_id",
        asset_id: "reference_index",
        asset_path: "assets/reference/contaminants/references/toy_contaminant_reference",
        label: "FASTQ contaminant depletion",
    },
    RequiredAssetBinding {
        stage_id: FASTQ_INDEX_REFERENCE,
        tool_id: "bowtie2_build",
        asset_role: "reference_fasta",
        asset_id: "phix174",
        asset_path: "assets/reference/contaminants/references/phix174.fasta",
        label: "FASTQ reference indexing",
    },
    RequiredAssetBinding {
        stage_id: FASTQ_INDEX_REFERENCE,
        tool_id: "bowtie2_build",
        asset_role: "reference_index_output",
        asset_id: "reference_index",
        asset_path: "artifacts/benchmark-smoke/readiness/local-ready/fastq.index_reference/reference_index/bowtie2/reference",
        label: "FASTQ reference indexing",
    },
    RequiredAssetBinding {
        stage_id: FASTQ_INDEX_REFERENCE,
        tool_id: "star",
        asset_role: "reference_fasta",
        asset_id: "phix174",
        asset_path: "assets/reference/contaminants/references/phix174.fasta",
        label: "FASTQ reference indexing",
    },
    RequiredAssetBinding {
        stage_id: FASTQ_INDEX_REFERENCE,
        tool_id: "star",
        asset_role: "reference_index_output",
        asset_id: "reference_index",
        asset_path: "artifacts/benchmark-smoke/readiness/local-ready/fastq.index_reference/reference_index/star",
        label: "FASTQ reference indexing",
    },
];

const BAM_REFERENCE_ASSETS: &[RequiredAssetBinding] = &[
    RequiredAssetBinding {
        stage_id: BAM_CONTAMINATION,
        tool_id: "contammix",
        asset_role: "reference_fasta",
        asset_id: "adna_bam_reference",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta",
        label: "BAM contamination",
    },
    RequiredAssetBinding {
        stage_id: BAM_CONTAMINATION,
        tool_id: "contammix",
        asset_role: "reference_panel",
        asset_id: "adna_contamination_panel",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat",
        label: "BAM contamination",
    },
    RequiredAssetBinding {
        stage_id: BAM_CONTAMINATION,
        tool_id: "schmutzi",
        asset_role: "reference_fasta",
        asset_id: "adna_bam_reference",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta",
        label: "BAM contamination",
    },
    RequiredAssetBinding {
        stage_id: BAM_CONTAMINATION,
        tool_id: "schmutzi",
        asset_role: "reference_panel",
        asset_id: "adna_contamination_panel",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat",
        label: "BAM contamination",
    },
    RequiredAssetBinding {
        stage_id: BAM_CONTAMINATION,
        tool_id: "verifybamid2",
        asset_role: "reference_fasta",
        asset_id: "adna_bam_reference",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta",
        label: "BAM contamination",
    },
    RequiredAssetBinding {
        stage_id: BAM_CONTAMINATION,
        tool_id: "verifybamid2",
        asset_role: "reference_panel",
        asset_id: "adna_contamination_panel",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat",
        label: "BAM contamination",
    },
    RequiredAssetBinding {
        stage_id: BAM_SEX,
        tool_id: "rxy",
        asset_role: "reference_fasta",
        asset_id: "adna_bam_reference",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta",
        label: "BAM sex",
    },
    RequiredAssetBinding {
        stage_id: BAM_SEX,
        tool_id: "yleaf",
        asset_role: "reference_fasta",
        asset_id: "adna_bam_reference",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta",
        label: "BAM sex",
    },
    RequiredAssetBinding {
        stage_id: BAM_HAPLOGROUPS,
        tool_id: "yleaf",
        asset_role: "reference_fasta",
        asset_id: "adna_bam_reference",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta",
        label: "BAM haplogroups",
    },
    RequiredAssetBinding {
        stage_id: BAM_HAPLOGROUPS,
        tool_id: "yleaf",
        asset_role: "reference_panel",
        asset_id: "adna-y-hg38-mini",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_y_haplogroup_panel.tsv",
        label: "BAM haplogroups",
    },
    RequiredAssetBinding {
        stage_id: BAM_GENOTYPING,
        tool_id: "bcftools",
        asset_role: "reference_fasta",
        asset_id: "corpus_01_bam_reference",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta",
        label: "BAM genotyping",
    },
    RequiredAssetBinding {
        stage_id: BAM_GENOTYPING,
        tool_id: "bcftools",
        asset_role: "sites_vcf",
        asset_id: "human_like_genotyping_candidate_sites",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_genotyping_candidate_sites.vcf",
        label: "BAM genotyping",
    },
    RequiredAssetBinding {
        stage_id: BAM_GENOTYPING,
        tool_id: "bcftools",
        asset_role: "regions",
        asset_id: "human_like_genotyping_target_regions",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_genotyping_target_regions.txt",
        label: "BAM genotyping",
    },
    RequiredAssetBinding {
        stage_id: BAM_RECALIBRATION,
        tool_id: "gatk",
        asset_role: "reference_fasta",
        asset_id: "corpus_01_bam_reference",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta",
        label: "BAM recalibration",
    },
    RequiredAssetBinding {
        stage_id: BAM_RECALIBRATION,
        tool_id: "gatk",
        asset_role: "known_sites",
        asset_id: "human_like_recalibration_known_sites",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf",
        label: "BAM recalibration",
    },
    RequiredAssetBinding {
        stage_id: BAM_KINSHIP,
        tool_id: "king",
        asset_role: "reference_fasta",
        asset_id: "corpus_01_bam_reference",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta",
        label: "BAM kinship",
    },
    RequiredAssetBinding {
        stage_id: BAM_KINSHIP,
        tool_id: "king",
        asset_role: "reference_panel",
        asset_id: "human_like_relatedness_panel",
        asset_path: "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_relatedness_panel.tsv",
        label: "BAM kinship",
    },
];

fn validate_taxonomy_database_roots(rows: &[StageToolAssetRow]) -> Result<(), BenchError> {
    let taxonomy_rows = rows
        .iter()
        .filter(|row| {
            row.domain == "fastq"
                && row.stage_id == FASTQ_SCREEN
                && row.asset_role == "taxonomy_database_root"
        })
        .collect::<Vec<_>>();
    let expected_tool_ids = TAXONOMY_TOOL_IDS;
    if taxonomy_rows.len() != expected_tool_ids.len() {
        return Err(BenchError::InvalidObservation {
            reason: format!(
                "FASTQ taxonomy asset coverage expected {} rows but found {}",
                expected_tool_ids.len(),
                taxonomy_rows.len()
            ),
        });
    }
    for tool_id in expected_tool_ids {
        let row = taxonomy_rows.iter().find(|row| row.tool_id == tool_id).ok_or_else(|| {
            BenchError::InvalidObservation {
                reason: format!("FASTQ taxonomy asset coverage is missing `{tool_id}`"),
            }
        })?;
        if row.asset_id != "taxonomy_reference"
            || row.asset_path != "assets/reference/taxonomy/references/mock_community_taxonomy"
        {
            return Err(BenchError::InvalidObservation { reason: format!(
                "FASTQ taxonomy asset row `{tool_id}` must stay bound to the governed taxonomy reference root"
            ) });
        }
    }
    Ok(())
}

fn require_asset_binding(
    rows: &[StageToolAssetRow],
    required: &RequiredAssetBinding,
) -> Result<(), BenchError> {
    let RequiredAssetBinding { stage_id, tool_id, asset_role, asset_id, asset_path, label } =
        *required;
    let row = rows
        .iter()
        .find(|row| {
            row.stage_id == stage_id && row.tool_id == tool_id && row.asset_role == asset_role
        })
        .ok_or_else(|| BenchError::InvalidObservation {
            reason: format!(
                "{label} asset coverage is missing `{stage_id}` / `{tool_id}` / `{asset_role}`"
            ),
        })?;
    if row.asset_id != asset_id || row.asset_path != asset_path {
        return Err(BenchError::InvalidObservation { reason: format!(
            "{label} asset row `{stage_id}` / `{tool_id}` / `{asset_role}` must stay bound to `{asset_id}` at `{asset_path}`"
        ) });
    }
    Ok(())
}

fn validate_rows(rows: &[StageToolAssetRow]) -> Result<(), BenchError> {
    if rows.is_empty() {
        return invalid("stage-tool asset rows are empty");
    }
    let mut identities = BTreeSet::new();
    for row in rows {
        if !matches!(row.domain.as_str(), "bam" | "fastq")
            || !row.stage_id.starts_with(&format!("{}.", row.domain))
            || [
                row.stage_id.as_str(),
                row.tool_id.as_str(),
                row.asset_role.as_str(),
                row.asset_id.as_str(),
                row.declaration_origin.as_str(),
            ]
            .iter()
            .any(|value| value.trim().is_empty())
            || row.declaration_origin != STAGE_TOOL_ASSETS_DECLARATION_ORIGIN
            || !safe_relative_path(&row.asset_path)
            || !identities.insert((
                row.domain.as_str(),
                row.stage_id.as_str(),
                row.tool_id.as_str(),
                row.asset_role.as_str(),
                row.asset_id.as_str(),
            ))
        {
            return invalid("stage-tool asset row semantics are invalid");
        }
    }
    if rows.windows(2).any(|rows| row_key(&rows[0]) >= row_key(&rows[1])) {
        return invalid("stage-tool asset rows are not canonically ordered");
    }
    Ok(())
}

fn row_key(row: &StageToolAssetRow) -> (&str, &str, &str, &str, &str) {
    (&row.domain, &row.stage_id, &row.tool_id, &row.asset_role, &row.asset_id)
}

fn count_by<'a>(
    rows: &'a [StageToolAssetRow],
    key: impl Fn(&'a StageToolAssetRow) -> &'a str,
) -> BTreeMap<String, usize> {
    rows.iter().fold(BTreeMap::new(), |mut counts, row| {
        *counts.entry(key(row).to_string()).or_default() += 1;
        counts
    })
}

fn safe_relative_path(value: &str) -> bool {
    let path = std::path::Path::new(value);
    !value.trim().is_empty()
        && !path.is_absolute()
        && path.components().all(|component| {
            matches!(component, std::path::Component::Normal(_) | std::path::Component::CurDir)
        })
}

fn invalid<T>(reason: &str) -> Result<T, BenchError> {
    Err(BenchError::InvalidObservation { reason: reason.to_string() })
}
