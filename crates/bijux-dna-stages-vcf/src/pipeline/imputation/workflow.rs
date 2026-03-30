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

include!("execution_engine.rs");

include!("postprocess.rs");
