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
    let panel = resolve_panel(
        &params.species_id,
        &params.build_id,
        params.panel_id.as_deref(),
    )?;
    let map = resolve_map(
        &params.species_id,
        &params.build_id,
        params.map_id.as_deref(),
    )?;
    if panel.species_id != species_context.species_id || panel.build_id != species_context.build_id
    {
        bail!("panel species/build does not match SpeciesContext");
    }
    if map.species_id != species_context.species_id || map.build_id != species_context.build_id {
        bail!("map species/build does not match SpeciesContext");
    }

    let panel_parent = panel_vcf
        .parent()
        .ok_or_else(|| anyhow!("panel path has no parent: {}", panel_vcf.display()))?;
    if panel_parent.file_name().and_then(|x| x.to_str()) != Some("raw") {
        bail!(
            "panel materialization refusal: panel must be acquired via scripts/tooling/acquire-panels.sh and live under .../raw/"
        );
    }
    let source_panel_root = panel_parent
        .parent()
        .ok_or_else(|| anyhow!("panel raw path missing panel root"))?;
    if !source_panel_root.join("normalized").exists() || !source_panel_root.join("derived").exists()
    {
        bail!(
            "panel materialization refusal: expected sibling normalized/derived dirs from acquire-panels materialization"
        );
    }

    let input_raw = std::fs::read_to_string(input_vcf)?;
    let panel_raw = std::fs::read_to_string(panel_vcf)?;
    let mut input_keys = std::collections::BTreeSet::<String>::new();
    let mut panel_by_chr = std::collections::BTreeMap::<String, u64>::new();
    let mut overlap_by_chr = std::collections::BTreeMap::<String, u64>::new();
    for line in input_raw.lines() {
        if let Some(fields) = parse_record_fields(line) {
            if let Some((_chr, key)) = variant_key(&fields) {
                input_keys.insert(key);
            }
        }
    }
    for line in panel_raw.lines() {
        if let Some(fields) = parse_record_fields(line) {
            if let Some((chr, key)) = variant_key(&fields) {
                *panel_by_chr.entry(chr.clone()).or_insert(0) += 1;
                if input_keys.contains(&key) {
                    *overlap_by_chr.entry(chr).or_insert(0) += 1;
                }
            }
        }
    }
    let panel_total: u64 = panel_by_chr.values().sum();
    let overlap_total: u64 = overlap_by_chr.values().sum();
    let overlap_fraction = if panel_total == 0 {
        0.0
    } else {
        overlap_total as f64 / panel_total as f64
    };

    std::fs::create_dir_all(out_dir)?;
    let lock_seed = format!(
        "{}|{}|{}",
        panel.id, panel.version, panel.build_id
    );
    let lock_hash = checksum_hex(lock_seed.as_bytes());
    let panel_root = out_dir
        .join("panels")
        .join(panel.id.clone())
        .join(lock_hash);
    let local_raw = panel_root.join("raw");
    let local_normalized = panel_root.join("normalized");
    let local_derived = panel_root.join("derived");
    std::fs::create_dir_all(&local_raw)?;
    std::fs::create_dir_all(&local_normalized)?;
    std::fs::create_dir_all(&local_derived)?;

    let local_raw_panel_vcf = local_raw.join(
        panel_vcf
            .file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("panel.vcf.gz"),
    );
    atomic_write_bytes(&local_raw_panel_vcf, &std::fs::read(panel_vcf)?)?;

    let prepared_panel_vcf = local_normalized.join("prepared_panel.vcf.gz");
    let prepared_panel_tbi = local_normalized.join("prepared_panel.vcf.gz.tbi");
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
        let ra = contig_rank
            .get(&canonical_contig_label(&ka.0))
            .copied()
            .unwrap_or(usize::MAX);
        let rb = contig_rank
            .get(&canonical_contig_label(&kb.0))
            .copied()
            .unwrap_or(usize::MAX);
        ra.cmp(&rb)
            .then(ka.1.cmp(&kb.1))
            .then(ka.2.cmp(&kb.2))
    });
    let normalized_payload = format!("{}\n{}\n", header_lines.join("\n"), record_lines.join("\n"));
    atomic_write_bytes(&prepared_panel_vcf, normalized_payload.as_bytes())?;
    atomic_write_bytes(&prepared_panel_tbi, b"tabix-index-placeholder\n")?;
    assert_bgzip_tabix_artifacts(&prepared_panel_vcf, &prepared_panel_tbi)?;

    let site_list = local_derived.join("panel_sites.tsv");
    let mut site_rows = String::from("contig\tpos\tref\talt\n");
    for rec in &record_lines {
        if let Some(fields) = parse_record_fields(rec) {
            site_rows.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                fields[0], fields[1], fields[3], fields[4]
            ));
        }
    }
    atomic_write_bytes(&site_list, site_rows.as_bytes())?;
    let chunk_regions = local_derived.join("chunk_regions.tsv");
    atomic_write_bytes(
        &chunk_regions,
        b"chunk_id\tregion\nchunk_000\t1:1-1000000\n",
    )?;
    if panel.compatibility.supports_minimac_m3vcf {
        let minimac_ready = local_derived.join("minimac.m3vcf.ready");
        atomic_write_bytes(&minimac_ready, b"true\n")?;
    }

    let manifest = serde_json::json!({
        "schema_version": "bijux.vcf.prepare_reference_panel.manifest.v1",
        "species_id": params.species_id,
        "build_id": params.build_id,
        "panel_root": panel_root,
        "source_panel_root": source_panel_root,
        "panel": {
            "id": panel.id,
            "version": panel.version,
            "file_count": panel.files.len(),
            "compatibility": panel.compatibility,
        },
        "map": {
            "id": map.id,
            "version": map.version,
            "file_count": map.files.len(),
            "compatibility": map.compatibility,
        }
    });
    atomic_write_json(&panel_manifest_json, &manifest)?;
    let per_chr = panel_by_chr
        .iter()
        .map(|(chr, total)| {
            let overlap = *overlap_by_chr.get(chr).unwrap_or(&0);
            let frac = if *total == 0 {
                0.0
            } else {
                overlap as f64 / *total as f64
            };
            serde_json::json!({
                "chr": chr,
                "panel_sites": total,
                "overlap_sites": overlap,
                "overlap_fraction": frac,
            })
        })
        .collect::<Vec<_>>();
    let overlap_payload = serde_json::json!({
        "schema_version": "bijux.vcf.prepare_reference_panel.overlap.v1",
        "global": {
            "panel_sites": panel_total,
            "overlap_sites": overlap_total,
            "overlap_fraction": overlap_fraction,
        },
        "per_chr": per_chr,
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
    let mut tsv = String::from("chr\tpanel_sites\toverlap_sites\toverlap_fraction\n");
    for (chr, total) in &panel_by_chr {
        let overlap = *overlap_by_chr.get(chr).unwrap_or(&0);
        let frac = if *total == 0 {
            0.0
        } else {
            overlap as f64 / *total as f64
        };
        tsv.push_str(&format!("{chr}\t{total}\t{overlap}\t{frac:.6}\n"));
    }
    atomic_write_bytes(&overlap_tsv, tsv.as_bytes())?;

    let chunk_plan = plan_regions_deterministic(species_context, &ChunkingPlanParams::default())?;
    let chunk_rows = chunk_plan
        .iter()
        .map(|c| {
            let panel_sites = *panel_by_chr.get(&c.contig).unwrap_or(&0);
            let overlap_sites = *overlap_by_chr.get(&c.contig).unwrap_or(&0);
            let overlap_fraction = if panel_sites == 0 {
                0.0
            } else {
                overlap_sites as f64 / panel_sites as f64
            };
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

