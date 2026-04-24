use super::*;

#[derive(Debug, Clone)]
pub struct DamageFilterStageParams {
    pub udg_regime: DamageUdgRegime,
    pub strict_regime: bool,
    pub min_qual: f64,
    pub max_damage_ratio: f64,
}

impl DamageFilterStageParams {
    #[must_use]
    pub fn recommended() -> Self {
        Self {
            udg_regime: DamageUdgRegime::Unknown,
            strict_regime: true,
            min_qual: 30.0,
            max_damage_ratio: 0.35,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DamageFilterOutputs {
    pub filtered_vcf: PathBuf,
    pub filtered_tbi: PathBuf,
    pub damage_filter_summary_json: PathBuf,
    pub damage_filter_counts_json: PathBuf,
    pub warnings_json: PathBuf,
    pub damage_genotype_manifest_json: PathBuf,
}

fn env_damage_mask_mode() -> String {
    std::env::var("BIJUX_VCF_DAMAGE_MASK_MODE")
        .ok()
        .map(|v| v.to_ascii_lowercase())
        .filter(|v| v == "remove" || v == "mark")
        .unwrap_or_else(|| "remove".to_string())
}

fn env_terminal_damage_threshold() -> f64 {
    std::env::var("BIJUX_VCF_DAMAGE_TERMINAL_THRESHOLD")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .map(|v| v.clamp(0.0, 1.0))
        .unwrap_or(0.50)
}

fn env_pmd_min_default(udg_regime: DamageUdgRegime) -> f64 {
    if let Some(parsed) =
        std::env::var("BIJUX_VCF_DAMAGE_PMD_MIN").ok().and_then(|v| v.parse::<f64>().ok())
    {
        return parsed;
    }
    match udg_regime {
        DamageUdgRegime::Udg => 0.0,
        DamageUdgRegime::NonUdg => 3.0,
        DamageUdgRegime::Unknown => 1.0,
    }
}

fn env_library_type() -> String {
    std::env::var("BIJUX_LIBRARY_TYPE")
        .ok()
        .map(|v| v.to_ascii_lowercase())
        .filter(|v| matches!(v.as_str(), "ssdna" | "dsdna"))
        .unwrap_or_else(|| "not_declared".to_string())
}

#[derive(Debug, Clone)]
pub struct GlPropagationStageParams {
    pub require_gl_or_pl: bool,
    pub expected_ploidy: Option<u8>,
    pub emit_bcf: bool,
}

impl GlPropagationStageParams {
    #[must_use]
    pub fn recommended() -> Self {
        Self { require_gl_or_pl: true, expected_ploidy: Some(2), emit_bcf: true }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GlPropagationOutputs {
    pub normalized_vcf: PathBuf,
    pub normalized_tbi: PathBuf,
    pub normalized_bcf: Option<PathBuf>,
    pub normalized_bcf_csi: Option<PathBuf>,
    pub gl_propagation_report_json: PathBuf,
}

/// # Errors
/// Returns an error if GL propagation contracts are violated.
pub fn run_gl_propagation_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &GlPropagationStageParams,
) -> Result<GlPropagationOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = read_vcf_text(input_vcf)?;
    let mut output_lines = Vec::<String>::new();
    let mut has_gl_or_pl = false;
    let mut allele_reordered = 0_u64;
    let mut ploidy_mismatch = 0_u64;
    let mut missing_normalized = 0_u64;
    let mut records = 0_u64;

    for line in raw.lines() {
        if line.starts_with('#') {
            output_lines.push(line.to_string());
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        records += 1;
        let mut row = fields.iter().map(|x| (*x).to_string()).collect::<Vec<_>>();
        if row.len() > 8 {
            let fmt = row[8].clone();
            if format_has_token(&fmt, &["GL", "PL"]) {
                has_gl_or_pl = true;
            }
            if row.len() > 9 {
                let before = row[9].clone();
                row[9] = normalize_sample_fields(&fmt, &row[9]);
                if row[9] != before {
                    missing_normalized += 1;
                }
                if let Some(expected) = params.expected_ploidy {
                    let keys = fmt.split(':').collect::<Vec<_>>();
                    if let Some(gt_idx) = keys.iter().position(|k| *k == "GT") {
                        let vals = row[9].split(':').collect::<Vec<_>>();
                        if let Some(gt) = vals.get(gt_idx) {
                            let observed = gt.split(['/', '|']).count() as u8;
                            if !gt.contains('.') && observed != expected {
                                ploidy_mismatch += 1;
                            }
                        }
                    }
                }
            }
        }
        if row.len() > 4 {
            let mut alts = row[4].split(',').map(str::to_string).collect::<Vec<_>>();
            let original = alts.clone();
            alts.sort();
            if alts != original {
                if row.len() > 8 && format_has_token(&row[8], &["GL", "PL"]) {
                    bail!("vcf.gl_propagation refusal: allele ordering mismatch with GL/PL fields");
                }
                row[4] = alts.join(",");
                allele_reordered += 1;
            }
        }
        output_lines.push(row.join("\t"));
    }
    if let Some(chrom_idx) = output_lines.iter().position(|line| line.starts_with("#CHROM\t")) {
        let has_gt = output_lines.iter().any(|line| line.starts_with("##FORMAT=<ID=GT,"));
        let has_gl = output_lines.iter().any(|line| line.starts_with("##FORMAT=<ID=GL,"));
        let has_pl = output_lines.iter().any(|line| line.starts_with("##FORMAT=<ID=PL,"));
        if !has_gt {
            output_lines.insert(
                chrom_idx,
                "##FORMAT=<ID=GT,Number=1,Type=String,Description=\"Genotype\">".to_string(),
            );
        }
        let insert_at =
            output_lines.iter().position(|line| line.starts_with("#CHROM\t")).unwrap_or(chrom_idx);
        if !has_gl {
            output_lines.insert(
                insert_at,
                "##FORMAT=<ID=GL,Number=G,Type=Float,Description=\"Genotype Likelihood\">"
                    .to_string(),
            );
        }
        let insert_at =
            output_lines.iter().position(|line| line.starts_with("#CHROM\t")).unwrap_or(chrom_idx);
        if !has_pl {
            output_lines.insert(
                insert_at,
                "##FORMAT=<ID=PL,Number=G,Type=Integer,Description=\"Phred-scaled Genotype Likelihood\">".to_string(),
            );
        }
    }
    if params.require_gl_or_pl && !has_gl_or_pl {
        bail!("vcf.gl_propagation requires GL/PL in FORMAT for downstream compatibility");
    }
    if ploidy_mismatch > 0 {
        bail!("vcf.gl_propagation refusal: ploidy mismatch detected in GT fields");
    }

    let normalized_vcf = out_dir.join("gl_normalized.vcf.gz");
    let normalized_payload =
        output_lines.join("\n") + if output_lines.is_empty() { "" } else { "\n" };
    let normalized_tbi =
        write_vcf_with_best_effort_index(&normalized_vcf, &normalized_payload, "gl_propagation")?;

    let (normalized_bcf, normalized_bcf_csi) = if params.emit_bcf {
        let bcf = out_dir.join("gl_normalized.bcf");
        let csi = out_dir.join("gl_normalized.bcf.csi");
        let normalized_vcf_s =
            normalized_vcf.to_str().ok_or_else(|| anyhow!("non-utf8 gl normalized vcf path"))?;
        let bcf_s = bcf.to_str().ok_or_else(|| anyhow!("non-utf8 gl normalized bcf path"))?;
        run_checked_command("bcftools", &["view", "-Ob", "-o", bcf_s, normalized_vcf_s])?;
        run_checked_command("bcftools", &["index", "-f", bcf_s])?;
        if !csi.exists() {
            bail!("vcf.gl_propagation contract violation: missing BCF index {}", csi.display());
        }
        (Some(bcf), Some(csi))
    } else {
        (None, None)
    };

    let gl_propagation_report_json = out_dir.join("gl_propagation_report.json");
    atomic_write_json(
        &gl_propagation_report_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.gl_propagation_report.v1",
            "records_seen": records,
            "has_gl_or_pl": has_gl_or_pl,
            "allele_reordered_records": allele_reordered,
            "missing_genotype_fields_normalized": missing_normalized,
            "expected_ploidy": params.expected_ploidy,
            "emit_bcf": params.emit_bcf
        }),
    )?;

    Ok(GlPropagationOutputs {
        normalized_vcf,
        normalized_tbi,
        normalized_bcf,
        normalized_bcf_csi,
        gl_propagation_report_json,
    })
}

/// # Errors
/// Returns an error if damage filtering contracts cannot be satisfied.
pub fn run_damage_filter_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &DamageFilterStageParams,
) -> Result<DamageFilterOutputs> {
    if params.strict_regime && params.udg_regime == DamageUdgRegime::Unknown {
        bail!("vcf.damage_filter refusal: strict mode requires known UDG regime");
    }
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = read_vcf_text(input_vcf)?;
    let mut headers = Vec::<String>::new();
    let mut kept = Vec::<String>::new();
    let mut counts = std::collections::BTreeMap::<String, u64>::new();
    let mut pre_damage_ct = 0_u64;
    let mut pre_damage_ga = 0_u64;
    let mut pre_total = 0_u64;
    let mut five_prime_signal = 0.0_f64;
    let mut three_prime_signal = 0.0_f64;
    let mut ct_five_prime_high = 0_u64;
    let mut ga_three_prime_high = 0_u64;
    let mut proxy_used = 0_u64;
    let mut filtered_ct = 0_u64;
    let mut filtered_ga = 0_u64;
    let mut filtered_other = 0_u64;
    let mut marked_lowconf = 0_u64;

    let mask_mode = env_damage_mask_mode();
    let library_type = env_library_type();
    let terminal_threshold = if library_type == "ssdna" {
        env_terminal_damage_threshold().min(0.40)
    } else {
        env_terminal_damage_threshold()
    };
    let pmd_min = env_pmd_min_default(params.udg_regime);
    let mut has_gl_like_field = false;
    let mut has_damage_info_field = false;

    let threshold = match params.udg_regime {
        DamageUdgRegime::Udg => params.max_damage_ratio.min(0.60),
        DamageUdgRegime::NonUdg => params.max_damage_ratio.max(0.20),
        DamageUdgRegime::Unknown => params.max_damage_ratio,
    };
    let bcftools_expression = format!(
        "QUAL>={:.3} && (INFO/CT_GA_DAMAGE_RATIO<={:.3} || !exists(INFO/CT_GA_DAMAGE_RATIO))",
        params.min_qual, threshold
    );

    for line in raw.lines() {
        if line.starts_with('#') {
            headers.push(line.to_string());
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        pre_total += 1;
        let reference = fields[3].to_ascii_uppercase();
        let alternate = fields[4].to_ascii_uppercase();
        let is_ct = reference == "C" && alternate == "T";
        let is_ga = reference == "G" && alternate == "A";
        if is_ct {
            pre_damage_ct += 1;
        }
        if is_ga {
            pre_damage_ga += 1;
        }
        let qual = fields[5].parse::<f64>().unwrap_or(0.0);
        if qual < params.min_qual {
            *counts.entry("low_qual".to_string()).or_insert(0) += 1;
            continue;
        }
        let info = fields[7];
        if fields.len() > 8 && format_has_token(fields[8], &["GL", "GP", "PL"]) {
            has_gl_like_field = true;
        }
        if info.contains("CT_GA_DAMAGE_RATIO=")
            || info.contains("DEAM5P=")
            || info.contains("DEAM3P=")
            || info.contains("PMD_SCORE=")
            || info.contains("PMD=")
            || info.contains("PMDSCORE=")
        {
            has_damage_info_field = true;
        }
        let ratio = if let Some(v) = parse_info_value_f64(info, "CT_GA_DAMAGE_RATIO") {
            v
        } else {
            proxy_used += 1;
            if is_ct || is_ga {
                1.0
            } else {
                0.0
            }
        };
        let pmd_score = parse_info_value_f64(info, "PMD_SCORE")
            .or_else(|| parse_info_value_f64(info, "PMD"))
            .or_else(|| parse_info_value_f64(info, "PMDSCORE"));
        let pmd_fail = pmd_score.is_some_and(|score| score < pmd_min);
        if is_ct || is_ga {
            let five = parse_info_value_f64(info, "DEAM5P").unwrap_or(ratio);
            let three = parse_info_value_f64(info, "DEAM3P").unwrap_or(ratio);
            five_prime_signal += five;
            three_prime_signal += three;
            if is_ct && five >= terminal_threshold {
                ct_five_prime_high += 1;
            }
            if is_ga && three >= terminal_threshold {
                ga_three_prime_high += 1;
            }
        }
        let terminal_damage = (is_ct
            && parse_info_value_f64(info, "DEAM5P").unwrap_or(ratio) >= terminal_threshold)
            || (is_ga
                && parse_info_value_f64(info, "DEAM3P").unwrap_or(ratio) >= terminal_threshold);
        let filter_for_damage = ratio > threshold || pmd_fail || terminal_damage;
        if filter_for_damage {
            if is_ct {
                filtered_ct += 1;
            } else if is_ga {
                filtered_ga += 1;
            } else {
                filtered_other += 1;
            }
            if ratio > threshold {
                *counts.entry("damage_ratio_exceeded".to_string()).or_insert(0) += 1;
            }
            if pmd_fail {
                *counts.entry("pmd_below_threshold".to_string()).or_insert(0) += 1;
            }
            if terminal_damage {
                *counts.entry("terminal_damage_filtered".to_string()).or_insert(0) += 1;
            }
            if mask_mode == "mark" {
                let mut row = fields.iter().map(|f| (*f).to_string()).collect::<Vec<_>>();
                row[6] = if row[6] == "." || row[6] == "PASS" {
                    "LOWCONF_DAMAGE_TERMINAL".to_string()
                } else {
                    format!("{};LOWCONF_DAMAGE_TERMINAL", row[6])
                };
                kept.push(row.join("\t"));
                marked_lowconf += 1;
                *counts.entry("lowconf_marked".to_string()).or_insert(0) += 1;
            }
            if mask_mode == "remove" {
                continue;
            }
        }
        *counts.entry("kept".to_string()).or_insert(0) += 1;
        kept.push(line.to_string());
    }
    let proxy_only_mode = pre_total > 0 && !has_gl_like_field && !has_damage_info_field;

    let filtered_vcf = out_dir.join("damage_filtered.vcf.gz");
    let mut payload = String::new();
    if !headers.is_empty() {
        payload.push_str(&headers.join("\n"));
        payload.push('\n');
    }
    if !kept.is_empty() {
        payload.push_str(&kept.join("\n"));
        payload.push('\n');
    }
    let filtered_tbi = write_vcf_with_best_effort_index(&filtered_vcf, &payload, "damage_filter")?;

    let total_damage = pre_damage_ct + pre_damage_ga;
    let asymmetry = if total_damage == 0 {
        0.0
    } else {
        (pre_damage_ct as f64 - pre_damage_ga as f64).abs() / total_damage as f64
    };
    let filtered_total = filtered_ct + filtered_ga + filtered_other;
    let filtered_fraction_by_mutation = if filtered_total == 0 {
        serde_json::json!({"ct": 0.0, "ga": 0.0, "other": 0.0})
    } else {
        serde_json::json!({
            "ct": filtered_ct as f64 / filtered_total as f64,
            "ga": filtered_ga as f64 / filtered_total as f64,
            "other": filtered_other as f64 / filtered_total as f64
        })
    };
    let damage_filter_summary_json = out_dir.join("damage_filter_summary.json");
    atomic_write_json(
        &damage_filter_summary_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.damage_filter_summary.v1",
            "udg_regime": params.udg_regime,
            "strict_regime": params.strict_regime,
            "bcftools_filter_expression": bcftools_expression,
            "prefilter": {
                "records_total": pre_total,
                "ct_events": pre_damage_ct,
                "ga_events": pre_damage_ga,
                "ct_ga_asymmetry": asymmetry,
                "read_position_signal": {
                    "five_prime_sum": five_prime_signal,
                    "three_prime_sum": three_prime_signal,
                    "ct_five_prime_high": ct_five_prime_high,
                    "ga_three_prime_high": ga_three_prime_high,
                    "proxy_used_records": proxy_used
                }
            },
            "filtering": {
                "filtered_counts": {
                    "ct": filtered_ct,
                    "ga": filtered_ga,
                    "other": filtered_other
                },
                "filtered_fraction_by_mutation_type": filtered_fraction_by_mutation,
                "marked_lowconf_records": marked_lowconf
            },
            "thresholds": {
                "min_qual": params.min_qual,
                "max_damage_ratio": threshold,
                "pmd_min": pmd_min,
                "terminal_damage_threshold": terminal_threshold
            },
            "masking_strategy": {
                "mode": mask_mode,
                "terminal_action": if mask_mode == "mark" { "mark_low_confidence" } else { "remove_transition_sites" }
            }
        }),
    )?;
    let damage_filter_counts_json = out_dir.join("damage_filter_counts.json");
    atomic_write_json(
        &damage_filter_counts_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.damage_filter_counts.v1",
            "counts": counts
        }),
    )?;
    let warnings_json = out_dir.join("warnings.json");
    let mut warnings = vec![serde_json::json!({
        "code": "W_VCF_DAMAGE_FILTER_EXPLAINED",
        "message": "damage_filter reports explicit reasons for each filtered category",
        "filtered_and_why": {
            "damage_ratio_exceeded": counts.get("damage_ratio_exceeded").copied().unwrap_or(0),
            "terminal_damage_filtered": counts.get("terminal_damage_filtered").copied().unwrap_or(0),
            "pmd_below_threshold": counts.get("pmd_below_threshold").copied().unwrap_or(0),
            "low_qual": counts.get("low_qual").copied().unwrap_or(0)
        }
    })];
    if proxy_only_mode {
        warnings.push(serde_json::json!({
            "code": "W_VCF_DAMAGE_FILTER_PROXY_ONLY",
            "message": "neither GL/GP/PL nor damage INFO tags were found; proxy transition heuristic was used",
            "required_fields_alignment": {
                "call_gl_format_any_of": ["GL", "GP", "PL"],
                "damage_filter_info_any_of": ["CT_GA_DAMAGE_RATIO", "DEAM5P", "DEAM3P", "PMD_SCORE"]
            }
        }));
    }
    atomic_write_json(
        &warnings_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.damage_filter.warnings.v1",
            "warnings": warnings
        }),
    )?;
    let damage_genotype_manifest_json = out_dir.join("damage_genotype_manifest.json");
    atomic_write_json(
        &damage_genotype_manifest_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.damage_genotype_manifest.v1",
            "udg_regime": params.udg_regime,
            "library_type": library_type,
            "masking_mode": mask_mode,
            "thresholds": {
                "min_qual": params.min_qual,
                "max_damage_ratio": threshold,
                "terminal_damage_threshold": terminal_threshold,
                "pmd_min": pmd_min
            },
            "required_fields_contract": {
                "format_any_of": ["GL", "GP", "PL"],
                "info_any_of": ["CT_GA_DAMAGE_RATIO", "DEAM5P", "DEAM3P", "PMD_SCORE", "PMD", "PMDSCORE"],
                "observed_has_gl_like": has_gl_like_field,
                "observed_has_damage_info": has_damage_info_field
            },
            "counts": counts,
            "asymmetry": {
                "ct_ga_asymmetry": asymmetry
            }
        }),
    )?;

    Ok(DamageFilterOutputs {
        filtered_vcf,
        filtered_tbi,
        damage_filter_summary_json,
        damage_filter_counts_json,
        warnings_json,
        damage_genotype_manifest_json,
    })
}
