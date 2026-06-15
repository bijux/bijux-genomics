fn parse_acceptance_stage_keys(raw: &str, stage_id: &str) -> Vec<String> {
    let mut in_target = false;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "[[stage]]" {
            in_target = false;
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("stage_id = ") {
            in_target = value.trim_matches('"') == stage_id;
            continue;
        }
        if in_target {
            if let Some(value) = trimmed.strip_prefix("acceptance = [") {
                let inner = value.trim_end_matches(']').trim();
                if inner.is_empty() {
                    return Vec::new();
                }
                return inner
                    .split(',')
                    .map(|x| x.trim().trim_matches('"').to_string())
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<_>>();
            }
        }
    }
    Vec::new()
}

fn load_downstream_acceptance_for_stage(stage_id: &str) -> Vec<String> {
    let raw = workspace_root()
        .and_then(|root| {
            std::fs::read_to_string(root.join("configs/vcf/downstream_acceptance.toml")).ok()
        })
        .unwrap_or_default();
    parse_acceptance_stage_keys(&raw, stage_id)
}

fn write_impute_bgzip_index_best_effort(
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

#[derive(Clone, Copy)]
enum BackendEvidence {
    GlLikelihood,
    PhasedWithMapMinimac,
    PhasedWithMap,
    Generic,
}

fn choose_backend_by_regime(requested: ImputeBackend, evidence: BackendEvidence) -> ImputeBackend {
    if !matches!(requested, ImputeBackend::Beagle) {
        return requested;
    }
    match evidence {
        BackendEvidence::GlLikelihood => ImputeBackend::Glimpse,
        BackendEvidence::PhasedWithMapMinimac => ImputeBackend::Minimac4,
        BackendEvidence::PhasedWithMap => ImputeBackend::Impute5,
        BackendEvidence::Generic => ImputeBackend::Beagle,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImputationSupportCall {
    genotype: String,
    donor_support: u64,
    low_confidence: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ImputedRecordOutcome {
    record_line: String,
    missing_before: u64,
    missing_after: u64,
    imputed_genotypes: u64,
    low_confidence_count: u64,
    not_imputable_reasons: std::collections::BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MaskedTruthOutcome {
    masked_truth_site_count: u64,
    imputed_match_count: u64,
    imputed_mismatch_count: u64,
    unresolved_count: u64,
    unresolved_reasons: std::collections::BTreeMap<String, u64>,
}

fn ensure_imputation_format_headers(headers: &mut Vec<String>, emit_ds: bool, emit_gp: bool) {
    if emit_ds && !headers.iter().any(|line| line.starts_with("##FORMAT=<ID=DS,")) {
        headers.push(
            "##FORMAT=<ID=DS,Number=1,Type=Float,Description=\"Deterministic imputed dosage\">"
                .to_string(),
        );
    }
    if emit_gp && !headers.iter().any(|line| line.starts_with("##FORMAT=<ID=GP,")) {
        headers.push(
            "##FORMAT=<ID=GP,Number=G,Type=Float,Description=\"Deterministic imputed genotype probabilities\">"
                .to_string(),
        );
    }
}

fn normalize_vcf_headers(headers: &[String]) -> Vec<String> {
    let mut fileformat = headers
        .iter()
        .find(|line| line.starts_with("##fileformat="))
        .cloned()
        .unwrap_or_else(|| "##fileformat=VCFv4.2".to_string());
    if fileformat.is_empty() {
        fileformat = "##fileformat=VCFv4.2".to_string();
    }
    let chrom_header =
        headers.iter().find(|line| line.starts_with("#CHROM\t")).cloned().unwrap_or_default();
    let mut meta_headers = headers
        .iter()
        .filter(|line| !line.starts_with("##fileformat=") && !line.starts_with("#CHROM\t"))
        .cloned()
        .collect::<Vec<_>>();
    meta_headers.sort();

    let mut normalized = vec![fileformat];
    normalized.extend(meta_headers);
    if !chrom_header.is_empty() {
        normalized.push(chrom_header);
    }
    normalized
}

fn is_missing_genotype(gt: &str) -> bool {
    gt.trim().is_empty() || gt == "." || gt.contains('.')
}

fn canonicalize_diploid_genotype(gt: &str) -> Option<String> {
    if is_missing_genotype(gt) {
        return None;
    }
    let mut alleles =
        gt.split(['/', '|']).map(str::trim).filter(|allele| !allele.is_empty()).collect::<Vec<_>>();
    if alleles.len() != 2 {
        return None;
    }
    alleles.sort_unstable();
    Some(format!("{}/{}", alleles[0], alleles[1]))
}

fn genotype_dosage(gt: &str) -> Option<f64> {
    let canonical = canonicalize_diploid_genotype(gt)?;
    let dosage = canonical.split('/').filter_map(|allele| allele.parse::<u64>().ok()).sum::<u64>();
    Some(dosage as f64)
}

fn genotype_probability_vector(gt: &str, low_confidence: bool) -> Option<&'static str> {
    let canonical = canonicalize_diploid_genotype(gt)?;
    let high_confidence = match canonical.as_str() {
        "0/0" => "0.97,0.02,0.01",
        "0/1" => "0.05,0.90,0.05",
        "1/1" => "0.01,0.02,0.97",
        _ => return None,
    };
    let low_confidence_probs = match canonical.as_str() {
        "0/0" => "0.65,0.25,0.10",
        "0/1" => "0.20,0.60,0.20",
        "1/1" => "0.10,0.25,0.65",
        _ => return None,
    };
    Some(if low_confidence { low_confidence_probs } else { high_confidence })
}

fn resolve_imputation_support_call(
    fields: &[&str],
    gt_idx: usize,
) -> Option<ImputationSupportCall> {
    let mut donors = std::collections::BTreeMap::<String, u64>::new();
    for sample_field in fields.iter().skip(9) {
        let parts = sample_field.split(':').collect::<Vec<_>>();
        let Some(gt) = parts.get(gt_idx).copied() else {
            continue;
        };
        let Some(canonical) = canonicalize_diploid_genotype(gt) else {
            continue;
        };
        *donors.entry(canonical).or_insert(0) += 1;
    }
    let total_donors = donors.values().sum::<u64>();
    if total_donors == 0 {
        return None;
    }
    let best_support = donors.values().copied().max()?;
    let tied_best = donors.values().filter(|support| **support == best_support).count();
    if tied_best != 1 {
        return None;
    }
    let (genotype, donor_support) =
        donors.into_iter().find(|(_, support)| *support == best_support)?;
    Some(ImputationSupportCall {
        genotype,
        donor_support,
        low_confidence: donor_support < 2 || donor_support != total_donors,
    })
}

fn rewrite_imputed_record(
    line: &str,
    emit_ds: bool,
    emit_gp: bool,
) -> Result<ImputedRecordOutcome> {
    let fields = parse_record_fields(line)
        .ok_or_else(|| anyhow!("impute stage encountered malformed VCF record"))?;
    if fields.len() <= 8 {
        return Ok(ImputedRecordOutcome {
            record_line: line.to_string(),
            missing_before: 0,
            missing_after: 0,
            imputed_genotypes: 0,
            low_confidence_count: 0,
            not_imputable_reasons: std::collections::BTreeMap::new(),
        });
    }

    let mut columns = fields.iter().map(|field| (*field).to_string()).collect::<Vec<_>>();
    let mut format_keys = columns[8].split(':').map(str::to_string).collect::<Vec<_>>();
    let gt_idx = format_keys.iter().position(|key| key == "GT");
    let ds_idx = if emit_ds {
        Some(format_keys.iter().position(|key| key == "DS").unwrap_or_else(|| {
            format_keys.push("DS".to_string());
            format_keys.len() - 1
        }))
    } else {
        None
    };
    let gp_idx = if emit_gp {
        Some(format_keys.iter().position(|key| key == "GP").unwrap_or_else(|| {
            format_keys.push("GP".to_string());
            format_keys.len() - 1
        }))
    } else {
        None
    };

    let support_call = gt_idx.and_then(|idx| resolve_imputation_support_call(&fields, idx));
    let mut missing_before = 0_u64;
    let mut missing_after = 0_u64;
    let mut imputed_genotypes = 0_u64;
    let mut low_confidence_count = 0_u64;
    let mut not_imputable_reasons = std::collections::BTreeMap::<String, u64>::new();

    for column in columns.iter_mut().skip(9) {
        let mut values = column.split(':').map(str::to_string).collect::<Vec<_>>();
        values.resize(format_keys.len(), ".".to_string());

        let mut final_gt =
            gt_idx.and_then(|idx| values.get(idx).cloned()).unwrap_or_else(|| ".".to_string());
        let gt_was_missing =
            gt_idx.and_then(|idx| values.get(idx)).is_some_and(|gt| is_missing_genotype(gt));
        if gt_was_missing {
            missing_before += 1;
            match &support_call {
                Some(support) => {
                    if let Some(idx) = gt_idx {
                        final_gt = support.genotype.clone();
                        values[idx] = final_gt.clone();
                        imputed_genotypes += 1;
                        if support.low_confidence {
                            low_confidence_count += 1;
                        }
                    }
                }
                None => {
                    *not_imputable_reasons
                        .entry("insufficient_donor_support".to_string())
                        .or_insert(0) += 1;
                }
            }
        }

        if gt_idx.is_some() && is_missing_genotype(&final_gt) {
            missing_after += 1;
        }

        if let Some(idx) = ds_idx {
            values[idx] = genotype_dosage(&final_gt)
                .map(|value| format!("{value:.3}"))
                .unwrap_or_else(|| ".".to_string());
        }
        if let Some(idx) = gp_idx {
            let low_confidence = gt_was_missing
                && support_call.as_ref().is_some_and(|support| support.low_confidence);
            values[idx] = genotype_probability_vector(&final_gt, low_confidence)
                .map(str::to_string)
                .unwrap_or_else(|| ".".to_string());
        }

        *column = values.join(":");
    }

    columns[8] = format_keys.join(":");
    Ok(ImputedRecordOutcome {
        record_line: columns.join("\t"),
        missing_before,
        missing_after,
        imputed_genotypes,
        low_confidence_count,
        not_imputable_reasons,
    })
}

fn collect_truth_genotypes(
    truth_path: &Path,
) -> Result<std::collections::BTreeMap<String, Vec<String>>> {
    let truth_raw = std::fs::read_to_string(truth_path)?;
    let mut truth_gt = std::collections::BTreeMap::<String, Vec<String>>::new();
    for line in truth_raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let gt_idx = parse_format_index(&fields, "GT");
        let Some((_, key)) = variant_key(&fields) else {
            continue;
        };
        let sample_gts = fields
            .iter()
            .skip(9)
            .map(|sample_field| {
                let parts = sample_field.split(':').collect::<Vec<_>>();
                gt_idx.and_then(|idx| parts.get(idx).copied()).unwrap_or(".").to_string()
            })
            .collect::<Vec<_>>();
        truth_gt.insert(key, sample_gts);
    }
    Ok(truth_gt)
}

fn compare_masked_truth(
    input_records: &[String],
    imputed_records: &[String],
    truth_path: &Path,
) -> Result<MaskedTruthOutcome> {
    let truth_gt = collect_truth_genotypes(truth_path)?;
    let imputed_by_key = imputed_records
        .iter()
        .filter_map(|line| {
            let fields = parse_record_fields(line)?;
            let (_, key) = variant_key(&fields)?;
            Some((key, fields))
        })
        .map(|(key, fields)| {
            (key, fields.iter().map(|field| (*field).to_string()).collect::<Vec<_>>())
        })
        .collect::<std::collections::BTreeMap<_, _>>();

    let mut masked_truth_site_count = 0_u64;
    let mut imputed_match_count = 0_u64;
    let mut imputed_mismatch_count = 0_u64;
    let mut unresolved_count = 0_u64;
    let mut unresolved_reasons = std::collections::BTreeMap::<String, u64>::new();

    for line in input_records {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let Some((_, key)) = variant_key(&fields) else {
            continue;
        };
        let Some(gt_idx) = parse_format_index(&fields, "GT") else {
            continue;
        };
        let Some(expected_samples) = truth_gt.get(&key) else {
            continue;
        };
        let Some(imputed_fields) = imputed_by_key.get(&key) else {
            continue;
        };
        for sample_offset in 0..expected_samples.len() {
            let input_sample = fields.get(9 + sample_offset).copied().unwrap_or(".");
            let imputed_sample =
                imputed_fields.get(9 + sample_offset).map(String::as_str).unwrap_or(".");
            let input_parts = input_sample.split(':').collect::<Vec<_>>();
            let imputed_parts = imputed_sample.split(':').collect::<Vec<_>>();
            let input_gt = input_parts.get(gt_idx).copied().unwrap_or(".");
            let imputed_gt = imputed_parts.get(gt_idx).copied().unwrap_or(".");
            let expected_gt =
                expected_samples.get(sample_offset).map(String::as_str).unwrap_or(".");

            if !is_missing_genotype(input_gt) || is_missing_genotype(expected_gt) {
                continue;
            }

            masked_truth_site_count += 1;
            if let Some(observed) = canonicalize_diploid_genotype(imputed_gt) {
                if canonicalize_diploid_genotype(expected_gt).as_deref()
                    == Some(observed.as_str())
                {
                    imputed_match_count += 1;
                } else {
                    imputed_mismatch_count += 1;
                }
            } else {
                unresolved_count += 1;
                *unresolved_reasons
                    .entry("insufficient_donor_support".to_string())
                    .or_insert(0) += 1;
            }
        }
    }

    Ok(MaskedTruthOutcome {
        masked_truth_site_count,
        imputed_match_count,
        imputed_mismatch_count,
        unresolved_count,
        unresolved_reasons,
    })
}

include!("execution_engine.rs");

include!("postprocess.rs");
