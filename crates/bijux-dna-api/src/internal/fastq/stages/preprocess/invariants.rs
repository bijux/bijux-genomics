use super::*;

#[derive(Debug, Clone, serde::Serialize)]
pub(super) struct FastqInvariantsReport {
    schema_version: String,
    pub(super) r1: FastqFileInvariant,
    pub(super) r2: Option<FastqFileInvariant>,
    pub(super) paired_consistent: bool,
    pub(super) paired_reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(super) struct FastqFileInvariant {
    path: PathBuf,
    gzip: bool,
    gzip_valid: bool,
    pub(super) read_count: u64,
    read_length_min: usize,
    read_length_max: usize,
    pub(super) read_length_mean: f64,
    read_length_histogram: std::collections::BTreeMap<String, u64>,
    qscore_ascii_min: u8,
    qscore_ascii_max: u8,
    pub(super) quality_encoding: String,
    quality_encoding_confidence: String,
}

#[derive(Debug, Clone)]
struct FastqScanStats {
    read_count: u64,
    read_length_min: usize,
    read_length_max: usize,
    read_length_mean: f64,
    read_length_histogram: std::collections::BTreeMap<String, u64>,
    qscore_ascii_min: u8,
    qscore_ascii_max: u8,
    first_headers: Vec<String>,
}

fn histogram_bucket_for_read_length(len: usize) -> String {
    if len < 50 {
        "lt50".to_string()
    } else if len < 75 {
        "50_74".to_string()
    } else if len < 100 {
        "75_99".to_string()
    } else if len < 151 {
        "100_150".to_string()
    } else if len < 251 {
        "151_250".to_string()
    } else {
        "ge251".to_string()
    }
}

fn fastq_is_gzip(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("gz"))
}

fn validate_gzip_path(path: &std::path::Path) -> Result<bool> {
    if !fastq_is_gzip(path) {
        return Ok(true);
    }
    let mut magic = [0_u8; 2];
    let mut file = std::fs::File::open(path)?;
    if file.read_exact(&mut magic).is_err() || magic != [0x1f, 0x8b] {
        return Ok(false);
    }
    let args = vec!["-t".to_string(), path.to_string_lossy().into_owned()];
    let output = bijux_dna_runner::command_runner::run_command("gzip", &args);
    Ok(output.map(|result| result.exit_code == 0).unwrap_or(false))
}

fn quality_encoding_confidence(min_ascii: u8, max_ascii: u8) -> String {
    if (33..=59).contains(&min_ascii) && max_ascii <= 74 {
        "high".to_string()
    } else if min_ascii >= 64 && max_ascii <= 104 {
        "medium".to_string()
    } else {
        "low".to_string()
    }
}

pub(super) fn open_fastq_lines(
    path: &std::path::Path,
) -> Result<Box<dyn Iterator<Item = String>>> {
    if path
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x.eq_ignore_ascii_case("gz"))
    {
        let args = vec!["-cd".to_string(), path.to_string_lossy().into_owned()];
        let output = bijux_dna_runner::command_runner::run_command("gzip", &args)
            .with_context(|| format!("gzip -cd {}", path.display()))?;
        if output.exit_code != 0 {
            return Err(anyhow!(
                "failed to decompress {}: {}",
                path.display(),
                output.stderr
            ));
        }
        let text = output.stdout;
        let lines = text.lines().map(ToString::to_string).collect::<Vec<_>>();
        return Ok(Box::new(lines.into_iter()));
    }
    let f = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = std::io::BufReader::new(f);
    let mut lines = Vec::new();
    for line in reader.lines() {
        lines.push(line?);
    }
    Ok(Box::new(lines.into_iter()))
}

fn quality_encoding_from_ascii(min_ascii: u8, max_ascii: u8) -> String {
    if min_ascii >= 33 && max_ascii <= 74 {
        "phred+33".to_string()
    } else if min_ascii >= 64 && max_ascii <= 104 {
        "phred+64".to_string()
    } else {
        "unclassified".to_string()
    }
}

fn scan_fastq_invariants(path: &std::path::Path) -> Result<FastqScanStats> {
    let mut read_count = 0_u64;
    let mut len_min = usize::MAX;
    let mut len_max = 0_usize;
    let mut len_total = 0_u64;
    let mut q_min = u8::MAX;
    let mut q_max = 0_u8;
    let mut first_headers = Vec::new();
    let mut read_length_histogram = std::collections::BTreeMap::<String, u64>::new();
    let mut i = 0_u64;
    let mut it = open_fastq_lines(path)?;
    loop {
        let h = it.next();
        let seq = it.next();
        let plus = it.next();
        let qual = it.next();
        let (Some(h), Some(seq), Some(plus), Some(qual)) = (h, seq, plus, qual) else {
            break;
        };
        if !h.starts_with('@') || !plus.starts_with('+') {
            return Err(anyhow!(
                "invalid FASTQ record framing in {}",
                path.display()
            ));
        }
        let l = seq.len();
        len_min = len_min.min(l);
        len_max = len_max.max(l);
        len_total += l as u64;
        *read_length_histogram
            .entry(histogram_bucket_for_read_length(l))
            .or_insert(0) += 1;
        for c in qual.bytes() {
            q_min = q_min.min(c);
            q_max = q_max.max(c);
        }
        if i < 16 {
            first_headers.push(h);
        }
        read_count += 1;
        i += 1;
    }
    if read_count == 0 {
        return Err(anyhow!("no reads detected in {}", path.display()));
    }
    Ok(FastqScanStats {
        read_count,
        read_length_min: len_min,
        read_length_max: len_max,
        read_length_mean: u64_to_f64(len_total) / u64_to_f64(read_count),
        read_length_histogram,
        qscore_ascii_min: q_min,
        qscore_ascii_max: q_max,
        first_headers,
    })
}

fn normalize_pair_header(header: &str) -> String {
    let core = header
        .trim_start_matches('@')
        .split_whitespace()
        .next()
        .unwrap_or(header);
    core.trim_end_matches("/1")
        .trim_end_matches("/2")
        .to_string()
}

pub(super) fn write_fastq_entry_invariants(
    root: &std::path::Path,
    r1: &std::path::Path,
    r2: Option<&std::path::Path>,
) -> Result<FastqInvariantsReport> {
    let r1s = scan_fastq_invariants(r1)?;
    let r1_gzip = fastq_is_gzip(r1);
    let r1_gzip_valid = validate_gzip_path(r1)?;
    if r1_gzip && !r1_gzip_valid {
        return Err(anyhow!("invalid gzip FASTQ stream: {}", r1.display()));
    }
    let r1_inv = FastqFileInvariant {
        path: r1.to_path_buf(),
        gzip: r1_gzip,
        gzip_valid: r1_gzip_valid,
        read_count: r1s.read_count,
        read_length_min: r1s.read_length_min,
        read_length_max: r1s.read_length_max,
        read_length_mean: r1s.read_length_mean,
        read_length_histogram: r1s.read_length_histogram.clone(),
        qscore_ascii_min: r1s.qscore_ascii_min,
        qscore_ascii_max: r1s.qscore_ascii_max,
        quality_encoding: quality_encoding_from_ascii(r1s.qscore_ascii_min, r1s.qscore_ascii_max),
        quality_encoding_confidence: quality_encoding_confidence(
            r1s.qscore_ascii_min,
            r1s.qscore_ascii_max,
        ),
    };
    let (r2_inv, paired_consistent, paired_reason) = if let Some(r2_path) = r2 {
        let r2s = scan_fastq_invariants(r2_path)?;
        let r2_gzip = fastq_is_gzip(r2_path);
        let r2_gzip_valid = validate_gzip_path(r2_path)?;
        if r2_gzip && !r2_gzip_valid {
            return Err(anyhow!("invalid gzip FASTQ stream: {}", r2_path.display()));
        }
        let mut ok = r1s.read_count == r2s.read_count;
        let mut reason = None;
        if ok {
            for (lhs, rhs) in r1s.first_headers.iter().zip(r2s.first_headers.iter()) {
                if normalize_pair_header(lhs) != normalize_pair_header(rhs) {
                    ok = false;
                    reason = Some("header pairing mismatch".to_string());
                    break;
                }
            }
        } else {
            reason = Some("read count mismatch between R1 and R2".to_string());
        }
        (
            Some(FastqFileInvariant {
                path: r2_path.to_path_buf(),
                gzip: r2_gzip,
                gzip_valid: r2_gzip_valid,
                read_count: r2s.read_count,
                read_length_min: r2s.read_length_min,
                read_length_max: r2s.read_length_max,
                read_length_mean: r2s.read_length_mean,
                read_length_histogram: r2s.read_length_histogram.clone(),
                qscore_ascii_min: r2s.qscore_ascii_min,
                qscore_ascii_max: r2s.qscore_ascii_max,
                quality_encoding: quality_encoding_from_ascii(
                    r2s.qscore_ascii_min,
                    r2s.qscore_ascii_max,
                ),
                quality_encoding_confidence: quality_encoding_confidence(
                    r2s.qscore_ascii_min,
                    r2s.qscore_ascii_max,
                ),
            }),
            ok,
            reason,
        )
    } else {
        (None, true, None)
    };
    let report = FastqInvariantsReport {
        schema_version: "bijux.fastq.invariants.v1".to_string(),
        r1: r1_inv,
        r2: r2_inv,
        paired_consistent,
        paired_reason,
    };
    bijux_dna_infra::atomic_write_json(&root.join("fastq_invariants.json"), &report)
        .context("write fastq_invariants.json")?;
    Ok(report)
}

fn u64_to_f64(v: u64) -> f64 {
    v.to_string().parse::<f64>().unwrap_or(0.0)
}
