use super::{open_fastq_lines, Context, Result};

pub(crate) fn load_qc_thresholds_map() -> std::collections::BTreeMap<String, f64> {
    let Some(path) =
        std::env::var_os("BIJUX_QC_THRESHOLDS_PATH").map(std::path::PathBuf::from).or_else(|| {
            std::env::var_os("BIJUX_REFERENCE_ROOT")
                .map(std::path::PathBuf::from)
                .map(|root| root.join("qc_thresholds.yaml"))
        })
    else {
        return std::collections::BTreeMap::new();
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return std::collections::BTreeMap::new();
    };
    raw.lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || !line.contains(':') {
                return None;
            }
            let (k, v) = line.split_once(':')?;
            let key = k.trim().to_string();
            let value = v.trim().parse::<f64>().ok()?;
            Some((key, value))
        })
        .collect()
}

pub(crate) fn copy_if_missing(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    if dst.exists() {
        return Ok(());
    }
    if let Some(parent) = dst.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    std::fs::copy(src, dst)
        .with_context(|| format!("copy {} -> {}", src.display(), dst.display()))?;
    Ok(())
}

pub(crate) fn command_exists(bin: &str) -> bool {
    let args = vec!["--version".to_string()];
    bijux_dna_runner::command_runner::run_command(bin, &args).is_ok()
}

pub(crate) fn run_stage_command(
    out_dir: &std::path::Path,
    command_label: &str,
    bin: &str,
    args: &[String],
) -> bool {
    let output = bijux_dna_runner::command_runner::run_command(bin, args);
    let (ok, stdout, stderr) = match output {
        Ok(out) => (out.exit_code == 0, out.stdout, out.stderr),
        Err(err) => (false, String::new(), format!("{err}")),
    };
    let payload = format!(
        "label={command_label}\ncmd={} {}\nok={ok}\n--- stdout ---\n{}\n--- stderr ---\n{}\n",
        bin,
        args.join(" "),
        stdout,
        stderr
    );
    let _ = bijux_dna_infra::atomic_write_bytes(
        &out_dir.join(format!("{command_label}.command.log")),
        payload.as_bytes(),
    );
    ok
}

pub(crate) fn write_fastq_to_fasta_if_missing(
    input_fastq: &std::path::Path,
    out_fasta: &std::path::Path,
) -> Result<()> {
    if out_fasta.exists() {
        return Ok(());
    }
    if command_exists("seqkit") {
        let ok = run_stage_command(
            out_fasta.parent().unwrap_or_else(|| std::path::Path::new(".")),
            "seqkit_fq2fa",
            "seqkit",
            &[
                "fq2fa".to_string(),
                input_fastq.to_string_lossy().to_string(),
                "-o".to_string(),
                out_fasta.to_string_lossy().to_string(),
            ],
        );
        if ok && out_fasta.exists() {
            return Ok(());
        }
    }
    // Deterministic fallback converter for basic FASTQ input.
    let mut out = String::new();
    let mut it = open_fastq_lines(input_fastq)?;
    while let (Some(h), Some(seq), Some(_plus), Some(_qual)) =
        (it.next(), it.next(), it.next(), it.next())
    {
        let header = h.trim_start_matches('@');
        out.push('>');
        out.push_str(header);
        out.push('\n');
        out.push_str(seq.trim());
        out.push('\n');
    }
    bijux_dna_infra::atomic_write_bytes(out_fasta, out.as_bytes())?;
    Ok(())
}
