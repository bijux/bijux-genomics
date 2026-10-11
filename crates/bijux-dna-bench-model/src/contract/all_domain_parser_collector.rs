//! Owner: bijux-dna-bench-model
//! Semantic admission for all-domain parser-collector evidence.

use std::collections::{BTreeMap, BTreeSet};

use bijux_dna_core::id_catalog::{BAM_VALIDATE, FASTQ_VALIDATE_READS};

use crate::{
    model::{
        AllDomainParserCollectorReport, AllDomainParserCollectorRow,
        AllDomainParserCollectorSourceKind, ALL_DOMAIN_PARSER_COLLECTOR_FAKE_RUN_ROOT,
        ALL_DOMAIN_PARSER_COLLECTOR_FIXTURE_ROOT, ALL_DOMAIN_PARSER_COLLECTOR_OUTPUT_PATH,
    },
    BenchError,
};

use super::ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_V1;

const REQUIRED_REAL_SMOKES: [(&str, &str, &str, &str); 4] = [
    ("real-smoke:bam.validate", "bam", BAM_VALIDATE, "samtools"),
    ("real-smoke:fastq.validate_reads", "fastq", FASTQ_VALIDATE_READS, "fastqc"),
    ("real-smoke:vcf.prepare_reference_panel", "vcf", "vcf.prepare_reference_panel", "bcftools"),
    ("real-smoke:vcf.stats", "vcf", "vcf.stats", "bcftools"),
];

struct CollectorCounts {
    fake_run_rows: usize,
    real_smoke_rows: usize,
    source_kinds: BTreeMap<String, usize>,
    domains: BTreeMap<String, usize>,
    document_kinds: BTreeMap<String, usize>,
}

/// Validate persisted parser-collector evidence and its aggregate closure.
///
/// # Errors
/// Returns an error for identity drift, incomplete real-smoke coverage, malformed rows,
/// or aggregate mismatch.
pub fn validate_all_domain_parser_collector(
    report: &AllDomainParserCollectorReport,
) -> Result<(), BenchError> {
    if report.schema_version != ALL_DOMAIN_PARSER_COLLECTOR_SCHEMA_V1 {
        return invalid("unsupported all-domain parser-collector schema");
    }
    if report.output_path != ALL_DOMAIN_PARSER_COLLECTOR_OUTPUT_PATH
        || report.fixture_root != ALL_DOMAIN_PARSER_COLLECTOR_FIXTURE_ROOT
        || report.fake_run_root != ALL_DOMAIN_PARSER_COLLECTOR_FAKE_RUN_ROOT
    {
        return invalid("all-domain parser-collector paths are not canonical");
    }
    if report.rows.is_empty() {
        return invalid("all-domain parser collector must produce at least one row");
    }

    let counts = validate_rows(&report.rows)?;
    if report.row_count != report.rows.len()
        || report.fake_run_row_count != counts.fake_run_rows
        || report.real_smoke_row_count != counts.real_smoke_rows
        || report.source_kind_counts != counts.source_kinds
        || report.domain_counts != counts.domains
        || report.document_kind_counts != counts.document_kinds
    {
        return invalid("all-domain parser-collector aggregates do not close");
    }
    Ok(())
}

fn validate_rows(rows: &[AllDomainParserCollectorRow]) -> Result<CollectorCounts, BenchError> {
    let mut seen_record_ids = BTreeSet::new();
    let mut fake_run_result_ids = BTreeSet::new();
    let mut real_smoke_record_ids = BTreeSet::new();
    let mut counts = CollectorCounts {
        fake_run_rows: 0,
        real_smoke_rows: 0,
        source_kinds: BTreeMap::new(),
        domains: BTreeMap::new(),
        document_kinds: BTreeMap::new(),
    };
    for row in rows {
        validate_row_shape(row, &mut seen_record_ids)?;
        match row.source_kind {
            AllDomainParserCollectorSourceKind::FakeRun => {
                validate_fake_run_row(row, &mut fake_run_result_ids)?;
                counts.fake_run_rows += 1;
            }
            AllDomainParserCollectorSourceKind::RealSmoke => {
                validate_real_smoke_row(row)?;
                real_smoke_record_ids.insert(row.record_id.as_str());
                counts.real_smoke_rows += 1;
            }
        }
        *counts.source_kinds.entry(row.source_kind.as_str().to_string()).or_default() += 1;
        *counts.domains.entry(row.domain.clone()).or_default() += 1;
        *counts.document_kinds.entry(row.document_kind.clone()).or_default() += 1;
    }
    let required = REQUIRED_REAL_SMOKES.into_iter().map(|(record_id, _, _, _)| record_id).collect();
    if real_smoke_record_ids != required {
        return invalid(
            "all-domain parser collector must retain fastq, bam, vcf.stats, and vcf.prepare_reference_panel real-smoke evidence",
        );
    }
    Ok(counts)
}

fn validate_row_shape<'a>(
    row: &'a AllDomainParserCollectorRow,
    seen_record_ids: &mut BTreeSet<&'a str>,
) -> Result<(), BenchError> {
    for (value, field) in [
        (&row.record_id, "record_id"),
        (&row.document_kind, "document_kind"),
        (&row.domain, "domain"),
        (&row.stage_id, "stage_id"),
        (&row.tool_id, "tool_id"),
        (&row.corpus_id, "corpus_id"),
        (&row.asset_profile_id, "asset_profile_id"),
        (&row.parsed_path, "parsed_path"),
        (&row.parsed_schema_version, "parsed_schema_version"),
    ] {
        require_text(value, field)?;
    }
    if !seen_record_ids.insert(row.record_id.as_str()) {
        return invalid(format!("duplicate parser record_id `{}`", row.record_id));
    }
    let mut canonical_keys = row.parsed_top_level_keys.clone();
    canonical_keys.sort();
    canonical_keys.dedup();
    if row.parsed_top_level_key_count != row.parsed_top_level_keys.len()
        || row.parsed_top_level_keys.is_empty()
        || canonical_keys != row.parsed_top_level_keys
    {
        return invalid(format!(
            "parser row `{}` has invalid parsed-key accounting",
            row.record_id
        ));
    }
    if row.normalized_snapshot.is_empty() {
        return invalid(format!("parser row `{}` lacks normalized evidence", row.record_id));
    }
    Ok(())
}

fn validate_fake_run_row<'a>(
    row: &'a AllDomainParserCollectorRow,
    result_ids: &mut BTreeSet<&'a str>,
) -> Result<(), BenchError> {
    let result_id = row.result_id.as_deref().ok_or_else(|| {
        BenchError::InvalidPolicy(format!(
            "fake-run parser row `{}` lacks result_id",
            row.record_id
        ))
    })?;
    require_text(result_id, "result_id")?;
    if !result_ids.insert(result_id) {
        return invalid(format!("duplicate fake-run result_id `{result_id}`"));
    }
    Ok(())
}

fn validate_real_smoke_row(row: &AllDomainParserCollectorRow) -> Result<(), BenchError> {
    if row.result_id.is_some() {
        return invalid(format!(
            "real-smoke parser row `{}` must not declare result_id",
            row.record_id
        ));
    }
    let (_, domain, stage_id, tool_id) = REQUIRED_REAL_SMOKES
        .iter()
        .find(|(record_id, _, _, _)| *record_id == row.record_id)
        .ok_or_else(|| {
            BenchError::InvalidPolicy(format!(
                "unexpected real-smoke parser row `{}`",
                row.record_id
            ))
        })?;
    if row.domain != *domain || row.stage_id != *stage_id || row.tool_id != *tool_id {
        return invalid(format!(
            "real-smoke parser row `{}` drifted from its stage/tool contract",
            row.record_id
        ));
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("all-domain parser collector requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
