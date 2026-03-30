use super::*;

pub(crate) fn variant_maf(fields: &[&str]) -> Option<f64> {
    if let Some(v) = parse_info_value_f64(fields[7], "AF") {
        return Some(if v > 0.5 { 1.0 - v } else { v });
    }
    if fields.len() <= 9 {
        return None;
    }
    let gt_idx = parse_format_index(fields, "GT")?;
    let mut alt = 0_u64;
    let mut total = 0_u64;
    for sample in &fields[9..] {
        let vals = sample.split(':').collect::<Vec<_>>();
        let gt = *vals.get(gt_idx)?;
        if gt.contains('.') {
            continue;
        }
        for allele in gt.split(['/', '|']) {
            total += 1;
            if allele == "1" {
                alt += 1;
            }
        }
    }
    if total == 0 {
        None
    } else {
        let af = alt as f64 / total as f64;
        Some(if af > 0.5 { 1.0 - af } else { af })
    }
}

pub(crate) fn try_run_tool(bin: &str, args: &[&str]) -> bool {
    std::process::Command::new(bin)
        .args(args)
        .output()
        .map(|x| x.status.success())
        .unwrap_or(false)
}

pub(crate) fn write_bgzip_with_best_effort_index(
    out_vcfgz: &Path,
    payload: &str,
    tmp_name: &str,
) -> Result<PathBuf> {
    let tmp_vcf = out_vcfgz
        .parent()
        .ok_or_else(|| anyhow!("missing parent for {}", out_vcfgz.display()))?
        .join(tmp_name);
    atomic_write_bytes(&tmp_vcf, payload.as_bytes())?;
    let out_tbi = crate::vcf_io::vcf_index_bgzip_tabix(&tmp_vcf, out_vcfgz)?;
    let _ = std::fs::remove_file(&tmp_vcf);
    Ok(out_tbi)
}
