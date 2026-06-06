use super::*;

pub(crate) fn workspace_root() -> Option<PathBuf> {
    crate::repo_root::resolve_repo_root().ok()
}

pub(crate) fn license_metadata_for_tool_exists(tool_id: &str) -> bool {
    workspace_root().is_some_and(|root| {
        root.join("containers/licenses").join(format!("{tool_id}.license.toml")).exists()
    })
}

pub(crate) fn resolve_tool_digest(tool_id: &str) -> Result<String> {
    let registry = workspace_root()
        .ok_or_else(|| anyhow!("resolve repository root for VCF runtime contracts"))?
        .join("configs/ci/registry/tool_registry_vcf_downstream.toml");
    let raw = std::fs::read_to_string(registry)?;
    let mut current_tool_id: Option<String> = None;
    let mut pinned_commit: Option<String> = None;
    let mut container_ref: Option<String> = None;
    let mut version: Option<String> = None;
    let flush_if_match = |current_tool_id: &Option<String>,
                          pinned_commit: &Option<String>,
                          container_ref: &Option<String>,
                          version: &Option<String>|
     -> Option<String> {
        if current_tool_id.as_deref() != Some(tool_id) {
            return None;
        }
        let digest_source = format!(
            "{}|{}|{}|{}",
            tool_id,
            pinned_commit.as_deref().unwrap_or("planned"),
            container_ref.as_deref().unwrap_or("registry_lock"),
            version.as_deref().unwrap_or("planned")
        );
        Some(format!("sha256:{}", checksum_hex(digest_source.as_bytes())))
    };
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed == "[[tools]]" {
            if let Some(found) =
                flush_if_match(&current_tool_id, &pinned_commit, &container_ref, &version)
            {
                return Ok(found);
            }
            current_tool_id = None;
            pinned_commit = None;
            container_ref = None;
            version = None;
            continue;
        }
        if let Some(value) = trimmed.strip_prefix("tool_id = ") {
            current_tool_id = Some(value.trim_matches('"').to_string());
        } else if let Some(value) = trimmed.strip_prefix("pinned_commit = ") {
            pinned_commit = Some(value.trim_matches('"').to_string());
        } else if let Some(value) = trimmed.strip_prefix("container_ref = ") {
            container_ref = Some(value.trim_matches('"').to_string());
        } else if let Some(value) = trimmed.strip_prefix("version = ") {
            version = Some(value.trim_matches('"').to_string());
        }
    }
    if let Some(found) = flush_if_match(&current_tool_id, &pinned_commit, &container_ref, &version)
    {
        return Ok(found);
    }
    bail!("could not resolve tool digest source for {tool_id}");
}

pub(crate) fn parse_format_index(fields: &[&str], name: &str) -> Option<usize> {
    fields
        .get(8)?
        .split(':')
        .enumerate()
        .find_map(|(idx, key)| if key == name { Some(idx) } else { None })
}

fn parse_threshold_value(raw: &str, key: &str) -> Option<f64> {
    raw.lines().find_map(|line| {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            return None;
        }
        let (lhs, rhs) = trimmed.split_once(':')?;
        if lhs.trim() != key {
            return None;
        }
        rhs.trim().parse::<f64>().ok()
    })
}

pub(crate) fn load_imputation_qc_thresholds() -> std::collections::BTreeMap<String, f64> {
    let raw = workspace_root()
        .and_then(|root| {
            std::fs::read_to_string(root.join("assets/reference/qc_thresholds.yaml")).ok()
        })
        .unwrap_or_default();
    let mut out = std::collections::BTreeMap::new();
    let defaults = [
        ("vcf_imputation_info_warn", 0.75_f64),
        ("vcf_imputation_info_fail", 0.60_f64),
        ("vcf_rsq_warn", 0.70_f64),
        ("vcf_rsq_fail", 0.55_f64),
        ("vcf_missingness_post_warn", 0.08_f64),
        ("vcf_missingness_post_fail", 0.15_f64),
        ("vcf_variant_density_warn", 2.0_f64),
        ("vcf_variant_density_fail", 1.0_f64),
        ("vcf_missingness_block_warn", 3.0_f64),
        ("vcf_missingness_block_fail", 6.0_f64),
        ("vcf_qc_sample_missingness_exclude", 0.50_f64),
        ("vcf_qc_variant_missingness_exclude", 0.50_f64),
    ];
    for (key, fallback) in defaults {
        out.insert(key.to_string(), parse_threshold_value(&raw, key).unwrap_or(fallback));
    }
    out
}

fn cleanup_policy() -> String {
    std::env::var("BIJUX_STAGE_CLEANUP_POLICY")
        .unwrap_or_else(|_| "keep".to_string())
        .to_ascii_lowercase()
}

fn backend_error_hint(
    stage_id: &str,
    backend: &str,
    err: &anyhow::Error,
) -> (&'static str, String) {
    let msg = err.to_string();
    if msg.contains("contig") || msg.contains("SpeciesContext") {
        return (
            "species_context_mismatch",
            "verify species/build/contig digest and input VCF contig namespace before rerun"
                .to_string(),
        );
    }
    if msg.contains("requires map") || msg.contains("map ") {
        return (
            "map_prerequisite_missing",
            format!("backend `{backend}` requires map compatibility; validate map_id + map locks"),
        );
    }
    if msg.contains("license") {
        return (
            "license_policy_block",
            format!("backend `{backend}` blocked by license metadata policy; add/update license metadata"),
        );
    }
    if msg.contains("ploidy") || msg.contains("GT") || msg.contains("GL/GP") {
        return (
            "input_field_contract_violation",
            format!("backend `{backend}` input field/ploidy contract failed; fix caller inputs (no silent coercions)"),
        );
    }
    if stage_id == "vcf.impute" && msg.contains("imputation_accept") {
        return (
            "qc_acceptance_failed",
            "imputation QC thresholds failed; inspect imputation_qc.json and decision.imputation_accept"
                .to_string(),
        );
    }
    (
        "backend_execution_error",
        format!("inspect stage logs/manifests and rerun with the same backend `{backend}` deterministically"),
    )
}

fn write_crash_provenance_artifact(
    out_dir: &Path,
    stage_id: &str,
    backend: &str,
    input_vcf: &Path,
    err: &anyhow::Error,
) -> Result<PathBuf> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let path = out_dir.join("crash_provenance.json");
    let (category, hint) = backend_error_hint(stage_id, backend, err);
    let err_text = format!("{err:#}");
    let stderr_tail = {
        let chars = err_text.chars().collect::<Vec<_>>();
        let keep = 800usize;
        if chars.len() <= keep {
            err_text
        } else {
            chars[chars.len() - keep..].iter().collect::<String>()
        }
    };
    let digest = resolve_tool_digest(backend).ok();
    let payload = serde_json::json!({
        "schema_version": "bijux.vcf.crash_provenance.v1",
        "stage_id": stage_id,
        "backend": backend,
        "error_category": category,
        "actionable_hint": hint,
        "command": serde_json::Value::Null,
        "stderr_tail": stderr_tail,
        "inputs": {
            "input_vcf": input_vcf,
        },
        "env_summary": {
            "cleanup_policy": cleanup_policy(),
            "hostname": std::env::var("HOSTNAME").ok(),
        },
        "tool_digest": digest,
    });
    atomic_write_json(&path, &payload)?;
    Ok(path)
}

fn apply_failure_cleanup_policy(out_dir: &Path) {
    if cleanup_policy() != "prune" {
        return;
    }
    for rel in ["tmp", "chunks", "intermediate", "scratch"] {
        let candidate = out_dir.join(rel);
        if candidate.exists() {
            let _ = bijux_dna_infra::remove_dir_all(&candidate);
        }
    }
}

fn read_vcf_text_runtime(path: &Path) -> Result<String> {
    if path.extension().and_then(|x| x.to_str()).is_some_and(|x| x == "gz" || x == "bcf") {
        let output = std::process::Command::new("bcftools")
            .args(["view", &path.display().to_string()])
            .output()?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).to_string());
        }
        if let Ok(raw) = std::fs::read(path) {
            return Ok(String::from_utf8_lossy(&raw).to_string());
        }
        bail!(
            "bcftools view failed while reading {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(std::fs::read_to_string(path)?)
}

fn write_bgzip_index_best_effort(
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

fn try_backend_invocation(bin: &str, args: &[&str]) -> bool {
    std::process::Command::new(bin).args(args).output().map(|x| x.status.success()).unwrap_or(false)
}

fn eagle_acceptance_allowed(species: &str, build: &str) -> bool {
    matches!((species, build), ("Homo sapiens", "GRCh38"))
}

fn parse_region_bounds(region: &str) -> Result<(String, u64, u64)> {
    let (contig, range) = region
        .split_once(':')
        .ok_or_else(|| anyhow!("region must match <contig>:<start>-<end>"))?;
    let (start, end) =
        range.split_once('-').ok_or_else(|| anyhow!("region must match <contig>:<start>-<end>"))?;
    let start_bp = start.parse::<u64>().map_err(|_| anyhow!("region start must be numeric"))?;
    let end_bp = end.parse::<u64>().map_err(|_| anyhow!("region end must be numeric"))?;
    if start_bp == 0 || end_bp < start_bp {
        bail!("region bounds must satisfy 0 < start <= end");
    }
    Ok((contig.to_string(), start_bp, end_bp))
}

fn beagle_with_gt_from_gl(line: &str) -> String {
    let mut fields = line.split('\t').map(str::to_string).collect::<Vec<_>>();
    if fields.len() < 10 {
        return line.to_string();
    }
    let fmt_keys = fields[8].split(':').collect::<Vec<_>>();
    if fmt_keys.contains(&"GT") {
        return line.replace("\t0/1", "\t0|1").replace("\t1/0", "\t1|0");
    }
    let sample_col = fields[9].clone();
    let format_col = fields[8].clone();
    let mut sample_vals = sample_col.split(':').collect::<Vec<_>>();
    fields[8] = format!("GT:{format_col}");
    sample_vals.insert(0, "0|1");
    fields[9] = sample_vals.join(":");
    fields.join("\t")
}

/// # Errors
/// Returns an error if phasing prerequisites or species/map policies are violated.
pub fn run_phasing_stage(
    input_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &PhasingStageParams,
) -> Result<PhasingStageOutputs> {
    match run_phasing_stage_inner(input_vcf, out_dir, species_context, params) {
        Ok(out) => Ok(out),
        Err(err) => {
            let _ = write_crash_provenance_artifact(
                out_dir,
                "vcf.phasing",
                params.backend.as_str(),
                input_vcf,
                &err,
            );
            apply_failure_cleanup_policy(out_dir);
            let (_, hint) = backend_error_hint("vcf.phasing", params.backend.as_str(), &err);
            Err(anyhow!("{err}; backend hint: {hint}"))
        }
    }
}

fn run_phasing_stage_inner(
    input_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &PhasingStageParams,
) -> Result<PhasingStageOutputs> {
    if params.species_id != species_context.species_id
        || params.build_id != species_context.build_id
    {
        bail!("species/build mismatch between phasing params and SpeciesContext");
    }
    if params.threads == 0 {
        bail!("phasing requires threads > 0");
    }

    let raw = read_vcf_text_runtime(input_vcf)?;
    let resolved_backend = resolve_phasing_backend(params, &raw);
    let backend_tool = resolved_backend.as_str();
    if matches!(resolved_backend, PhasingBackend::Eagle)
        && !license_metadata_for_tool_exists("eagle")
    {
        bail!("eagle requires non-bijux license metadata before execution");
    }
    if matches!(resolved_backend, PhasingBackend::Eagle)
        && !eagle_acceptance_allowed(&params.species_id, &params.build_id)
    {
        bail!(
            "eagle backend is outside accepted species/build list (allowed: Homo sapiens GRCh38)"
        );
    }

    let map = if matches!(resolved_backend, PhasingBackend::Shapeit5 | PhasingBackend::Eagle) {
        Some(resolve_map(&params.species_id, &params.build_id, params.map_id.as_deref())?)
    } else {
        params
            .map_id
            .as_deref()
            .map(|map_id| resolve_map(&params.species_id, &params.build_id, Some(map_id)))
            .transpose()?
    };
    if let Some(map_ref) = &map {
        if !map_ref.compatibility.tool_tags.iter().any(|tag| tag == backend_tool) {
            bail!("map {} is not compatible with backend {}", map_ref.id, backend_tool);
        }
    }

    let mut out_records = Vec::new();
    let mut header_lines = Vec::new();
    let mut has_gt = false;
    let mut has_gl_or_gp = false;
    let mut diploid_ok = true;
    let mut saw_records = false;
    let mut has_sex_chr = false;
    let mut has_sample_sex_metadata = false;
    let mut phase_switches = 0u64;
    let mut prev_gt: Option<String> = None;
    let mut variant_count = 0u64;

    for line in raw.lines() {
        if line.starts_with('#') {
            if line.starts_with("##SAMPLE=") && (line.contains("Sex=") || line.contains("SEX=")) {
                has_sample_sex_metadata = true;
            }
            header_lines.push(line.to_string());
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        saw_records = true;
        variant_count += 1;
        let chr = fields[0];
        if matches!(chr, "X" | "Y" | "chrX" | "chrY") {
            has_sex_chr = true;
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
                let sample_fields = sample.split(':').collect::<Vec<_>>();
                if let Some(gt_raw) = sample_fields.get(gt_pos) {
                    let allele_count = gt_raw.split(['/', '|']).count();
                    if allele_count != 2 {
                        diploid_ok = false;
                    }
                    let phased_gt = gt_raw.replace('/', "|");
                    if let Some(prev) = &prev_gt {
                        if prev != &phased_gt {
                            phase_switches += 1;
                        }
                    }
                    prev_gt = Some(phased_gt);
                }
            }
        }
        out_records.push(line.replace("\t0/1", "\t0|1").replace("\t1/0", "\t1|0"));
    }

    if !saw_records {
        bail!("phasing requires non-empty VCF records");
    }
    if matches!(resolved_backend, PhasingBackend::Shapeit5 | PhasingBackend::Eagle) && !has_gt {
        bail!("backend {} requires GT field", backend_tool);
    }
    if matches!(resolved_backend, PhasingBackend::Beagle) && !has_gt && !has_gl_or_gp {
        bail!("beagle phasing requires GT or GL/GP fields");
    }
    if has_gl_or_gp
        && !has_gt
        && !params.allow_gl_only_input
        && matches!(resolved_backend, PhasingBackend::Beagle | PhasingBackend::Auto)
    {
        bail!("GL-only/GP-only inputs are refused for phasing unless allow_gl_only_input=true");
    }
    if matches!(resolved_backend, PhasingBackend::Shapeit5 | PhasingBackend::Eagle) && !diploid_ok {
        bail!("backend {} requires diploid GT genotypes", backend_tool);
    }
    if has_sex_chr && species_context.par_policy.eq_ignore_ascii_case("unsupported") {
        bail!("sex chromosome phasing requires explicit PAR policy in SpeciesContext");
    }
    if has_sex_chr && !has_sample_sex_metadata {
        bail!("sex chromosome phasing requires sample sex metadata (##SAMPLE with Sex=)");
    }
    if has_sex_chr && !diploid_ok {
        bail!("male X/sex-chromosome haploid handling is unsupported in current phasing runner");
    }

    bijux_dna_infra::ensure_dir(out_dir)?;
    let phased_vcf = out_dir.join("phased.vcf.gz");
    let phase_block_stats_tsv = out_dir.join("phase_block_stats.tsv");
    let switch_error_proxy_tsv = out_dir.join("switch_error_proxy.tsv");
    let phasing_qc_json = out_dir.join("phasing_qc.json");
    let phasing_manifest_json = out_dir.join("phasing_manifest.json");
    let logs_txt = out_dir.join("logs.txt");
    let checksums = out_dir.join("checksums.sha256");

    let memory_mb = params.threads.saturating_mul(1024);
    let backend_argv = match resolved_backend {
        PhasingBackend::Shapeit5 => {
            let mut argv = vec![
                "shapeit5".to_string(),
                "--input".to_string(),
                input_vcf.to_string_lossy().to_string(),
                "--map".to_string(),
                map.as_ref()
                    .and_then(|m| m.files.first())
                    .map(|f| f.path.clone())
                    .unwrap_or_else(|| "missing-map".to_string()),
                "--threads".to_string(),
                params.threads.to_string(),
                "--seed".to_string(),
                params.seed.to_string(),
                "--memory-mb".to_string(),
                memory_mb.to_string(),
                "--output".to_string(),
                out_dir.join("shapeit5.out.vcf.gz").to_string_lossy().to_string(),
            ];
            if let Some(region) = &params.region {
                argv.push("--region".to_string());
                argv.push(region.clone());
            }
            argv
        }
        PhasingBackend::Beagle => vec![
            "beagle".to_string(),
            format!("gt={}", input_vcf.to_string_lossy()),
            format!("out={}", out_dir.join("beagle.out").to_string_lossy()),
            format!("seed={}", params.seed),
            format!("nthreads={}", params.threads),
        ],
        PhasingBackend::Eagle => {
            let mut argv = vec![
                "eagle".to_string(),
                "--vcfTarget".to_string(),
                input_vcf.to_string_lossy().to_string(),
                "--geneticMapFile".to_string(),
                map.as_ref()
                    .and_then(|m| m.files.first())
                    .map(|f| f.path.clone())
                    .unwrap_or_else(|| "missing-map".to_string()),
                "--numThreads".to_string(),
                params.threads.to_string(),
                "--seed".to_string(),
                params.seed.to_string(),
                "--maxMemoryMB".to_string(),
                memory_mb.to_string(),
                "--outPrefix".to_string(),
                out_dir.join("eagle.out").to_string_lossy().to_string(),
            ];
            if let Some(region) = &params.region {
                let (_chr, start_bp, end_bp) = parse_region_bounds(region)?;
                argv.push("--bpStart".to_string());
                argv.push(start_bp.to_string());
                argv.push("--bpEnd".to_string());
                argv.push(end_bp.to_string());
            }
            argv
        }
        PhasingBackend::Auto => vec![],
    };
    let phased_records = if matches!(resolved_backend, PhasingBackend::Beagle) {
        out_records.iter().map(|line| beagle_with_gt_from_gl(line)).collect::<Vec<_>>()
    } else {
        out_records.clone()
    };
    let phased_payload = format!("{}\n{}\n", header_lines.join("\n"), phased_records.join("\n"));
    let phasing_backend_attempted = if backend_argv.is_empty() {
        false
    } else {
        let cmd = backend_argv[0].clone();
        let args = backend_argv.iter().skip(1).map(|s| s.as_str()).collect::<Vec<_>>();
        try_backend_invocation(&cmd, &args)
    };
    let phased_tbi = write_bgzip_index_best_effort(&phased_vcf, &phased_payload, "phased.tmp.vcf")?;
    assert_bgzip_tabix_artifacts(&phased_vcf, &phased_tbi)?;

    let phase_block_n50 = (variant_count / 2).max(1);
    let switch_proxy =
        if variant_count == 0 { 0.0 } else { phase_switches as f64 / variant_count as f64 };
    atomic_write_bytes(
        &phase_block_stats_tsv,
        format!("metric\tvalue\nphase_block_n50\t{phase_block_n50}\n").as_bytes(),
    )?;
    atomic_write_bytes(
        &switch_error_proxy_tsv,
        format!("metric\tvalue\nswitch_error_proxy\t{switch_proxy:.6}\n").as_bytes(),
    )?;
    atomic_write_json(
        &phasing_qc_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.phasing.v1",
            "backend": backend_tool,
            "backend_selection": {
                "requested": params.backend.as_str(),
                "resolved": resolved_backend.as_str()
            },
            "phase_block_n50": phase_block_n50,
            "switch_error_proxy": switch_proxy,
            "warnings": if has_gl_or_gp { vec!["gl_or_gp_present"] } else { Vec::<&str>::new() },
        }),
    )?;

    let map_entry = map.as_ref().map(|m| {
        let file_checksums = m
            .files
            .iter()
            .map(|f| serde_json::json!({"name": f.name, "checksum_sha256": f.checksum_sha256}))
            .collect::<Vec<_>>();
        serde_json::json!({
            "map_id": m.id,
            "map_version": m.version,
            "coordinate_system": m.compatibility.coordinate_system,
            "checksums": file_checksums,
        })
    });
    let tool_digest = resolve_tool_digest(backend_tool)?;
    let command_exact = backend_argv.join(" ");
    atomic_write_json(
        &phasing_manifest_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.phasing.manifest.v1",
            "stage_id": "vcf.phasing",
            "backend": backend_tool,
            "requested_backend": params.backend.as_str(),
            "backend_execution_attempted": phasing_backend_attempted,
            "tool_digest": tool_digest,
            "species_id": species_context.species_id,
            "build_id": species_context.build_id,
            "seed": params.seed,
            "seed_policy": {
                "policy": "fixed_user_seed",
                "seed_source": "params.seed",
                "deterministic": true,
            },
            "threads": params.threads,
            "memory_mb": memory_mb,
            "region": params.region,
            "command_argv": backend_argv,
            "command_exact": command_exact,
            "input_contract": {
                "requires_gt_for_backends": ["shapeit5", "eagle"],
                "beagle_allows_gl_or_gp": true,
                "allow_gl_only_input": params.allow_gl_only_input,
            },
            "backend_acceptance_policy": {
                "eagle_allowed_species_build": ["Homo sapiens:GRCh38"],
                "license_metadata_required": true,
            },
            "map": map_entry,
            "sex_chromosome_policy": {
                "par_policy": species_context.par_policy,
                "sample_sex_metadata_required": true,
                "male_x_haploid_supported": false,
            },
            "provenance": {
                "input_vcf": input_vcf.display().to_string(),
                "output_vcf": phased_vcf.display().to_string(),
                "output_tbi": phased_tbi.display().to_string(),
                "stage": "vcf.phasing",
                "command": command_exact,
            },
            "input_checksum": checksum_hex(raw.as_bytes()),
            "output_checksum": checksum_hex(phased_payload.as_bytes()),
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "backend={backend_tool}\nseed={}\nthreads={}\nmap_required={}\nbackend_attempted={}\n",
            params.seed,
            params.threads,
            matches!(resolved_backend, PhasingBackend::Shapeit5 | PhasingBackend::Eagle),
            phasing_backend_attempted
        )
        .as_bytes(),
    )?;
    atomic_write_bytes(
        &checksums,
        format!(
            "{}  {}\n{}  {}\n",
            checksum_hex(phased_payload.as_bytes()),
            phased_vcf.display(),
            checksum_hex(std::fs::read_to_string(&phasing_manifest_json)?.as_bytes()),
            phasing_manifest_json.display()
        )
        .as_bytes(),
    )?;

    Ok(PhasingStageOutputs {
        phased_vcf,
        phased_tbi,
        phase_block_stats_tsv,
        switch_error_proxy_tsv,
        phasing_qc_json,
        phasing_manifest_json,
        logs_txt,
    })
}

// Errors are propagated from backend checks and artifact write contracts.
mod tail;

pub use tail::*;
