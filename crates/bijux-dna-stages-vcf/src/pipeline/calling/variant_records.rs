use super::*;

pub(crate) fn parse_record_fields(line: &str) -> Option<Vec<&str>> {
    if line.trim().is_empty() || line.starts_with('#') {
        return None;
    }
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() < 8 {
        return None;
    }
    Some(fields)
}

pub(crate) fn read_vcf_text(path: &Path) -> Result<String> {
    if path.extension().and_then(|x| x.to_str()).is_some_and(|x| x == "gz" || x == "bcf") {
        let output = crate::engine::execution::run_command_output(
            "bcftools",
            ["view", &path.display().to_string()],
            None,
        )?;
        if output.exit_code == 0 {
            return Ok(output.stdout);
        }
        // Compatibility fallback for legacy plain-text payloads mislabeled as `.vcf.gz`.
        if let Ok(raw) = std::fs::read(path) {
            return Ok(String::from_utf8_lossy(&raw).to_string());
        }
        bail!("bcftools view failed while reading {}: {}", path.display(), output.stderr);
    }
    Ok(std::fs::read_to_string(path)?)
}

pub(crate) fn variant_key(fields: &[&str]) -> Option<(String, String)> {
    if fields.len() < 5 {
        return None;
    }
    let chr = fields[0].to_string();
    let key = format!("{}:{}:{}:{}", fields[0], fields[1], fields[3], fields[4]);
    Some((chr, key))
}

pub(crate) fn normalize_alleles(reference: &str, alternate: &str) -> (String, String) {
    (reference.to_ascii_uppercase(), alternate.to_ascii_uppercase())
}

pub(crate) fn format_has_token(fmt: &str, tokens: &[&str]) -> bool {
    fmt.split(':').any(|key| tokens.iter().any(|token| token == &key))
}

pub(crate) fn sample_has_diploid_gt(fmt: &str, sample: &str) -> bool {
    let keys = fmt.split(':').collect::<Vec<_>>();
    let Some(gt_idx) = keys.iter().position(|k| *k == "GT") else {
        return false;
    };
    let vals = sample.split(':').collect::<Vec<_>>();
    let Some(gt) = vals.get(gt_idx) else {
        return false;
    };
    gt.split(['/', '|']).count() == 2
}

pub(crate) fn sample_to_haploid_gt(fmt: &str, sample: &str) -> String {
    let keys = fmt.split(':').collect::<Vec<_>>();
    let mut vals = sample.split(':').map(str::to_string).collect::<Vec<_>>();
    if let Some(gt_idx) = keys.iter().position(|k| *k == "GT") {
        if let Some(gt) = vals.get(gt_idx).cloned() {
            let first = gt.split(['/', '|']).next().unwrap_or(".").to_string();
            vals[gt_idx] = first;
            return vals.join(":");
        }
    }
    if let Some(gp_idx) = keys.iter().position(|k| *k == "GP") {
        if let Some(gp) = vals.get(gp_idx) {
            let probs = gp.split(',').filter_map(|x| x.parse::<f64>().ok()).collect::<Vec<_>>();
            if probs.len() >= 3 {
                let best_idx = probs
                    .iter()
                    .enumerate()
                    .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(idx, _)| idx)
                    .unwrap_or(1);
                let hap = if best_idx == 0 { "0" } else { "1" };
                return hap.to_string();
            }
        }
    }
    if let Some(pl_idx) = keys.iter().position(|k| *k == "PL") {
        if let Some(pl) = vals.get(pl_idx) {
            let scores = pl.split(',').filter_map(|x| x.parse::<f64>().ok()).collect::<Vec<_>>();
            if scores.len() >= 3 {
                let best_idx = scores
                    .iter()
                    .enumerate()
                    .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(idx, _)| idx)
                    .unwrap_or(1);
                let hap = if best_idx == 0 { "0" } else { "1" };
                return hap.to_string();
            }
        }
    }
    sample.to_string()
}

pub(crate) fn normalize_header_sample_order(vcf_text: &str) -> String {
    let mut out = String::new();
    let mut sample_order: Option<Vec<usize>> = None;
    for line in vcf_text.lines() {
        if line.starts_with("#CHROM\t") {
            let parts = line.split('\t').collect::<Vec<_>>();
            if parts.len() <= 9 {
                out.push_str(line);
                out.push('\n');
                continue;
            }
            let fixed = parts[..9].to_vec();
            let mut samples = parts[9..]
                .iter()
                .enumerate()
                .map(|(i, name)| (i, (*name).to_string()))
                .collect::<Vec<_>>();
            samples.sort_by(|a, b| a.1.cmp(&b.1));
            let order = samples.iter().map(|(i, _)| *i).collect::<Vec<_>>();
            sample_order = Some(order);
            let mut row = fixed.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
            row.extend(samples.into_iter().map(|(_, name)| name));
            out.push_str(&row.join("\t"));
            out.push('\n');
            continue;
        }
        if line.starts_with('#') {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if let (Some(order), Some(fields)) = (sample_order.as_ref(), parse_record_fields(line)) {
            if fields.len() > 9 {
                let mut row = fields.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
                let samples = row[9..].to_vec();
                let reordered =
                    order.iter().filter_map(|idx| samples.get(*idx).cloned()).collect::<Vec<_>>();
                row.truncate(9);
                row.extend(reordered);
                out.push_str(&row.join("\t"));
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

pub(crate) fn parse_info_value_f64(info: &str, key: &str) -> Option<f64> {
    info.split(';').find_map(|entry| {
        let mut parts = entry.splitn(2, '=');
        match (parts.next(), parts.next()) {
            (Some(k), Some(v)) if k == key => v.parse::<f64>().ok(),
            _ => None,
        }
    })
}

pub(crate) fn normalize_sample_fields(format_field: &str, sample_field: &str) -> String {
    let keys = format_field.split(':').collect::<Vec<_>>();
    let mut vals = sample_field.split(':').map(str::to_string).collect::<Vec<_>>();
    if vals.len() < keys.len() {
        vals.resize(keys.len(), ".".to_string());
    }
    for (i, key) in keys.iter().enumerate() {
        if vals.get(i).is_none_or(|v| v.trim().is_empty()) {
            vals[i] = ".".to_string();
        }
        if (*key == "GL" || *key == "PL") && vals[i] == "." {
            vals[i] = ".,.,.".to_string();
        }
    }
    vals.join(":")
}

pub(crate) fn parse_af_from_info(info: &str) -> Option<f64> {
    parse_info_value_f64(info, "AF").or_else(|| parse_info_value_f64(info, "MAF"))
}

pub(crate) fn genotype_missing_fraction(format_field: &str, sample_fields: &[&str]) -> Option<f64> {
    let keys = format_field.split(':').collect::<Vec<_>>();
    let gt_idx = keys.iter().position(|k| *k == "GT")?;
    if sample_fields.is_empty() {
        return Some(0.0);
    }
    let mut missing = 0_u64;
    let mut total = 0_u64;
    for sample in sample_fields {
        let vals = sample.split(':').collect::<Vec<_>>();
        if let Some(gt) = vals.get(gt_idx) {
            total += 1;
            if gt.contains('.') {
                missing += 1;
            }
        }
    }
    Some(if total == 0 { 0.0 } else { missing as f64 / total as f64 })
}
