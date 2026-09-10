//! Owner: bijux-dna-bench-corpus
//! AMPLICON corpus abundance ownership.
use super::{
    anyhow, fs, path_relative_to_repo, resolve_manifest_relative_path,
    AmpliconAbundanceTableMetrics, AmpliconAbundanceTruthRow, AmpliconCorpusAbundanceTable,
    AmpliconCorpusAbundanceTableValidationReport, AmpliconCorpusFixtureManifest,
    AmpliconCorpusSampleKind, BTreeSet, Path, Result,
};
use anyhow::Context;

/// Load declared rows and validate their identities, values and coverage.
///
/// # Errors
/// Rejects unreadable or malformed tables and rows that contradict the corpus declarations.
pub fn load_validated_amplicon_abundance_rows(
    repo_root: &Path,
    manifest_path: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    table_kind: &str,
) -> Result<Vec<AmpliconAbundanceTruthRow>> {
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        anyhow!("fixture manifest has no parent directory: {}", manifest_path.display())
    })?;
    let table =
        manifest.abundance_tables.iter().find(|table| table.table_kind == table_kind).ok_or_else(
            || {
                anyhow!(
                    "amplicon corpus fixture `{}` does not declare abundance table kind `{}`",
                    manifest.corpus_id,
                    table_kind
                )
            },
        )?;
    validate_amplicon_abundance_table(repo_root, manifest_dir, manifest, table)?;
    load_amplicon_abundance_rows(repo_root, manifest_dir, manifest, table)
}

pub(super) fn load_amplicon_abundance_rows(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    table: &AmpliconCorpusAbundanceTable,
) -> Result<Vec<AmpliconAbundanceTruthRow>> {
    let table_path = resolve_manifest_relative_path(manifest_dir, &table.table_path);
    let raw = fs::read_to_string(&table_path)
        .with_context(|| format!("read {}", table_path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!(
            "amplicon corpus abundance table `{}` is empty: {}",
            table.sample_id,
            table_path.display()
        )
    })?;
    if header != "sample_id\tfeature_id\tabundance" {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` header is unexpected in {}",
            table.sample_id,
            table_path.display()
        ));
    }

    let biological_sample_ids = manifest
        .samples
        .iter()
        .filter(|sample| matches!(sample.sample_kind, AmpliconCorpusSampleKind::Biological))
        .map(|sample| sample.sample_id.as_str())
        .collect::<BTreeSet<_>>();
    let manifest_relative_path = path_relative_to_repo(repo_root, &table_path);
    let mut sample_feature_pairs = BTreeSet::new();
    let mut rows = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let mut fields = line.split('\t');
        let sample_id = fields.next().ok_or_else(|| {
            anyhow!(
                "amplicon corpus abundance table `{}` row is missing `sample_id` in {}",
                table.sample_id,
                manifest_relative_path
            )
        })?;
        let feature_id = fields.next().ok_or_else(|| {
            anyhow!(
                "amplicon corpus abundance table `{}` row is missing `feature_id` in {}",
                table.sample_id,
                manifest_relative_path
            )
        })?;
        let abundance = fields.next().ok_or_else(|| {
            anyhow!(
                "amplicon corpus abundance table `{}` row is missing `abundance` in {}",
                table.sample_id,
                manifest_relative_path
            )
        })?;
        if fields.next().is_some() {
            return Err(anyhow!(
                "amplicon corpus abundance table `{}` row has too many columns in {}",
                table.sample_id,
                manifest_relative_path
            ));
        }
        if sample_id.trim().is_empty() || feature_id.trim().is_empty() {
            return Err(anyhow!(
                "amplicon corpus abundance table `{}` must not contain empty identifiers in {}",
                table.sample_id,
                manifest_relative_path
            ));
        }
        if !biological_sample_ids.contains(sample_id) {
            return Err(anyhow!(
                "amplicon corpus abundance table `{}` references undeclared biological sample `{sample_id}` in {}",
                table.sample_id,
                manifest_relative_path
            ));
        }
        let abundance_value = abundance.parse::<u64>().with_context(|| {
            format!(
                "parse abundance value for sample `{sample_id}` feature `{feature_id}` in {manifest_relative_path}"
            )
        })?;
        if !sample_feature_pairs.insert((sample_id.to_string(), feature_id.to_string())) {
            return Err(anyhow!(
                "amplicon corpus abundance table `{}` repeats (`{sample_id}`, `{feature_id}`) in {}",
                table.sample_id,
                manifest_relative_path
            ));
        }
        rows.push(AmpliconAbundanceTruthRow {
            sample_id: sample_id.to_string(),
            feature_id: feature_id.to_string(),
            abundance: abundance_value,
        });
    }
    if rows.is_empty() {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` must declare at least one row in {}",
            table.sample_id,
            manifest_relative_path
        ));
    }
    Ok(rows)
}

pub(super) fn validate_amplicon_abundance_table(
    repo_root: &Path,
    manifest_dir: &Path,
    manifest: &AmpliconCorpusFixtureManifest,
    table: &AmpliconCorpusAbundanceTable,
) -> Result<AmpliconCorpusAbundanceTableValidationReport> {
    let table_path = resolve_manifest_relative_path(manifest_dir, &table.table_path);
    if !table_path.is_file() {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` is missing: {}",
            table.sample_id,
            table_path.display()
        ));
    }

    let raw = fs::read_to_string(&table_path)
        .with_context(|| format!("read {}", table_path.display()))?;
    let mut lines = raw.lines();
    let header = lines.next().ok_or_else(|| {
        anyhow!(
            "amplicon corpus abundance table `{}` is empty: {}",
            table.sample_id,
            table_path.display()
        )
    })?;
    if header != "sample_id\tfeature_id\tabundance" {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` header is unexpected in {}",
            table.sample_id,
            table_path.display()
        ));
    }

    let biological_sample_ids = manifest
        .samples
        .iter()
        .filter(|sample| matches!(sample.sample_kind, AmpliconCorpusSampleKind::Biological))
        .map(|sample| sample.sample_id.as_str())
        .collect::<BTreeSet<_>>();
    let manifest_relative_path = path_relative_to_repo(repo_root, &table_path);
    let mut sample_ids = BTreeSet::new();
    let mut feature_ids = BTreeSet::new();
    let mut sample_feature_pairs = BTreeSet::new();
    let mut row_count = 0_u64;

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let (sample_id, feature_id) =
            parse_abundance_row(line, table, &biological_sample_ids, &manifest_relative_path)?;
        if !sample_feature_pairs.insert((sample_id.to_string(), feature_id.to_string())) {
            return Err(anyhow!(
                "amplicon corpus abundance table `{}` repeats (`{sample_id}`, `{feature_id}`) in {}",
                table.sample_id,
                manifest_relative_path
            ));
        }
        sample_ids.insert(sample_id.to_string());
        feature_ids.insert(feature_id.to_string());
        row_count = row_count.saturating_add(1);
    }

    if row_count == 0 {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` must declare at least one row in {}",
            table.sample_id,
            manifest_relative_path
        ));
    }

    let metrics = AmpliconAbundanceTableMetrics {
        row_count,
        sample_count: sample_ids.len() as u64,
        feature_count: feature_ids.len() as u64,
    };
    validate_abundance_counts(table, &metrics)?;

    let source_paths = crate::paths::validate_source_paths(
        repo_root,
        &table.source_paths,
        &format!("amplicon corpus abundance table `{}`", table.sample_id),
    )?;

    Ok(AmpliconCorpusAbundanceTableValidationReport {
        sample_id: table.sample_id.clone(),
        table_kind: table.table_kind.clone(),
        table_path: path_relative_to_repo(repo_root, &table_path),
        source_paths,
        observed_row_count: metrics.row_count,
        observed_sample_count: metrics.sample_count,
        observed_feature_count: metrics.feature_count,
        valid: true,
    })
}

fn parse_abundance_row<'a>(
    line: &'a str,
    table: &AmpliconCorpusAbundanceTable,
    biological_sample_ids: &BTreeSet<&str>,
    manifest_relative_path: &str,
) -> Result<(&'a str, &'a str)> {
    let mut fields = line.split('\t');
    let sample_id = fields.next().ok_or_else(|| {
        anyhow!(
            "amplicon corpus abundance table `{}` row is missing `sample_id` in {}",
            table.sample_id,
            manifest_relative_path
        )
    })?;
    let feature_id = fields.next().ok_or_else(|| {
        anyhow!(
            "amplicon corpus abundance table `{}` row is missing `feature_id` in {}",
            table.sample_id,
            manifest_relative_path
        )
    })?;
    let abundance = fields.next().ok_or_else(|| {
        anyhow!(
            "amplicon corpus abundance table `{}` row is missing `abundance` in {}",
            table.sample_id,
            manifest_relative_path
        )
    })?;
    if fields.next().is_some() {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` row has too many columns in {}",
            table.sample_id,
            manifest_relative_path
        ));
    }
    if sample_id.trim().is_empty() {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` must not contain empty `sample_id` values in {}",
            table.sample_id,
            manifest_relative_path
        ));
    }
    if feature_id.trim().is_empty() {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` must not contain empty `feature_id` values in {}",
            table.sample_id,
            manifest_relative_path
        ));
    }
    if !biological_sample_ids.contains(sample_id) {
        return Err(anyhow!(
        "amplicon corpus abundance table `{}` references undeclared biological sample `{sample_id}` in {}",
        table.sample_id,
        manifest_relative_path
    ));
    }
    let abundance_value = abundance.parse::<f64>().with_context(|| {
    format!(
        "parse abundance value for sample `{sample_id}` feature `{feature_id}` in {manifest_relative_path}"
    )
})?;
    if !abundance_value.is_finite() || abundance_value < 0.0 {
        return Err(anyhow!(
        "amplicon corpus abundance table `{}` must use finite non-negative abundance values in {}",
        table.sample_id,
        manifest_relative_path
    ));
    }
    Ok((sample_id, feature_id))
}

fn validate_abundance_counts(
    table: &AmpliconCorpusAbundanceTable,
    metrics: &AmpliconAbundanceTableMetrics,
) -> Result<()> {
    if metrics.row_count != table.expected_row_count {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` expected {} rows but observed {}",
            table.sample_id,
            table.expected_row_count,
            metrics.row_count
        ));
    }
    if metrics.sample_count != table.expected_sample_count {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` expected {} samples but observed {}",
            table.sample_id,
            table.expected_sample_count,
            metrics.sample_count
        ));
    }
    if metrics.feature_count != table.expected_feature_count {
        return Err(anyhow!(
            "amplicon corpus abundance table `{}` expected {} features but observed {}",
            table.sample_id,
            table.expected_feature_count,
            metrics.feature_count
        ));
    }
    Ok(())
}
