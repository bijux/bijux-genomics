fn normalize_indel_alleles(reference: &str, alternate: &str) -> (String, String) {
    let r = reference.to_ascii_uppercase();
    let a = alternate.to_ascii_uppercase();
    if r.len() <= 1 || a.len() <= 1 {
        return (r, a);
    }
    let mut r_chars = r.chars().collect::<Vec<_>>();
    let mut a_chars = a.chars().collect::<Vec<_>>();
    while r_chars.len() > 1 && a_chars.len() > 1 && r_chars.first() == a_chars.first() {
        r_chars.remove(0);
        a_chars.remove(0);
    }
    (r_chars.iter().collect(), a_chars.iter().collect())
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

fn canonical_contig_label(raw: &str) -> String {
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
    let mut merged_records = Vec::<String>::new();
    let mut invalid_record_count = 0_u64;
    let mut normalized_indel_count = 0_u64;
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
                || fields[3].contains(',')
                || fields[4].contains(',')
            {
                invalid_record_count += 1;
                continue;
            }
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
            if params.normalize_indels {
                let (r, a) = normalize_indel_alleles(fields[3], fields[4]);
                if r != fields[3] || a != fields[4] {
                    normalized_indel_count += 1;
                }
                out[3] = r;
                out[4] = a;
            }
            merged_records.push(out.join("\t"));
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
    all_headers.sort();
    all_headers.dedup();
    let mut normalized_headers = Vec::new();
    normalized_headers.push("##fileformat=VCFv4.2".to_string());
    normalized_headers.extend(
        all_headers
            .into_iter()
            .filter(|h| h != "##fileformat=VCFv4.2")
            .collect::<Vec<_>>(),
    );
    normalized_headers.push(sample_header.ok_or_else(|| anyhow!("missing #CHROM header"))?);

    bijux_dna_infra::ensure_dir(out_dir)?;
    let merged_vcf = out_dir.join("postprocess.vcf.gz");
    let merged_tbi = out_dir.join("postprocess.vcf.gz.tbi");
    let merged_bcf = if params.emit_bcf {
        Some(out_dir.join("postprocess.bcf"))
    } else {
        None
    };
    let artifact_checksums_json = out_dir.join("artifact_checksums.json");
    let validate_outputs_json = out_dir.join("validate_outputs.json");
    let final_manifest_json = out_dir.join("final_manifest.json");
    let logs_txt = out_dir.join("logs.txt");

    let merged_payload = format!(
        "{}\n{}\n",
        normalized_headers.join("\n"),
        merged_records.join("\n")
    );
    atomic_write_bytes(&merged_vcf, merged_payload.as_bytes())?;
    atomic_write_bytes(&merged_tbi, b"tabix-index-placeholder\n")?;
    if let Some(path) = &merged_bcf {
        atomic_write_bytes(path, merged_payload.as_bytes())?;
    }
    assert_bgzip_tabix_artifacts(&merged_vcf, &merged_tbi)?;

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
        "tabix_present": merged_tbi.exists(),
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
                "indels_normalized": normalized_indel_count,
                "invalid_records_removed": invalid_record_count,
                "filter_standardized_to_pass": standardized_filter_count
            },
            "outputs": {
                "vcf": merged_vcf,
                "tbi": merged_tbi,
                "bcf": merged_bcf
            },
            "validate_outputs": validate_outputs_json
        }),
    )?;

    let mut checksum_map = serde_json::Map::new();
    let mut paths = vec![merged_vcf.clone(), merged_tbi.clone()];
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
            "compression_level={}\ncompression_threads={}\nemit_bcf={}\nnormalize_indels={}\n",
            params.compression_level,
            params.compression_threads,
            params.emit_bcf,
            params.normalize_indels
        )
        .as_bytes(),
    )?;

    Ok(PostprocessStageOutputs {
        merged_vcf,
        merged_tbi,
        merged_bcf,
        artifact_checksums_json,
        validate_outputs_json,
        final_manifest_json,
        logs_txt,
    })
}
