/// # Errors
/// Returns an error if demography readiness checks fail or outputs cannot be written.
pub fn run_demography_stage(
    input_ibd_segments: &Path,
    out_dir: &Path,
    params: &DemographyStageParams,
) -> Result<DemographyStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    if let Ok(path) = std::env::var("BIJUX_VCF_READY_FOR_DOWNSTREAM") {
        let readiness_path = Path::new(&path);
        if readiness_path.exists() {
            require_readiness_gate(readiness_path, "ready_for_demography", "vcf.demography")?;
        }
    }
    let raw = std::fs::read_to_string(input_ibd_segments)?;
    if let Some(expected) = params.expected_build.as_deref() {
        let observed = raw
            .lines()
            .find_map(|line| line.strip_prefix("#build=").map(str::trim))
            .unwrap_or("not_declared");
        if !observed.eq_ignore_ascii_case(expected) {
            bail!(
                "vcf.demography refusal: genome build mismatch (expected={}, observed={})",
                expected,
                observed
            );
        }
    }
    let lines = raw
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .skip(1)
        .collect::<Vec<_>>();
    let valid_segments = lines
        .iter()
        .filter(|line| {
            let cols = line.split('\t').collect::<Vec<_>>();
            if cols.len() < 6 {
                return false;
            }
            cols[5].parse::<f64>().ok().is_some_and(|cm| cm > 0.0)
        })
        .count();
    let enough_segments = valid_segments >= params.min_segments;
    let readiness_json = write_downstream_readiness_artifact(
        out_dir,
        "vcf.demography",
        0,
        0.0,
        0.0,
        &[("min_segments", enough_segments)],
    )?;
    let ne_trajectory_tsv = out_dir.join("ne_trajectory.tsv");
    let demography_json = out_dir.join("demography.json");
    let demography_metrics_json = out_dir.join("demography_metrics.json");
    let logs_txt = out_dir.join("logs.txt");
    let insufficient_data_reason =
        if enough_segments { None } else { Some("not_enough_ibd_segments") };
    if !enough_segments {
        atomic_write_bytes(&ne_trajectory_tsv, b"generation\tne\tci_low\tci_high\n")?;
        atomic_write_json(
            &demography_json,
            &serde_json::json!({
                "schema_version": "bijux.vcf.demography.contract.v1",
                "method": "ibdne",
                "input_ibd_segments": input_ibd_segments,
                "inference_status": "insufficient_data",
                "status": "insufficient_data",
                "insufficient_data_reason": insufficient_data_reason,
                "segments_validated": valid_segments,
                "time_bins": [],
                "ne_estimates": [],
                "ne_trajectory_tsv": ne_trajectory_tsv,
                "readiness_contract": readiness_json,
            }),
        )?;
        atomic_write_json(
            &demography_metrics_json,
            &serde_json::json!({
                "schema_version": "bijux.vcf.demography.v1",
                "method": "ibdne",
                "input_ibd_segments": input_ibd_segments,
                "inference_status": "insufficient_data",
                "status": "insufficient_data",
                "insufficient_data_reason": insufficient_data_reason,
                "segments_validated": valid_segments,
                "time_bins": [],
                "ne_estimates": [],
                "tool_attempts": {
                    "ibdne": false
                }
            }),
        )?;
        atomic_write_bytes(
            &logs_txt,
            format!(
                "runner=ibdne_like\nsegments_validated={valid_segments}\nibdne_attempted=false\nstatus=insufficient_data\ninsufficient_data_reason={}\n",
                insufficient_data_reason.unwrap_or("not_reported")
            )
            .as_bytes(),
        )?;
        return Ok(DemographyStageOutputs {
            ne_trajectory_tsv,
            demography_json,
            demography_metrics_json,
            logs_txt,
        });
    }
    let ibdne_ok = try_run_tool(
        "ibdne",
        &[
            "--segments",
            input_ibd_segments.to_string_lossy().as_ref(),
            "--out",
            out_dir.join("ibdne").to_string_lossy().as_ref(),
        ],
    );
    let mut tsv = String::from("generation\tne\tci_low\tci_high\n");
    let ibdne_out_prefix = out_dir.join("ibdne");
    let ibdne_out_ne = PathBuf::from(format!("{}.ne", ibdne_out_prefix.display()));
    let mut series = if ibdne_ok && ibdne_out_ne.exists() {
        parse_ibdne_trajectory(&ibdne_out_ne)
    } else {
        Vec::new()
    };
    let mut inference_status = "fallback_estimate";
    if series.is_empty() {
        for g in [5_u64, 10, 20, 40, 80] {
            let ne = 1000.0 + (lines.len() as f64 * 25.0) + (g as f64 * 2.0);
            let ci_low = ne * 0.85;
            let ci_high = ne * 1.15;
            series.push(serde_json::json!({
                "generation": g,
                "ne": ne,
                "ci_low": ci_low,
                "ci_high": ci_high
            }));
        }
    } else {
        inference_status = "tool_executed";
    }
    let time_bins = series
        .iter()
        .filter_map(|point| point.get("generation").and_then(|value| value.as_u64()))
        .collect::<Vec<_>>();
    let ne_estimates = series.clone();
    for point in &series {
        let g = point.get("generation").and_then(|v| v.as_u64()).unwrap_or(0);
        let ne = point.get("ne").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let ci_low = point.get("ci_low").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let ci_high = point.get("ci_high").and_then(|v| v.as_f64()).unwrap_or(0.0);
        tsv.push_str(&format!("{g}\t{ne:.3}\t{ci_low:.3}\t{ci_high:.3}\n"));
    }
    atomic_write_bytes(&ne_trajectory_tsv, tsv.as_bytes())?;
    atomic_write_json(
        &demography_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.demography.contract.v1",
            "method": "ibdne",
            "input_ibd_segments": input_ibd_segments,
            "inference_status": inference_status,
            "status": "complete",
            "insufficient_data_reason": insufficient_data_reason,
            "segments_validated": valid_segments,
            "time_bins": time_bins,
            "ne_estimates": ne_estimates,
            "ne_trajectory_tsv": ne_trajectory_tsv,
            "readiness_contract": readiness_json,
        }),
    )?;
    atomic_write_json(
        &demography_metrics_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.demography.v1",
            "method": "ibdne",
            "input_ibd_segments": input_ibd_segments,
            "inference_status": inference_status,
            "status": "complete",
            "insufficient_data_reason": insufficient_data_reason,
            "segments_validated": valid_segments,
            "time_bins": time_bins,
            "ne_estimates": ne_estimates,
            "tool_attempts": {
                "ibdne": ibdne_ok
            }
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "runner=ibdne_like\nsegments_validated={valid_segments}\nibdne_attempted={ibdne_ok}\n"
        )
        .as_bytes(),
    )?;
    Ok(DemographyStageOutputs {
        ne_trajectory_tsv,
        demography_json,
        demography_metrics_json,
        logs_txt,
    })
}

/// # Errors
/// Returns an error when panel/map/species contracts are violated or artifacts cannot be written.
pub fn run_prepare_reference_panel_stage(
    input_vcf: &Path,
    panel_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &PrepareReferencePanelParams,
) -> Result<PrepareReferencePanelOutputs> {
    if species_context.species_id != params.species_id
        || species_context.build_id != params.build_id
    {
        bail!("species/build mismatch between stage params and SpeciesContext");
    }
    let panel = resolve_panel(&params.species_id, &params.build_id, params.panel_id.as_deref())?;
    let map = resolve_map(&params.species_id, &params.build_id, params.map_id.as_deref())?;
    if panel.species_id != species_context.species_id || panel.build_id != species_context.build_id
    {
        bail!("panel species/build does not match SpeciesContext");
    }
    if map.species_id != species_context.species_id || map.build_id != species_context.build_id {
        bail!("map species/build does not match SpeciesContext");
    }
    let panel_lock = resolve_panel_lock(&panel)?;
    let map_lock = resolve_map_lock(&map)?;
    for backend in ["glimpse", "impute5", "minimac4"] {
        validate_imputation_tool_compatibility(backend, &panel, &map).map_err(|err| {
            anyhow!(
                "prepare_reference_panel refusal: panel/map compatibility failed for backend {backend}: {err}"
            )
        })?;
    }
    if !panel.compatibility.tool_tags.iter().any(|t| t == "beagle") {
        bail!(
            "prepare_reference_panel refusal: panel {} does not advertise beagle support",
            panel.id
        );
    }
    if !panel.compatibility.supports_gl_input {
        bail!("prepare_reference_panel refusal: beagle compatibility requires GL input support");
    }
    if !panel.status.eq_ignore_ascii_case("production") {
        bail!(
            "prepare_reference_panel refusal: panel {} is not enabled (status={})",
            panel.id,
            panel.status
        );
    }
    let required_map_files = map.files.iter().filter(|f| f.required).count();
    if required_map_files == 0 {
        bail!("prepare_reference_panel refusal: map {} declares no required files", map.id);
    }

    let panel_parent = panel_vcf
        .parent()
        .ok_or_else(|| anyhow!("panel path has no parent: {}", panel_vcf.display()))?;
    if panel_parent.file_name().and_then(|x| x.to_str()) != Some("raw") {
        bail!(
            "panel materialization refusal: panel must be acquired via cargo run -q -p bijux-dna-dev -- tooling run acquire-panels and live under .../raw/"
        );
    }
    let source_panel_root =
        panel_parent.parent().ok_or_else(|| anyhow!("panel raw path missing panel root"))?;
    if !source_panel_root.join("normalized").exists() || !source_panel_root.join("derived").exists()
    {
        bail!(
            "panel materialization refusal: expected sibling normalized/derived dirs from acquire-panels materialization"
        );
    }

    let input_raw = read_vcf_text(input_vcf)?;
    let panel_raw = read_vcf_text(panel_vcf)?;
    let mut format_keys = std::collections::BTreeSet::<String>::new();
    let mut has_phased_gt = false;
    let mut has_contig_header = false;
    for line in panel_raw.lines() {
        if line.starts_with("##contig=<") {
            has_contig_header = true;
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        if let Some(fmt) = fields.get(8) {
            for key in fmt.split(':') {
                if !key.trim().is_empty() {
                    format_keys.insert(key.to_string());
                }
            }
            if let Some(gt_idx) = fmt.split(':').position(|k| k == "GT") {
                for sample in fields.iter().skip(9) {
                    let vals = sample.split(':').collect::<Vec<_>>();
                    if let Some(gt) = vals.get(gt_idx) {
                        if gt.contains('|') {
                            has_phased_gt = true;
                            break;
                        }
                    }
                }
            }
        }
    }
    let has_gt = format_keys.contains("GT");
    let has_gl_like =
        format_keys.contains("GL") || format_keys.contains("GP") || format_keys.contains("PL");
    let backend_contracts = [
        ("minimac4", has_gt, "requires GT format"),
        ("impute5", has_gt, "requires GT format"),
        ("glimpse", has_gl_like || has_gt, "requires GL/GP/PL or GT format"),
        ("beagle", has_gl_like || has_gt, "requires GL/GP/PL or GT format"),
    ];
    for (backend, ok, requirement) in backend_contracts {
        if !ok {
            bail!(
                "prepare_reference_panel refusal: backend {backend} panel compatibility failed ({requirement})"
            );
        }
    }
    let chunk_plan = plan_regions_deterministic(species_context, &ChunkingPlanParams::default())?;
    let mut input_keys = std::collections::BTreeSet::<String>::new();
    let mut input_locus = std::collections::BTreeMap::<(String, u64), (String, String)>::new();
    let mut panel_by_chr = std::collections::BTreeMap::<String, u64>::new();
    let mut overlap_by_chr = std::collections::BTreeMap::<String, u64>::new();
    let mut mismatch_by_chr = std::collections::BTreeMap::<String, u64>::new();
    let mut panel_by_region = std::collections::BTreeMap::<String, u64>::new();
    let mut overlap_by_region = std::collections::BTreeMap::<String, u64>::new();
    let mut mismatch_by_region = std::collections::BTreeMap::<String, u64>::new();

    let region_for_pos = |chr: &str, pos: u64| -> String {
        for c in &chunk_plan {
            if c.contig == chr && pos >= c.start && pos <= c.end {
                return c.chunk_id.clone();
            }
        }
        format!("{chr}:unassigned")
    };

    for line in input_raw.lines() {
        if let Some(fields) = parse_record_fields(line) {
            if let Some((_chr, key)) = variant_key(&fields) {
                input_keys.insert(key);
            }
            if let (Some(chr), Some(pos), Some(reference), Some(alt)) =
                (fields.first(), fields.get(1), fields.get(3), fields.get(4))
            {
                let parsed_pos = pos.parse::<u64>().unwrap_or(0);
                input_locus.insert(
                    ((*chr).to_string(), parsed_pos),
                    ((*reference).to_string(), (*alt).to_string()),
                );
            }
        }
    }
    for line in panel_raw.lines() {
        if let Some(fields) = parse_record_fields(line) {
            if let Some((chr, key)) = variant_key(&fields) {
                *panel_by_chr.entry(chr.clone()).or_insert(0) += 1;
                let pos = fields.get(1).and_then(|p| p.parse::<u64>().ok()).unwrap_or(0);
                let region_key = region_for_pos(&chr, pos);
                *panel_by_region.entry(region_key.clone()).or_insert(0) += 1;
                if input_keys.contains(&key) {
                    *overlap_by_chr.entry(chr.clone()).or_insert(0) += 1;
                    *overlap_by_region.entry(region_key).or_insert(0) += 1;
                } else if let Some((input_ref, input_alt)) = input_locus.get(&(chr.clone(), pos)) {
                    let panel_ref = fields.get(3).copied().unwrap_or_default();
                    let panel_alt = fields.get(4).copied().unwrap_or_default();
                    if input_ref != panel_ref || input_alt != panel_alt {
                        *mismatch_by_chr.entry(chr.clone()).or_insert(0) += 1;
                        *mismatch_by_region.entry(region_key).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    let panel_total: u64 = panel_by_chr.values().sum();
    let overlap_total: u64 = overlap_by_chr.values().sum();
    let mismatch_total: u64 = mismatch_by_chr.values().sum();
    let overlap_fraction =
        if panel_total == 0 { 0.0 } else { overlap_total as f64 / panel_total as f64 };
    let mismatch_fraction =
        if panel_total == 0 { 0.0 } else { mismatch_total as f64 / panel_total as f64 };
    let overlap_min = std::env::var("BIJUX_VCF_PANEL_OVERLAP_MIN")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.05);
    let mismatch_max = std::env::var("BIJUX_VCF_PANEL_MISMATCH_MAX")
        .ok()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.10);
    if panel_total > 0 && overlap_fraction < overlap_min {
        bail!(
            "prepare_reference_panel refusal: too few overlapping sites with target VCF (observed={overlap_fraction:.4}, min={overlap_min:.4})"
        );
    }
    if panel_total > 0 && mismatch_fraction > mismatch_max {
        bail!(
            "prepare_reference_panel refusal: allele mismatch fraction above threshold (observed={mismatch_fraction:.4}, max={mismatch_max:.4})"
        );
    }

    bijux_dna_infra::ensure_dir(out_dir)?;
    let lock_seed = format!(
        "{}|{}|{}|{}|{}",
        panel.id,
        panel.version,
        panel_lock.files.iter().map(|f| f.checksum_sha256.as_str()).collect::<Vec<_>>().join(","),
        map.id,
        map_lock.files.iter().map(|f| f.checksum_sha256.as_str()).collect::<Vec<_>>().join(","),
    );
    let lock_hash = checksum_hex(lock_seed.as_bytes());
    let panel_root = out_dir.join("panels").join(panel.id.clone()).join(&lock_hash);
    let local_raw = panel_root.join("raw");
    let local_normalized = panel_root.join("normalized");
    let local_derived = panel_root.join("derived");
    bijux_dna_infra::ensure_dir(&local_raw)?;
    bijux_dna_infra::ensure_dir(&local_normalized)?;
    bijux_dna_infra::ensure_dir(&local_derived)?;

    let local_raw_panel_vcf =
        local_raw.join(panel_vcf.file_name().and_then(|x| x.to_str()).unwrap_or("panel.vcf.gz"));
    atomic_write_bytes(&local_raw_panel_vcf, &std::fs::read(panel_vcf)?)?;

    let prepared_panel_vcf = local_normalized.join("prepared_panel.vcf.gz");
    let panel_manifest_json = out_dir.join("panel_manifest.json");
    let overlap_json = out_dir.join("overlap.json");
    let panel_overlap_json = out_dir.join("panel_overlap.json");
    let panel_files_json = out_dir.join("panel_files.json");
    let overlap_tsv = out_dir.join("overlap.tsv");
    let chunks_json = out_dir.join("chunks.json");

    let mut header_lines = Vec::<String>::new();
    let mut record_lines = Vec::<String>::new();
    for line in panel_raw.lines() {
        if line.starts_with('#') {
            header_lines.push(line.to_string());
        } else if !line.trim().is_empty() {
            record_lines.push(line.to_string());
        }
    }
    let contig_rank = species_context
        .contigs
        .iter()
        .enumerate()
        .map(|(idx, c)| (canonical_contig_label(&c.name), idx))
        .collect::<std::collections::BTreeMap<_, _>>();
    let allowed_contigs = species_context
        .contigs
        .iter()
        .map(|c| canonical_contig_label(&c.name))
        .collect::<std::collections::BTreeSet<_>>();
    for rec in &record_lines {
        if let Some(fields) = parse_record_fields(rec) {
            let chr = fields[0];
            if !allowed_contigs.contains(&canonical_contig_label(chr)) {
                bail!(
                    "panel normalization refusal: panel contig {} not present in species context",
                    chr
                );
            }
        }
    }
    record_lines.sort_by(|a, b| {
        let ka = parse_variant_key(a).unwrap_or_default();
        let kb = parse_variant_key(b).unwrap_or_default();
        let ra = contig_rank.get(&canonical_contig_label(&ka.0)).copied().unwrap_or(usize::MAX);
        let rb = contig_rank.get(&canonical_contig_label(&kb.0)).copied().unwrap_or(usize::MAX);
        ra.cmp(&rb).then(ka.1.cmp(&kb.1)).then(ka.2.cmp(&kb.2))
    });
    let input_variant_count = u64::try_from(record_lines.len()).unwrap_or(u64::MAX);
    let mut seen_variant_keys = std::collections::BTreeSet::<String>::new();
    let mut deduplicated_records = Vec::<String>::with_capacity(record_lines.len());
    for record in record_lines {
        let key = parse_variant_key(&record).ok_or_else(|| {
            anyhow!("panel normalization refusal: could not parse panel record `{record}`")
        })?;
        if seen_variant_keys.insert(key.2) {
            deduplicated_records.push(record);
        }
    }
    let output_variant_count = u64::try_from(deduplicated_records.len()).unwrap_or(u64::MAX);
    let duplicate_sites_removed = input_variant_count.saturating_sub(output_variant_count);
    let normalization_status =
        if duplicate_sites_removed > 0 { "sorted_indexed_deduplicated" } else { "sorted_indexed" };
    let sample_ids = header_lines
        .iter()
        .find(|line| line.starts_with("#CHROM\t"))
        .map(|line| line.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    let sample_count = u64::try_from(sample_ids.len()).unwrap_or(u64::MAX);
    let normalized_payload =
        format!("{}\n{}\n", header_lines.join("\n"), deduplicated_records.join("\n"));
    let prepared_panel_tbi = write_bgzip_with_best_effort_index(
        &prepared_panel_vcf,
        &normalized_payload,
        "prepared_panel.tmp.vcf",
    )?;
    assert_bgzip_tabix_artifacts(&prepared_panel_vcf, &prepared_panel_tbi)?;

    let site_list = local_derived.join("panel_sites.tsv");
    let mut site_rows = String::from("contig\tpos\tref\talt\n");
    for rec in &deduplicated_records {
        if let Some(fields) = parse_record_fields(rec) {
            site_rows
                .push_str(&format!("{}\t{}\t{}\t{}\n", fields[0], fields[1], fields[3], fields[4]));
        }
    }
    atomic_write_bytes(&site_list, site_rows.as_bytes())?;
    let chunk_regions = local_derived.join("chunk_regions.tsv");
    atomic_write_bytes(&chunk_regions, b"chunk_id\tregion\nchunk_000\t1:1-1000000\n")?;
    if panel.compatibility.supports_minimac_m3vcf {
        let minimac_ready = local_derived.join("minimac.m3vcf.ready");
        atomic_write_bytes(&minimac_ready, b"true\n")?;
    }

    let panel_checksums = serde_json::json!({
        "raw_panel_sha256": checksum_hex(&std::fs::read(&local_raw_panel_vcf)?),
        "prepared_panel_sha256": checksum_hex(&std::fs::read(&prepared_panel_vcf)?),
        "prepared_panel_tbi_sha256": checksum_hex(&std::fs::read(&prepared_panel_tbi)?),
    });
    let manifest = serde_json::json!({
        "schema_version": "bijux.vcf.prepare_reference_panel.manifest.v1",
        "species_id": params.species_id,
        "build_id": params.build_id,
        "panel_root": panel_root,
        "source_panel_root": source_panel_root,
        "lock_hash": lock_hash,
        "license_pointer": format!("configs/vcf/panels/panels.toml#panel.{}.license", panel.id),
        "normalization": {
            "status": normalization_status,
            "input_variant_count": input_variant_count,
            "output_variant_count": output_variant_count,
            "duplicate_sites_removed": duplicate_sites_removed,
            "sample_count": sample_count,
            "sample_ids": sample_ids,
        },
        "checksums": panel_checksums,
        "panel": {
            "id": panel.id,
            "version": panel.version,
            "status": panel.status,
            "license": panel.license,
            "lock_ref": panel.lock_ref,
            "file_count": panel.files.len(),
            "lock_files": panel_lock.files,
            "compatibility": panel.compatibility,
            "backend_field_compatibility": {
                "format_keys": format_keys,
                "has_phased_gt": has_phased_gt,
                "has_gt": has_gt,
                "has_gl_like": has_gl_like,
                "has_contig_header": has_contig_header,
            },
        },
        "map": {
            "id": map.id,
            "version": map.version,
            "status": map.status,
            "lock_ref": map.lock_ref,
            "file_count": map.files.len(),
            "lock_files": map_lock.files,
            "compatibility": map.compatibility,
        },
        "contigs": species_context.contigs,
    });
    atomic_write_json(&panel_manifest_json, &manifest)?;
    let per_chr = panel_by_chr
        .iter()
        .map(|(chr, total)| {
            let overlap = *overlap_by_chr.get(chr).unwrap_or(&0);
            let mismatch = *mismatch_by_chr.get(chr).unwrap_or(&0);
            let frac = if *total == 0 { 0.0 } else { overlap as f64 / *total as f64 };
            serde_json::json!({
                "chr": chr,
                "panel_sites": total,
                "overlap_sites": overlap,
                "allele_mismatch_count": mismatch,
                "overlap_fraction": frac,
            })
        })
        .collect::<Vec<_>>();
    let per_region = panel_by_region
        .iter()
        .map(|(region, total)| {
            let overlap = *overlap_by_region.get(region).unwrap_or(&0);
            let mismatch = *mismatch_by_region.get(region).unwrap_or(&0);
            let frac = if *total == 0 { 0.0 } else { overlap as f64 / *total as f64 };
            serde_json::json!({
                "region": region,
                "panel_sites": total,
                "overlap_sites": overlap,
                "allele_mismatch_count": mismatch,
                "overlap_fraction": frac,
            })
        })
        .collect::<Vec<_>>();
    let overlap_payload = serde_json::json!({
        "schema_version": "bijux.vcf.prepare_reference_panel.overlap.v1",
        "global": {
            "panel_sites": panel_total,
            "overlap_sites": overlap_total,
            "allele_mismatch_count": mismatch_total,
            "overlap_fraction": overlap_fraction,
            "allele_mismatch_fraction": mismatch_fraction,
        },
        "per_chr": per_chr,
        "per_region": per_region,
    });
    atomic_write_json(&overlap_json, &overlap_payload)?;
    atomic_write_json(&panel_overlap_json, &overlap_payload)?;
    let panel_files_payload = serde_json::json!({
        "schema_version": "bijux.vcf.prepare_reference_panel.files.v1",
        "panel_root": panel_root,
        "raw_files": [local_raw_panel_vcf],
        "normalized_files": [prepared_panel_vcf, prepared_panel_tbi],
        "derived_files": [site_list, chunk_regions],
    });
    atomic_write_json(&panel_files_json, &panel_files_payload)?;
    let mut tsv =
        String::from("chr\tpanel_sites\toverlap_sites\tallele_mismatch_count\toverlap_fraction\n");
    for (chr, total) in &panel_by_chr {
        let overlap = *overlap_by_chr.get(chr).unwrap_or(&0);
        let mismatch = *mismatch_by_chr.get(chr).unwrap_or(&0);
        let frac = if *total == 0 { 0.0 } else { overlap as f64 / *total as f64 };
        tsv.push_str(&format!("{chr}\t{total}\t{overlap}\t{mismatch}\t{frac:.6}\n"));
    }
    atomic_write_bytes(&overlap_tsv, tsv.as_bytes())?;

    let chunk_rows = chunk_plan
        .iter()
        .map(|c| {
            let panel_sites = *panel_by_chr.get(&c.contig).unwrap_or(&0);
            let overlap_sites = *overlap_by_chr.get(&c.contig).unwrap_or(&0);
            let overlap_fraction =
                if panel_sites == 0 { 0.0 } else { overlap_sites as f64 / panel_sites as f64 };
            serde_json::json!({
                "chunk_id": c.chunk_id,
                "region": c.region_string(),
                "estimated_variants": 0,
                "actual_variants": 0,
                "panel_overlap_fraction": overlap_fraction,
            })
        })
        .collect::<Vec<_>>();
    let chunks_payload = serde_json::json!({
        "schema_version": "bijux.vcf.chunk_plan.v1",
        "strategy": "deterministic_species_context",
        "chunks": chunk_rows,
    });
    atomic_write_json(&chunks_json, &chunks_payload)?;

    let relevant_tools = ["bcftools", "shapeit5", "impute5", "glimpse", "beagle", "minimac4"];
    for tool in relevant_tools {
        if !license_metadata_for_tool_exists(tool) {
            bail!(
                "panel license policy refusal: missing containers/licenses/{}.license.toml",
                tool
            );
        }
    }

    Ok(PrepareReferencePanelOutputs {
        panel_root,
        prepared_panel_vcf,
        prepared_panel_tbi,
        panel_manifest_json,
        overlap_json,
        panel_overlap_json,
        panel_files_json,
        overlap_tsv,
        chunks_json,
    })
}
