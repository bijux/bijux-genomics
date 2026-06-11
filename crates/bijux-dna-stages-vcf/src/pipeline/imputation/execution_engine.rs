pub(crate) fn run_impute_stage_inner(
    input_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &ImputeStageParams,
) -> Result<ImputeStageOutputs> {
    if params.species_id != species_context.species_id
        || params.build_id != species_context.build_id
    {
        bail!("species/build mismatch between impute params and SpeciesContext");
    }
    let domain_guard = params.species_id.to_ascii_lowercase();
    if domain_guard.contains("edna") || domain_guard.contains("pollen") {
        bail!("impute stage refusal: non-vcf domain inputs are not supported");
    }
    if params.threads == 0 {
        bail!("impute requires threads > 0");
    }

    let panel = resolve_panel(&params.species_id, &params.build_id, params.panel_id.as_deref())?;
    let run_started = std::time::Instant::now();
    let raw = if input_vcf
        .extension()
        .and_then(|x| x.to_str())
        .is_some_and(|x| x == "gz" || x == "bcf")
    {
        let output = std::process::Command::new("bcftools")
            .args(["view", &input_vcf.display().to_string()])
            .output()?;
        if !output.status.success() {
            bail!(
                "bcftools view failed while reading {}: {}",
                input_vcf.display(),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        std::fs::read_to_string(input_vcf)?
    };
    let mut headers = Vec::new();
    let mut records = Vec::new();
    let mut has_gt = false;
    let mut has_gl_or_gp = false;
    let mut has_phased_gt = false;
    let mut has_sex_chr = false;
    let mut contig_seen = std::collections::BTreeSet::<String>::new();
    let species_contigs =
        species_context.contigs.iter().map(|c| c.name.clone()).collect::<Vec<_>>();
    let species_contig_set =
        species_contigs.iter().cloned().collect::<std::collections::BTreeSet<_>>();
    let mut allele_flip_like = 0u64;
    let mut ref_mismatch_like = 0u64;
    let mut gt_observed = 0u64;
    let mut ct_ga_like = 0u64;
    let mut total_records = 0u64;
    for line in raw.lines() {
        if line.starts_with('#') {
            headers.push(line.to_string());
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        if matches!(fields[0], "X" | "Y" | "chrX" | "chrY") {
            has_sex_chr = true;
        }
        contig_seen.insert(fields[0].to_string());
        if !species_contig_set.contains(fields[0]) {
            ref_mismatch_like += 1;
        }
        if fields[3].eq_ignore_ascii_case(fields[4]) {
            allele_flip_like += 1;
        }
        let ref_upper = fields[3].to_ascii_uppercase();
        let alt_upper = fields[4].to_ascii_uppercase();
        if (ref_upper == "C" && alt_upper == "T") || (ref_upper == "G" && alt_upper == "A") {
            ct_ga_like += 1;
        }
        let gt_idx = parse_format_index(&fields, "GT");
        let gl_idx = parse_format_index(&fields, "GL");
        let gp_idx = parse_format_index(&fields, "GP");
        if gt_idx.is_some() {
            has_gt = true;
        }
        if gl_idx.is_some() || gp_idx.is_some() {
            has_gl_or_gp = true;
        }
        if let Some(gt_pos) = gt_idx {
            if let Some(sample) = fields.get(9) {
                let parts = sample.split(':').collect::<Vec<_>>();
                if let Some(gt) = parts.get(gt_pos) {
                    gt_observed += 1;
                    if gt.contains('|') {
                        has_phased_gt = true;
                    }
                    let ploidy = gt.split(['/', '|']).count();
                    if !gt.contains('.') && ploidy != 2 {
                        bail!("unsupported ploidy model at impute stage: only diploid genotypes are supported");
                    }
                }
            }
        }
        total_records += 1;
        records.push(line.to_string());
    }
    if records.is_empty() {
        bail!("impute requires non-empty VCF records");
    }
    if !contig_seen.is_subset(&species_contig_set) {
        bail!("contig digest/namespace mismatch between input VCF and SpeciesContext");
    }
    let overlap_threshold = 0.1f64;
    let overlap_fraction = if contig_seen.is_empty() {
        0.0
    } else {
        contig_seen.iter().filter(|c| species_contig_set.contains(*c)).count() as f64
            / contig_seen.len() as f64
    };
    if overlap_fraction < overlap_threshold {
        bail!("panel/species overlap below threshold");
    }
    if has_sex_chr && species_context.par_policy.eq_ignore_ascii_case("unsupported") {
        bail!("sex chromosome imputation requires explicit PAR policy in SpeciesContext");
    }

    let map = if params.map_id.is_some()
        || matches!(params.backend, ImputeBackend::Impute5 | ImputeBackend::Minimac4)
        || (matches!(params.backend, ImputeBackend::Beagle) && has_phased_gt)
    {
        Some(resolve_map(&params.species_id, &params.build_id, params.map_id.as_deref())?)
    } else {
        None
    };

    let backend_evidence = if has_gl_or_gp {
        BackendEvidence::GlLikelihood
    } else if has_phased_gt && panel.compatibility.supports_minimac_m3vcf && map.is_some() {
        BackendEvidence::PhasedWithMapMinimac
    } else if has_phased_gt && map.is_some() {
        BackendEvidence::PhasedWithMap
    } else {
        BackendEvidence::Generic
    };
    let recommended_backend = choose_backend_by_regime(params.backend, backend_evidence);
    let effective_backend = recommended_backend;
    if !matches!(effective_backend, ImputeBackend::Beagle) || params.map_id.is_some() {
        let map_for_compat = match &map {
            Some(m) => m.clone(),
            None => resolve_map(&params.species_id, &params.build_id, params.map_id.as_deref())?,
        };
        validate_imputation_tool_compatibility(
            effective_backend.as_str(),
            &panel,
            &map_for_compat,
        )?;
    }

    let sample_header = headers
        .iter()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("missing #CHROM header in input VCF"))?;
    let sample_ids = sample_header.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>();
    if sample_ids.is_empty() {
        bail!("input VCF must contain at least one sample");
    }
    if sample_ids.windows(2).any(|w| w.first().is_some_and(|x| x.is_empty()) || w[0] == w[1]) {
        bail!("sample order stability contract failed: duplicate/empty sample IDs");
    }

    match effective_backend {
        ImputeBackend::Glimpse => {
            if !has_gl_or_gp {
                bail!("GLIMPSE requires GL/GP fields for lowcov GL flow");
            }
            if !panel.compatibility.glimpse_reference_format.to_ascii_lowercase().contains("sites")
            {
                bail!("GLIMPSE requires panel compatibility with explicit sites representation");
            }
        }
        ImputeBackend::Impute5 => {
            if map.is_none() {
                bail!("Impute5 requires map_id/map asset");
            }
            if !has_gt && !has_gl_or_gp {
                bail!("Impute5 requires GT or GL/GP fields");
            }
        }
        ImputeBackend::Minimac4 => {
            if !has_phased_gt {
                bail!("Minimac4 requires phased GT prerequisite");
            }
            if !panel.compatibility.supports_minimac_m3vcf {
                bail!("Minimac4 requires m3vcf-compatible panel representation");
            }
            if map.is_none() {
                bail!("Minimac4 requires map_id/map asset");
            }
        }
        ImputeBackend::Beagle => {
            if !has_gt && !has_gl_or_gp {
                bail!("Beagle imputation requires GT or GL/GP fields");
            }
            if !params.emit_ds && !params.emit_gp {
                bail!("Beagle imputation requires at least one of DS/GP output policies");
            }
        }
    }

    return include!("execution_outputs.rs");
}
