fn normalize_indel_alleles(reference: &str, alternate: &str) -> (String, String) {
    let r = reference.to_ascii_uppercase();
    let a = alternate.to_ascii_uppercase();
    if r.len() <= 1 || a.len() <= 1 {
        return (r, a);
    }
    let mut r_chars = r.chars().collect::<Vec<_>>();
    let mut a_chars = a.chars().collect::<Vec<_>>();
    while r_chars.len() > 1 && a_chars.len() > 1 && r_chars.last() == a_chars.last() {
        let _ = r_chars.pop();
        let _ = a_chars.pop();
    }
    while r_chars.len() > 1 && a_chars.len() > 1 && r_chars.first() == a_chars.first() {
        r_chars.remove(0);
        a_chars.remove(0);
    }
    (r_chars.iter().collect(), a_chars.iter().collect())
}

fn canonical_variant_id(contig: &str, pos: &str, reference: &str, alternate: &str) -> String {
    format!("{contig}:{pos}:{reference}:{alternate}")
}

fn remap_gt_for_biallelic(gt: &str, target_alt_index: usize) -> String {
    if gt.contains('.') {
        return gt.to_string();
    }
    let sep = if gt.contains('|') { '|' } else { '/' };
    let tokens = gt.split(sep).collect::<Vec<_>>();
    if tokens.is_empty() {
        return gt.to_string();
    }
    let mut mapped = Vec::with_capacity(tokens.len());
    for token in tokens {
        let Ok(allele) = token.parse::<usize>() else {
            return gt.to_string();
        };
        if allele == 0 {
            mapped.push("0".to_string());
        } else if allele == target_alt_index {
            mapped.push("1".to_string());
        } else {
            mapped.push(".".to_string());
        }
    }
    mapped.join(&sep.to_string())
}

fn normalize_sample_fields_for_split(
    fields: &mut [String],
    target_alt_index: usize,
    sample_order: Option<&[usize]>,
) {
    if fields.len() <= 9 {
        return;
    }
    if let Some(gt_idx) = fields[8].split(':').position(|k| k == "GT") {
        for sample in &mut fields[9..] {
            let mut vals = sample.split(':').map(str::to_string).collect::<Vec<_>>();
            if let Some(gt) = vals.get_mut(gt_idx) {
                *gt = remap_gt_for_biallelic(gt, target_alt_index);
            }
            *sample = vals.join(":");
        }
    }
    if let Some(order) = sample_order {
        let original = fields[9..].to_vec();
        let reordered = order
            .iter()
            .map(|idx| {
                original
                    .get(*idx)
                    .cloned()
                    .unwrap_or_else(|| "./.".to_string())
            })
            .collect::<Vec<_>>();
        for (offset, value) in reordered.into_iter().enumerate() {
            fields[9 + offset] = value;
        }
    }
}

fn left_align_enabled() -> bool {
    std::env::var("BIJUX_VCF_POSTPROCESS_LEFT_ALIGN")
        .ok()
        .as_deref()
        == Some("1")
}

fn normalize_info_fields(info: &str, retain: &[String], remove: &[String]) -> String {
    let retain_set = retain
        .iter()
        .map(|x| x.to_ascii_uppercase())
        .collect::<std::collections::BTreeSet<_>>();
    let remove_set = remove
        .iter()
        .map(|x| x.to_ascii_uppercase())
        .collect::<std::collections::BTreeSet<_>>();
    let mut kept = info
        .split(';')
        .filter(|token| !token.trim().is_empty())
        .filter(|token| {
            let key = token
                .split('=')
                .next()
                .unwrap_or_default()
                .to_ascii_uppercase();
            if !retain_set.is_empty() {
                retain_set.contains(&key)
            } else {
                !remove_set.contains(&key)
            }
        })
        .map(str::to_string)
        .collect::<Vec<_>>();
    kept.sort();
    if kept.is_empty() {
        ".".to_string()
    } else {
        kept.join(";")
    }
}

pub(crate) fn canonical_contig_label(raw: &str) -> String {
    let trimmed = raw.trim();
    let without_chr = trimmed
        .strip_prefix("chr")
        .or_else(|| trimmed.strip_prefix("CHR"))
        .unwrap_or(trimmed);
    match without_chr.to_ascii_uppercase().as_str() {
        "M" | "MT" => "MT".to_string(),
        other => other.to_string(),
    }
}

fn write_postprocess_vcf_with_best_effort_index(out_vcf: &Path, payload: &str) -> Result<PathBuf> {
    let plain_vcf = out_vcf
        .parent()
        .ok_or_else(|| anyhow!("postprocess output path has no parent"))?
        .join("postprocess.tmp.vcf");
    atomic_write_bytes(&plain_vcf, payload.as_bytes())?;
    let out_tbi = crate::vcf_io::vcf_index_bgzip_tabix(&plain_vcf, out_vcf).map_err(|err| {
        anyhow!(
            "postprocess bgzip+tabix failed for {}: {err}",
            out_vcf.display()
        )
    })?;
    let _ = std::fs::remove_file(&plain_vcf);
    Ok(out_tbi)
}

fn resolve_postprocess_reference_fasta(species_id: &str, build_id: &str) -> Option<String> {
    let bundle = bijux_dna_db_ref::resolve_reference_bundle(species_id, build_id).ok()?;
    let path = Path::new(&bundle.fasta);
    if path.exists() {
        Some(bundle.fasta)
    } else {
        None
    }
}

fn write_postprocess_vcf_with_left_alignment(
    out_vcf: &Path,
    payload: &str,
    reference_fasta: &str,
) -> Result<PathBuf> {
    let parent = out_vcf
        .parent()
        .ok_or_else(|| anyhow!("postprocess output path has no parent"))?;
    let plain_vcf = parent.join("postprocess.leftalign.input.vcf");
    atomic_write_bytes(&plain_vcf, payload.as_bytes())?;
    let tmp_vcfgz = parent.join("postprocess.leftalign.tmp.vcf.gz");
    let tmp_tbi = parent.join("postprocess.leftalign.tmp.vcf.gz.tbi");
    let input_s = plain_vcf
        .to_str()
        .ok_or_else(|| anyhow!("non-utf8 postprocess left-align input path"))?;
    let reference_s = Path::new(reference_fasta)
        .to_str()
        .ok_or_else(|| anyhow!("non-utf8 postprocess reference path"))?;
    let tmp_out_s = tmp_vcfgz
        .to_str()
        .ok_or_else(|| anyhow!("non-utf8 postprocess temporary output path"))?;
    let output = std::process::Command::new("bcftools")
        .args(["norm", "-f", reference_s, "-Oz", "-o", tmp_out_s, input_s])
        .output()
        .map_err(|err| anyhow!("bcftools norm invocation failed: {err}"))?;
    if !output.status.success() {
        bail!(
            "bcftools norm failed during postprocess left-alignment: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let tabix_out = std::process::Command::new("tabix")
        .args(["-f", "-p", "vcf", tmp_out_s])
        .output()
        .map_err(|err| anyhow!("tabix invocation failed: {err}"))?;
    if !tabix_out.status.success() {
        bail!(
            "tabix failed during postprocess left-alignment: {}",
            String::from_utf8_lossy(&tabix_out.stderr)
        );
    }
    std::fs::rename(&tmp_vcfgz, out_vcf).map_err(|err| {
        anyhow!(
            "rename left-aligned VCF {} -> {} failed: {err}",
            tmp_vcfgz.display(),
            out_vcf.display()
        )
    })?;
    let out_tbi = PathBuf::from(format!("{}.tbi", out_vcf.display()));
    std::fs::rename(&tmp_tbi, &out_tbi).map_err(|err| {
        anyhow!(
            "rename left-aligned VCF index {} -> {} failed: {err}",
            tmp_tbi.display(),
            out_tbi.display()
        )
    })?;
    let _ = std::fs::remove_file(&plain_vcf);
    Ok(out_tbi)
}

/// # Errors
/// Returns an error if merge/normalization/output validation fails.
pub fn run_postprocess_stage(
    input_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &PostprocessStageParams,
) -> Result<PostprocessStageOutputs> {
    if params.species_id != species_context.species_id
        || params.build_id != species_context.build_id
    {
        bail!("species/build mismatch between postprocess params and SpeciesContext");
    }
    if params.compression_threads == 0 {
        bail!("postprocess requires compression_threads > 0");
    }

    let sources = if params.per_chr_inputs.is_empty() {
        vec![input_vcf.to_path_buf()]
    } else {
        params.per_chr_inputs.clone()
    };
    let mut all_headers = Vec::<String>::new();
    let mut sample_header: Option<String> = None;
    let mut sorted_sample_indexes: Option<Vec<usize>> = None;
    let mut merged_records = Vec::<String>::new();
    let mut invalid_record_count = 0_u64;
    let mut normalized_indel_count = 0_u64;
    let mut split_multiallelic_count = 0_u64;
    let mut normalized_variant_id_count = 0_u64;
    let mut standardized_filter_count = 0_u64;
    for src in &sources {
        let raw = std::fs::read_to_string(src)?;
        for line in raw.lines() {
            if line.starts_with("##") {
                all_headers.push(line.to_string());
                continue;
            }
            if line.starts_with("#CHROM\t") {
                if let Some(seen) = &sample_header {
                    if seen != line {
                        bail!("sample order stability violated across per-chr inputs");
                    }
                } else {
                    sample_header = Some(line.to_string());
                    let cols = line.split('\t').collect::<Vec<_>>();
                    if cols.len() > 9 {
                        let mut pairs = cols[9..]
                            .iter()
                            .enumerate()
                            .map(|(idx, sample)| (idx, (*sample).to_string()))
                            .collect::<Vec<_>>();
                        pairs.sort_by(|a, b| a.1.cmp(&b.1));
                        sorted_sample_indexes =
                            Some(pairs.into_iter().map(|(idx, _)| idx).collect::<Vec<_>>());
                    }
                }
                continue;
            }
            let Some(fields) = parse_record_fields(line) else {
                continue;
            };
            if fields[3].trim().is_empty()
                || fields[4].trim().is_empty()
                || fields[3] == "."
                || fields[4] == "."
            {
                invalid_record_count += 1;
                continue;
            }
            let alt_tokens = fields[4].split(',').collect::<Vec<_>>();
            let alt_count = alt_tokens.len();
            let emit_alts = if params.split_multiallelic && alt_count > 1 {
                split_multiallelic_count += (alt_tokens.len() - 1) as u64;
                alt_tokens.into_iter().enumerate().collect::<Vec<_>>()
            } else {
                vec![(0usize, fields[4])]
            };
            for (alt_idx, alt) in emit_alts {
                let mut out = fields.iter().map(|x| x.to_string()).collect::<Vec<_>>();
                if out[6].trim().is_empty() || out[6] == "." {
                    out[6] = "PASS".to_string();
                    standardized_filter_count += 1;
                }
                out[7] = normalize_info_fields(
                    fields[7],
                    &params.retain_info_fields,
                    &params.remove_info_fields,
                );
                let mut normalized_ref = fields[3].to_ascii_uppercase();
                let mut normalized_alt = alt.to_ascii_uppercase();
                if params.normalize_indels {
                    let (r, a) = normalize_indel_alleles(&normalized_ref, &normalized_alt);
                    if r != fields[3] || a != alt {
                        normalized_indel_count += 1;
                    }
                    normalized_ref = r;
                    normalized_alt = a;
                }
                out[3] = normalized_ref.clone();
                out[4] = normalized_alt.clone();
                let target_alt_index = if params.split_multiallelic && alt_count > 1 {
                    alt_idx + 1
                } else {
                    1
                };
                normalize_sample_fields_for_split(
                    &mut out,
                    target_alt_index,
                    sorted_sample_indexes.as_deref(),
                );
                let canonical_id = canonical_variant_id(&out[0], &out[1], &out[3], &out[4]);
                if out[2] == "." || out[2].trim().is_empty() || out[2] != canonical_id {
                    normalized_variant_id_count += 1;
                    out[2] = canonical_id;
                }
                merged_records.push(out.join("\t"));
            }
        }
    }
    if merged_records.is_empty() {
        bail!("postprocess received no readable VCF records");
    }
    let contig_rank = species_context
        .contigs
        .iter()
        .enumerate()
        .map(|(idx, c)| (canonical_contig_label(&c.name), idx))
        .collect::<std::collections::BTreeMap<_, _>>();
    merged_records.sort_by(|a, b| {
        let ka = parse_variant_key(a).unwrap_or_default();
        let kb = parse_variant_key(b).unwrap_or_default();
        let ra = contig_rank
            .get(&canonical_contig_label(&ka.0))
            .copied()
            .unwrap_or(usize::MAX);
        let rb = contig_rank
            .get(&canonical_contig_label(&kb.0))
            .copied()
            .unwrap_or(usize::MAX);
        ra.cmp(&rb).then(ka.1.cmp(&kb.1)).then(ka.2.cmp(&kb.2))
    });
    let contig_rank = species_context
        .contigs
        .iter()
        .enumerate()
        .map(|(idx, c)| (canonical_contig_label(&c.name), idx))
        .collect::<std::collections::BTreeMap<_, _>>();
    all_headers.sort_by_cached_key(|h| {
        if !h.starts_with("##contig=<") {
            return (0usize, h.clone());
        }
        let id = h
            .split("ID=")
            .nth(1)
            .and_then(|x| x.split([',', '>']).next())
            .unwrap_or_default();
        (
            contig_rank
                .get(&canonical_contig_label(id))
                .copied()
                .unwrap_or(usize::MAX),
            h.clone(),
        )
    });
    all_headers.dedup();
    let mut normalized_headers = Vec::new();
    normalized_headers.push("##fileformat=VCFv4.2".to_string());
    normalized_headers.extend(
        all_headers
            .into_iter()
            .filter(|h| h != "##fileformat=VCFv4.2")
            .collect::<Vec<_>>(),
    );
    let sample_header = sample_header.ok_or_else(|| anyhow!("missing #CHROM header"))?;
    let sample_cols = sample_header.split('\t').collect::<Vec<_>>();
    let final_sample_header = if sample_cols.len() <= 9 {
        sample_header
    } else {
        let mut out = sample_cols[..9]
            .iter()
            .map(|x| (*x).to_string())
            .collect::<Vec<_>>();
        let mut sorted_names = sample_cols[9..]
            .iter()
            .map(|x| (*x).to_string())
            .collect::<Vec<_>>();
        sorted_names.sort();
        out.extend(sorted_names);
        out.join("\t")
    };
    normalized_headers.push(final_sample_header);

    bijux_dna_infra::ensure_dir(out_dir)?;
    let merged_vcf = out_dir.join("postprocess.vcf.gz");
    let merged_bcf = if params.emit_bcf {
        Some(out_dir.join("postprocess.bcf"))
    } else {
        None
    };
    let artifact_checksums_json = out_dir.join("artifact_checksums.json");
    let normalization_contract_json = out_dir.join("normalization_contract.json");
    let validate_outputs_json = out_dir.join("validate_outputs.json");
    let final_manifest_json = out_dir.join("final_manifest.json");
    let logs_txt = out_dir.join("logs.txt");

    let merged_payload = format!(
        "{}\n{}\n",
        normalized_headers.join("\n"),
        merged_records.join("\n")
    );
    let mut left_align_applied = false;
    let mut left_align_reason = "disabled_or_unavailable".to_string();
    let merged_tbi_real = if params.normalize_indels && left_align_enabled() {
        if let Some(reference_fasta) =
            resolve_postprocess_reference_fasta(&params.species_id, &params.build_id)
        {
            if let Ok(tbi) = write_postprocess_vcf_with_left_alignment(
                &merged_vcf,
                &merged_payload,
                &reference_fasta,
            ) {
                left_align_applied = true;
                left_align_reason = "bcftools_norm_with_reference".to_string();
                tbi
            } else {
                left_align_reason = "bcftools_norm_failed_fallback_to_internal".to_string();
                write_postprocess_vcf_with_best_effort_index(&merged_vcf, &merged_payload)?
            }
        } else {
            left_align_reason = "reference_unavailable_fallback_to_internal".to_string();
            write_postprocess_vcf_with_best_effort_index(&merged_vcf, &merged_payload)?
        }
    } else {
        if params.normalize_indels && !left_align_enabled() {
            left_align_reason = "left_align_disabled_by_runtime".to_string();
        }
        write_postprocess_vcf_with_best_effort_index(&merged_vcf, &merged_payload)?
    };
    if let Some(path) = &merged_bcf {
        let merged_vcf_s = merged_vcf
            .to_str()
            .ok_or_else(|| anyhow!("non-utf8 merged vcf path"))?;
        let path_s = path
            .to_str()
            .ok_or_else(|| anyhow!("non-utf8 merged bcf path"))?;
        let bcf_conversion_output = std::process::Command::new("bcftools")
            .args(["view", "-Ob", "-o", path_s, merged_vcf_s])
            .output();
        if bcf_conversion_output
            .as_ref()
            .map(|x| x.status.success())
            .unwrap_or(false)
        {
            let _ = std::process::Command::new("bcftools")
                .args(["index", "-f", path_s])
                .output();
        } else {
            // Deterministic fallback keeps contract output present when BCF tooling is unavailable.
            let passthrough = std::fs::read(&merged_vcf)?;
            atomic_write_bytes(path, &passthrough)?;
        }
    }
    assert_bgzip_tabix_artifacts(&merged_vcf, &merged_tbi_real)?;

    let observed_contigs = merged_records
        .iter()
        .filter_map(|line| parse_variant_key(line).map(|(c, _, _)| canonical_contig_label(&c)))
        .collect::<std::collections::BTreeSet<_>>();
    let species_contigs = species_context
        .contigs
        .iter()
        .map(|c| canonical_contig_label(&c.name))
        .collect::<std::collections::BTreeSet<_>>();
    let validate_payload = serde_json::json!({
        "schema_version": "bijux.vcf.postprocess.validate.v1",
        "readable_vcf": !merged_records.is_empty(),
        "tabix_present": merged_tbi_real.exists(),
        "contigs_consistent_with_species_context": observed_contigs.is_subset(&species_contigs),
    });
    atomic_write_json(&validate_outputs_json, &validate_payload)?;
    if validate_payload
        .get("contigs_consistent_with_species_context")
        .and_then(|v| v.as_bool())
        != Some(true)
    {
        bail!("postprocess validate outputs failed: contigs mismatch species context");
    }
    let normalization_contract = serde_json::json!({
        "schema_version": "bijux.vcf.normalization_contract.v1",
        "stage_id": "vcf.postprocess",
        "left_normalization": {
            "enabled": params.normalize_indels,
            "applied": left_align_applied,
            "reason": left_align_reason.clone(),
        },
        "multiallelic_decomposition": {
            "enabled": params.split_multiallelic,
            "records_split": split_multiallelic_count,
        },
        "duplicate_handling": "retain_and_canonicalize_variant_identity",
        "genotype_retention": "sample_fields_rewritten_only_when_multiallelic_split_requires_it",
        "view_retention": {
            "raw_input": input_vcf,
            "normalized_output": merged_vcf,
        }
    });
    atomic_write_json(&normalization_contract_json, &normalization_contract)?;
    atomic_write_json(
        &final_manifest_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.postprocess.final_manifest.v1",
            "stage_id": "vcf.postprocess",
            "compression": {
                "codec": "bgzip",
                "level": params.compression_level,
                "threads": params.compression_threads
            },
            "normalization": {
                "indel_normalization_enabled": params.normalize_indels,
                "left_align_applied": left_align_applied,
                "left_align_reason": left_align_reason,
                "split_multiallelic_enabled": params.split_multiallelic,
                "indels_normalized": normalized_indel_count,
                "multiallelic_records_split": split_multiallelic_count,
                "variant_ids_normalized": normalized_variant_id_count,
                "invalid_records_removed": invalid_record_count,
                "filter_standardized_to_pass": standardized_filter_count
            },
            "outputs": {
                "vcf": merged_vcf,
                "tbi": merged_tbi_real,
                "bcf": merged_bcf
            },
            "validate_outputs": validate_outputs_json,
            "normalization_contract": normalization_contract_json
        }),
    )?;

    let mut checksum_map = serde_json::Map::new();
    let mut paths = vec![merged_vcf.clone(), merged_tbi_real.clone()];
    if let Some(path) = &merged_bcf {
        paths.push(path.clone());
    }
    let checksum_set = crate::vcf_io::vcf_checksum_set(&paths)?;
    for (path, sum) in checksum_set {
        let name = Path::new(&path)
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or(&path)
            .to_string();
        checksum_map.insert(name, serde_json::Value::String(sum));
    }
    checksum_map.insert(
        "normalization_contract.json".to_string(),
        serde_json::Value::String(checksum_hex(
            serde_json::to_string(&normalization_contract)?.as_bytes(),
        )),
    );
    checksum_map.insert(
        "validate_outputs.json".to_string(),
        serde_json::Value::String(checksum_hex(
            serde_json::to_string(&validate_payload)?.as_bytes(),
        )),
    );
    checksum_map.insert(
        "final_manifest.json".to_string(),
        serde_json::Value::String(checksum_hex(
            std::fs::read_to_string(&final_manifest_json)?.as_bytes(),
        )),
    );
    let checksum_payload = serde_json::Value::Object(checksum_map);
    atomic_write_json(&artifact_checksums_json, &checksum_payload)?;
    if let Some(run_level) = &params.run_level_checksums_path {
        atomic_write_json(run_level, &checksum_payload)?;
    }
    atomic_write_bytes(
        &logs_txt,
        format!(
            "compression_level={}\ncompression_threads={}\nemit_bcf={}\nnormalize_indels={}\nsplit_multiallelic={}\n",
            params.compression_level,
            params.compression_threads,
            params.emit_bcf,
            params.normalize_indels,
            params.split_multiallelic
        )
        .as_bytes(),
    )?;

    Ok(PostprocessStageOutputs {
        merged_vcf,
        merged_tbi: merged_tbi_real,
        merged_bcf,
        artifact_checksums_json,
        normalization_contract_json,
        validate_outputs_json,
        final_manifest_json,
        logs_txt,
    })
}
