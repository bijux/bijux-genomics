//! Owner: bijux-dna-analyze
//! Facts loaders.

use bijux_dna_runtime::FactsRowV1;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde_json;

use super::AnalyzeError;
use crate::model::stable_sort_records;
use crate::model::FactTable;

/// Load facts from a JSONL file.
///
/// # Errors
/// Returns an error if the file is missing, unreadable, or contains invalid rows.
pub fn load_facts(path: &Path) -> std::result::Result<Vec<FactsRowV1>, AnalyzeError> {
    if !path.exists() {
        return Err(AnalyzeError::MissingFile {
            path: path.display().to_string(),
        });
    }
    let file = std::fs::File::open(path).map_err(|err| AnalyzeError::InvalidJson {
        message: err.to_string(),
    })?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|err| AnalyzeError::InvalidJson {
            message: err.to_string(),
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let mut parsed_row: FactsRowV1 =
            serde_json::from_str(&line).map_err(|err| AnalyzeError::InvalidJsonlRow {
                line: idx + 1,
                message: err.to_string(),
            })?;
        if parsed_row.schema_version != "bijux.facts.v1" {
            return Err(AnalyzeError::InvalidSchemaVersion {
                found: parsed_row.schema_version,
                expected: "bijux.facts.v1".to_string(),
            });
        }
        if !parsed_row.effective_metric_provenance().is_complete() {
            return Err(AnalyzeError::InvalidJsonlRow {
                line: idx + 1,
                message: "incomplete metric provenance contract".to_string(),
            });
        }
        normalize_bank_hashes(&mut parsed_row.bank_hashes);
        rows.push(parsed_row);
    }
    stable_sort_records(&mut rows, |row| {
        (
            row.run_id.as_str(),
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.params_hash.as_str(),
            "",
        )
    });
    Ok(rows)
}

/// # Errors
/// Returns an error if the parquet reader is not enabled.
#[cfg(not(feature = "parquet"))]
pub fn load_facts_parquet(_path: &Path) -> std::result::Result<Vec<FactsRowV1>, AnalyzeError> {
    Err(AnalyzeError::UnsupportedParquet {
        message: "enable the parquet feature to read parquet facts".to_string(),
    })
}

/// # Errors
/// Returns an error if the parquet reader fails.
#[cfg(feature = "parquet")]
pub fn load_facts_parquet(path: &Path) -> std::result::Result<Vec<FactsRowV1>, AnalyzeError> {
    use parquet::file::reader::{FileReader, SerializedFileReader};
    let file = std::fs::File::open(path).map_err(|err| AnalyzeError::InvalidJson {
        message: err.to_string(),
    })?;
    let reader = SerializedFileReader::new(file).map_err(|err| AnalyzeError::InvalidJson {
        message: err.to_string(),
    })?;
    let mut rows = Vec::new();
    let iter = reader
        .get_row_iter(None)
        .map_err(|err| AnalyzeError::InvalidJson {
            message: err.to_string(),
        })?;
    for record in iter {
        let record = record.map_err(|err| AnalyzeError::InvalidJson {
            message: err.to_string(),
        })?;
        let value = record.to_json_value();
        let mut parsed: FactsRowV1 =
            serde_json::from_value(value).map_err(|err| AnalyzeError::InvalidJson {
                message: err.to_string(),
            })?;
        if parsed.schema_version != "bijux.facts.v1" {
            return Err(AnalyzeError::InvalidSchemaVersion {
                found: parsed.schema_version,
                expected: "bijux.facts.v1".to_string(),
            });
        }
        if !parsed.effective_metric_provenance().is_complete() {
            return Err(AnalyzeError::InvalidJson {
                message: "incomplete metric provenance contract".to_string(),
            });
        }
        normalize_bank_hashes(&mut parsed.bank_hashes);
        rows.push(parsed);
    }
    stable_sort_records(&mut rows, |row| {
        (
            row.run_id.as_str(),
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.params_hash.as_str(),
            "",
        )
    });
    Ok(rows)
}

fn normalize_bank_hashes(bank_hashes: &mut serde_json::Value) {
    let Some(obj) = bank_hashes.as_object_mut() else {
        *bank_hashes = serde_json::json!({
            "adapter_bank_hash": "unknown",
            "reference_bank_hash": "unknown",
            "taxonomy_db_hash": "unknown",
            "taxonomy_db_version": "unknown",
        });
        return;
    };
    obj.entry("adapter_bank_hash".to_string())
        .or_insert_with(|| serde_json::Value::String("unknown".to_string()));
    obj.entry("reference_bank_hash".to_string())
        .or_insert_with(|| serde_json::Value::String("unknown".to_string()));
    obj.entry("taxonomy_db_hash".to_string())
        .or_insert_with(|| serde_json::Value::String("unknown".to_string()));
    obj.entry("taxonomy_db_version".to_string())
        .or_insert_with(|| serde_json::Value::String("unknown".to_string()));
}

/// # Errors
/// Returns an error if facts cannot be loaded from jsonl or parquet.
pub fn load_facts_auto(path: &Path) -> std::result::Result<Vec<FactsRowV1>, AnalyzeError> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("parquet") => load_facts_parquet(path),
        _ => load_facts(path),
    }
}

/// # Errors
/// Returns an error if facts loading or validation fails.
pub fn load_fact_table(path: &Path) -> std::result::Result<FactTable, AnalyzeError> {
    let rows = load_facts(path)?;
    FactTable::from_facts(&rows).map_err(|err| AnalyzeError::InvalidJson {
        message: err.to_string(),
    })
}
