use super::*;

pub(crate) fn try_run_tool(bin: &str, args: &[&str]) -> bool {
    std::process::Command::new(bin).args(args).output().map(|x| x.status.success()).unwrap_or(false)
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
