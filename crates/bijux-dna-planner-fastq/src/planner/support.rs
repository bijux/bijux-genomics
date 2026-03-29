use anyhow::Result;
use bijux_dna_domain_fastq::{STAGE_EXTRACT_UMIS, STAGE_MERGE_PAIRS};

pub(crate) fn apply_layout_branching(mut stages: Vec<String>, paired: bool) -> Vec<String> {
    if paired {
        return stages;
    }
    // Single-end runs must not schedule paired-only stages.
    stages.retain(|stage| {
        stage != STAGE_MERGE_PAIRS.as_str() && stage != STAGE_EXTRACT_UMIS.as_str()
    });
    stages
}

pub(crate) fn estimate_mean_q(path: &std::path::Path, max_records: usize) -> Result<f64> {
    let raw = std::fs::read_to_string(path)?;
    let mut total = 0.0;
    let mut count = 0_u64;
    for (idx, line) in raw.lines().enumerate() {
        if idx % 4 == 3 {
            for byte in line.as_bytes() {
                let score = (*byte as i32 - 33).max(0) as f64;
                total += score;
                count += 1;
            }
            if (idx / 4) + 1 >= max_records {
                break;
            }
        }
    }
    if count == 0 {
        return Ok(0.0);
    }
    Ok(total / count as f64)
}
