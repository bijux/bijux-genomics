use crate::planning::values::parse_cli_arg_pairs;
use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
const DEPLETE_RRNA_DEFAULT_MIN_IDENTITY: f64 = 0.95;
#[derive(Debug, Clone)]
pub(crate) struct DepleteRrnaStageOptions {
    pub(crate) rrna_db: PathBuf,
    pub(crate) rrna_bundle_id: String,
    pub(crate) min_identity: f64,
}

pub(crate) fn resolve_deplete_rrna_stage_options(
    stage_args: &[String],
) -> Result<DepleteRrnaStageOptions> {
    let stage_values = parse_cli_arg_pairs("stage-arg", stage_args)?;
    let rrna_db = stage_values
        .get("rrna_db")
        .ok_or_else(|| anyhow!("fastq.deplete_rrna requires --rrna-db in stage arguments"))?;
    let rrna_db = PathBuf::from(rrna_db);
    if !rrna_db.is_file() {
        return Err(anyhow!("missing rRNA bundle FASTA: {}", rrna_db.display()));
    }
    let bundle_id = stage_values.get("rrna_bundle_id").ok_or_else(|| {
        anyhow!("fastq.deplete_rrna requires --rrna-bundle-id in stage arguments")
    })?;
    crate::admission::path_component(bundle_id, "rRNA bundle identifier")?;
    Ok(DepleteRrnaStageOptions {
        rrna_db,
        rrna_bundle_id: stage_values.get("rrna_bundle_id").cloned().ok_or_else(|| {
            anyhow!("fastq.deplete_rrna requires --rrna-bundle-id in stage arguments")
        })?,
        min_identity: stage_values
            .get("min_identity")
            .map(|row| row.parse::<f64>())
            .transpose()
            .with_context(|| "parse min_identity from stage arguments")?
            .unwrap_or(DEPLETE_RRNA_DEFAULT_MIN_IDENTITY),
    })
}
