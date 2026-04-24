use super::{anyhow, ExecuteRunRequest, Path, Result};

fn relative_path_string(base: &Path, path: &Path) -> String {
    path.strip_prefix(base).unwrap_or(path).to_string_lossy().to_string()
}

fn declared_string_array(value: Option<&serde_json::Value>, key: &str) -> Result<Vec<String>> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let items = value.as_array().ok_or_else(|| anyhow!("{key} must be an array when declared"))?;
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| anyhow!("{key} entries must be strings"))
        })
        .collect()
}

#[allow(clippy::too_many_lines)]
pub(super) fn maybe_emit_reference_manifest(
    request: &ExecuteRunRequest,
    run_artifacts_dir: &std::path::Path,
) -> Result<()> {
    let reference_inputs = request
        .plan
        .io
        .inputs
        .iter()
        .filter(|artifact| {
            let name = artifact.name.to_string().to_ascii_lowercase();
            name.contains("reference") || name.contains("fasta") || name.contains("ref")
        })
        .collect::<Vec<_>>();
    let species = request
        .plan
        .params
        .get("species_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let build =
        request.plan.params.get("build_id").and_then(serde_json::Value::as_str).map(str::to_string);
    let usecase =
        request.plan.params.get("usecase").and_then(serde_json::Value::as_str).map(str::to_string);
    let observed_contigs =
        declared_string_array(request.plan.params.get("observed_contigs"), "observed_contigs")?;

    let mut inputs = Vec::new();
    for artifact in reference_inputs {
        let resolved = request.plan.out_dir.join(&artifact.path);
        let sha256 = if resolved.exists() {
            Some(bijux_dna_infra::hash_file_sha256(&resolved)?)
        } else {
            None
        };
        inputs.push(serde_json::json!({
            "name": artifact.name,
            "path": relative_path_string(&request.plan.out_dir, &resolved),
            "sha256": sha256,
        }));
    }

    let reference_provenance = request.plan.params.get("reference_provenance").cloned();
    let workspace_root = crate::support::workspace::resolve_repo_root()?;
    let lock_json = workspace_root.join("configs/runtime/references/locks/lock.json");
    let lock_sig = workspace_root.join("configs/runtime/references/locks/lock.json.sha256");
    let lock_json_sha256 = if lock_json.exists() {
        Some(bijux_dna_infra::hash_file_sha256(&lock_json)?)
    } else {
        None
    };
    let lock_sig_sha256 =
        if lock_sig.exists() { Some(bijux_dna_infra::hash_file_sha256(&lock_sig)?) } else { None };
    let authority = if species.is_none() && build.is_none() && reference_provenance.is_none() {
        None
    } else {
        let mut authority = serde_json::Map::new();
        if let Some(species_id) = species {
            authority.insert("species_id".to_string(), serde_json::json!(species_id));
        }
        if let Some(build_id) = build {
            authority.insert("build_id".to_string(), serde_json::json!(build_id));
        }
        if let Some(kind) = usecase {
            authority.insert("usecase".to_string(), serde_json::json!(kind));
        }
        if !observed_contigs.is_empty() {
            authority.insert("observed_contigs".to_string(), serde_json::json!(observed_contigs));
        }
        if let Some(provenance) = reference_provenance {
            authority.insert("reference_provenance".to_string(), provenance);
        }
        authority.insert(
            "reference_lock".to_string(),
            serde_json::json!({
                "lock_json": relative_path_string(&workspace_root, &lock_json),
                "lock_json_sha256": lock_json_sha256,
                "lock_signature": relative_path_string(&workspace_root, &lock_sig),
                "lock_signature_sha256": lock_sig_sha256,
            }),
        );
        Some(serde_json::Value::Object(authority))
    };

    let payload = serde_json::json!({
        "schema_version": "bijux.reference_manifest.v1",
        "stage_id": request.plan.stage_id.to_string(),
        "tool_id": request.plan.tool_id.to_string(),
        "reference_inputs": inputs,
        "authority": authority,
    });
    bijux_dna_infra::atomic_write_json(
        &run_artifacts_dir.join("reference_manifest.json"),
        &payload,
    )?;
    Ok(())
}

pub(super) fn resolve_and_write_regime_stamp(
    request: &ExecuteRunRequest,
    run_artifacts_dir: &std::path::Path,
) -> Result<serde_json::Value> {
    let stamp = resolve_regime_stamp(request)?;
    bijux_dna_infra::atomic_write_json(&run_artifacts_dir.join("regime_stamp.json"), &stamp)?;
    Ok(stamp)
}

fn resolve_regime_stamp(request: &ExecuteRunRequest) -> Result<serde_json::Value> {
    let requested = request
        .plan
        .params
        .get("coverage_regime")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let profile = request
        .plan
        .params
        .get("regime_profile")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("default");
    let thresholds = load_coverage_thresholds(profile)?;

    let observed_bam_mean_depth =
        request.plan.params.get("mean_depth_x").and_then(serde_json::Value::as_f64).or_else(|| {
            let path =
                request.plan.out_dir.join("bam").join("coverage").join("coverage.regime.json");
            if !path.exists() {
                return None;
            }
            std::fs::read_to_string(path).ok().and_then(|raw| {
                serde_json::from_str::<serde_json::Value>(&raw)
                    .ok()
                    .and_then(|json| json.get("mean_depth").and_then(serde_json::Value::as_f64))
            })
        });

    let observed_fastq_estimated_depth = classify_fastq_only_estimated_depth(request);
    let observed_mean_depth = observed_bam_mean_depth.or(observed_fastq_estimated_depth);
    let observed_variant_density = classify_vcf_variant_density(request);

    let (selected, trigger) = if let Some(mean_depth) = observed_mean_depth {
        classify_depth_to_regime(mean_depth, thresholds)
    } else if let Some(regime) = requested.as_deref() {
        (
            regime,
            "requested coverage_regime used because observed mean depth is unavailable".to_string(),
        )
    } else if stage_requires_regime(request.plan.stage_id.as_str()) {
        return Err(anyhow!(
            "coverage regime required for stage {} but cannot be resolved: provide coverage_regime or mean_depth_x/FASTQ expected genome size inputs",
            request.plan.stage_id
        ));
    } else {
        (
            "not_required",
            "regime not required for stage and no observed/requested signal available".to_string(),
        )
    };

    let (impute_backend_default, chunk_size_bp_default) = match selected {
        "gl" => (Some("glimpse"), Some(2_000_000_u64)),
        "pseudohaploid" => (Some("beagle"), Some(3_000_000_u64)),
        "diploid" => (Some("minimac4"), Some(5_000_000_u64)),
        _ => (None, None),
    };

    Ok(serde_json::json!({
        "schema_version": "bijux.coverage_regime_stamp.v1",
        "stage_id": request.plan.stage_id.to_string(),
        "selected_regime": selected,
        "requested_regime": requested,
        "regime_profile": profile,
        "trigger": trigger,
        "thresholds_used": {
            "gl_max_depth": thresholds.gl_max,
            "pseudohaploid_max_depth": thresholds.pseudohaploid_max,
            "diploid_min_depth": thresholds.diploid_min,
        },
        "observed_coverage_stats": {
            "mean_depth_x": observed_mean_depth,
            "bam_mean_depth_x": observed_bam_mean_depth,
            "fastq_estimated_depth_x": observed_fastq_estimated_depth,
            "variant_density_per_mb": observed_variant_density,
        },
        "routing": {
            "call_stage": match selected {
                "gl" => Some("vcf.call_gl"),
                "pseudohaploid" => Some("vcf.call_pseudohaploid"),
                "diploid" => Some("vcf.call_diploid"),
                _ => None,
            },
            "imputation_backend_default": impute_backend_default,
            "chunk_size_bp_default": chunk_size_bp_default,
        }
    }))
}

fn classify_depth_to_regime(
    mean_depth: f64,
    thresholds: CoverageThresholds,
) -> (&'static str, String) {
    if mean_depth <= thresholds.gl_max {
        ("gl", format!("mean_depth_x <= gl_max_depth ({mean_depth:.4} <= {})", thresholds.gl_max))
    } else if mean_depth <= thresholds.pseudohaploid_max {
        (
            "pseudohaploid",
            format!(
                "gl_max_depth < mean_depth_x <= pseudohaploid_max_depth ({} < {mean_depth:.4} <= {})",
                thresholds.gl_max, thresholds.pseudohaploid_max
            ),
        )
    } else if mean_depth >= thresholds.diploid_min {
        (
            "diploid",
            format!(
                "mean_depth_x >= diploid_min_depth ({mean_depth:.4} >= {})",
                thresholds.diploid_min
            ),
        )
    } else {
        (
            "pseudohaploid",
            "fallback band between pseudohaploid_max_depth and diploid_min_depth".to_string(),
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct CoverageThresholds {
    gl_max: f64,
    pseudohaploid_max: f64,
    diploid_min: f64,
}

fn load_coverage_thresholds(profile: &str) -> Result<CoverageThresholds> {
    let root = crate::support::workspace::resolve_repo_root()?;
    let raw = std::fs::read_to_string(root.join("configs/runtime/coverage_regimes.toml"))?;
    let parsed: toml::Value = toml::from_str(&raw)?;
    let decision = parsed
        .get("decision")
        .and_then(|v| v.get("coverage_regime"))
        .ok_or_else(|| anyhow!("missing decision.coverage_regime in coverage_regimes.toml"))?;
    let base = decision
        .get("thresholds")
        .ok_or_else(|| anyhow!("missing decision.coverage_regime.thresholds"))?;
    let profile_thresholds = decision.get("profiles").and_then(|v| v.get(profile)).unwrap_or(base);
    let read_f = |obj: &toml::Value, key: &str| -> Result<f64> {
        obj.get(key)
            .and_then(toml::Value::as_float)
            .or_else(|| {
                obj.get(key)
                    .and_then(toml::Value::as_integer)
                    .and_then(|v| v.to_string().parse::<f64>().ok())
            })
            .ok_or_else(|| anyhow!("missing or invalid threshold key `{key}`"))
    };
    Ok(CoverageThresholds {
        gl_max: read_f(profile_thresholds, "gl_max_depth")?,
        pseudohaploid_max: read_f(profile_thresholds, "pseudohaploid_max_depth")?,
        diploid_min: read_f(profile_thresholds, "diploid_min_depth")?,
    })
}

fn stage_requires_regime(stage_id: &str) -> bool {
    matches!(
        stage_id,
        "vcf.call"
            | "vcf.call_gl"
            | "vcf.call_diploid"
            | "vcf.call_pseudohaploid"
            | "vcf.impute"
            | "vcf.phasing"
    )
}

fn classify_fastq_only_estimated_depth(request: &ExecuteRunRequest) -> Option<f64> {
    let expected_genome_size_bp = request
        .plan
        .params
        .get("expected_genome_size_bp")
        .and_then(serde_json::Value::as_u64)
        .or_else(|| std::env::var("BIJUX_EXPECTED_GENOME_SIZE_BP").ok()?.parse::<u64>().ok())?;
    if expected_genome_size_bp == 0 {
        return None;
    }
    let mut read_count: u64 = 0;
    let mut mean_len_sum: f64 = 0.0;
    let mut files: u64 = 0;
    for artifact in &request.plan.io.inputs {
        let p = request.plan.out_dir.join(&artifact.path);
        if !is_fastq_like_path(&p) {
            continue;
        }
        let inv = request.plan.out_dir.join("fastq_invariants.json");
        if inv.exists() {
            if let Ok(raw) = std::fs::read_to_string(&inv) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
                    let r1_count =
                        json.pointer("/r1/read_count").and_then(serde_json::Value::as_u64);
                    let r1_len =
                        json.pointer("/r1/read_length_mean").and_then(serde_json::Value::as_f64);
                    if let (Some(c), Some(l)) = (r1_count, r1_len) {
                        read_count = read_count.saturating_add(c);
                        mean_len_sum += l;
                        files = files.saturating_add(1);
                    }
                }
            }
        }
    }
    if read_count == 0 || files == 0 {
        return None;
    }
    let files_f = files.to_string().parse::<f64>().unwrap_or(1.0);
    let read_count_f = read_count.to_string().parse::<f64>().unwrap_or(0.0);
    let genome_bp_f = expected_genome_size_bp.to_string().parse::<f64>().unwrap_or(1.0);
    let avg_len = mean_len_sum / files_f;
    Some((read_count_f * avg_len) / genome_bp_f)
}

fn is_fastq_like_path(path: &Path) -> bool {
    let file_name = path
        .file_name()
        .and_then(|x| x.to_str())
        .filter(|value| !value.trim().is_empty())
        .map_or_else(|| path.display().to_string(), ToOwned::to_owned);
    let lower = file_name.to_ascii_lowercase();
    let ext_fastq = path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("fastq") || ext.eq_ignore_ascii_case("fq"));
    ext_fastq || lower.ends_with(".fastq.gz") || lower.ends_with(".fq.gz")
}

fn classify_vcf_variant_density(request: &ExecuteRunRequest) -> Option<f64> {
    let mut variants: u64 = 0;
    let mut span_bp: u64 = 0;
    for artifact in &request.plan.io.inputs {
        let p = request.plan.out_dir.join(&artifact.path);
        let name = p
            .file_name()
            .and_then(|x| x.to_str())
            .filter(|value| !value.trim().is_empty())
            .map_or_else(|| p.display().to_string(), ToOwned::to_owned);
        let ext_is_vcf = std::path::Path::new(&name)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("vcf"));
        if !(ext_is_vcf || name.to_ascii_lowercase().ends_with(".vcf.gz")) {
            continue;
        }
        let raw = if std::path::Path::new(&name)
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
        {
            let args = vec!["-cd".to_string(), p.to_string_lossy().into_owned()];
            bijux_dna_runner::command_runner::run_command("gzip", &args)
                .ok()
                .filter(|o| o.exit_code == 0)
                .map(|o| o.stdout)
        } else {
            std::fs::read_to_string(&p).ok()
        }?;
        let mut contig_max = std::collections::BTreeMap::<String, u64>::new();
        for line in raw.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let mut cols = line.split('\t');
            let chr = cols.next()?.to_string();
            let pos = cols.next()?.parse::<u64>().ok()?;
            variants = variants.saturating_add(1);
            let entry = contig_max.entry(chr).or_insert(0);
            *entry = (*entry).max(pos);
        }
        span_bp = span_bp.saturating_add(contig_max.values().sum::<u64>());
    }
    if variants == 0 || span_bp == 0 {
        return None;
    }
    let variants_f = variants.to_string().parse::<f64>().unwrap_or(0.0);
    let span_f = span_bp.to_string().parse::<f64>().unwrap_or(1.0);
    Some(variants_f / (span_f / 1_000_000_f64))
}

#[cfg(test)]
#[allow(clippy::items_after_test_module)]
mod coverage_regime_tests {
    use super::{
        classify_depth_to_regime, is_fastq_like_path, stage_requires_regime, CoverageThresholds,
    };
    use std::path::Path;

    #[test]
    fn same_input_depth_yields_same_regime_deterministically() {
        let t = CoverageThresholds { gl_max: 1.5, pseudohaploid_max: 6.0, diploid_min: 8.0 };
        let (a, _) = classify_depth_to_regime(1.2, t);
        let (b, _) = classify_depth_to_regime(1.2, t);
        let (c, _) = classify_depth_to_regime(1.2, t);
        assert_eq!(a, "gl");
        assert_eq!(a, b);
        assert_eq!(b, c);
    }

    #[test]
    fn depth_boundaries_route_to_expected_regime() {
        let t = CoverageThresholds { gl_max: 1.5, pseudohaploid_max: 6.0, diploid_min: 8.0 };
        assert_eq!(classify_depth_to_regime(1.5, t).0, "gl");
        assert_eq!(classify_depth_to_regime(1.5001, t).0, "pseudohaploid");
        assert_eq!(classify_depth_to_regime(6.0, t).0, "pseudohaploid");
        assert_eq!(classify_depth_to_regime(8.0, t).0, "diploid");
        assert_eq!(classify_depth_to_regime(7.0, t).0, "pseudohaploid");
    }

    #[test]
    fn regime_required_stage_catalog_is_explicit() {
        assert!(stage_requires_regime("vcf.call"));
        assert!(stage_requires_regime("vcf.call_gl"));
        assert!(stage_requires_regime("vcf.call_diploid"));
        assert!(stage_requires_regime("vcf.call_pseudohaploid"));
        assert!(stage_requires_regime("vcf.impute"));
        assert!(stage_requires_regime("vcf.phasing"));
        assert!(!stage_requires_regime("vcf.qc"));
        assert!(!stage_requires_regime("bam.coverage"));
    }

    #[test]
    fn fastq_path_detection_ignores_non_fastq_gz() {
        assert!(is_fastq_like_path(Path::new("r1.fastq")));
        assert!(is_fastq_like_path(Path::new("r1.fq")));
        assert!(is_fastq_like_path(Path::new("r1.fastq.gz")));
        assert!(is_fastq_like_path(Path::new("r1.fq.gz")));
        assert!(!is_fastq_like_path(Path::new("reference.fa.gz")));
        assert!(!is_fastq_like_path(Path::new("notes.txt.gz")));
    }
}
