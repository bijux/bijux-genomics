use anyhow::Result;

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
