use super::{
    anyhow, artifact_root_path, fs, json, read_json_value, read_utf8, sha256_hex, sha256_hex_bytes,
    value_string, write_json_pretty, write_utf8, BTreeMap, Context, Path, PathBuf, Regex, Result,
    Value, WalkDir, Workspace,
};

pub(super) fn toy_profile_id(profile: &str) -> &'static str {
    match profile {
        "fastq" => "fastq_reference_adna",
        "bam" => "bam_reference_adna",
        "vcf" => "vcf_reference_basic",
        _ => "unknown",
    }
}

pub(super) fn verify_toy_inputs(workspace: &Workspace) -> Result<BTreeMap<String, String>> {
    let toy_root = workspace.path("assets/toy/core-v1");
    let manifest = read_utf8(&toy_root.join("CHECKSUMS.sha256"))?;
    let mut checksums = BTreeMap::new();
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((digest, rel)) = trimmed.split_once("  ") else {
            continue;
        };
        let actual = sha256_hex(&toy_root.join(rel))?;
        if actual != digest {
            return Err(anyhow!(
                "toy input checksum mismatch for {rel}: expected {digest}, got {actual}"
            ));
        }
        checksums.insert(rel.to_string(), digest.to_string());
    }
    Ok(checksums)
}

pub(super) fn generate_toy_profile(
    workspace: &Workspace,
    profile: &str,
    out_root: &Path,
    checksums: &BTreeMap<String, String>,
) -> Result<PathBuf> {
    let profile_id = toy_profile_id(profile);
    let out_dir = out_root.join(profile_id);
    bijux_dna_infra::ensure_dir(&out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let generated_at = std::env::var("BIJUX_TOY_GENERATED_AT")
        .unwrap_or_else(|_| "1970-01-01T00:00:00+00:00".to_string());
    let manifest = json!({
        "schema_version": "bijux.toy.run_manifest.v1",
        "profile_id": profile_id,
        "domain": profile,
        "generated_at": generated_at,
        "inputs_root": workspace.rel(&workspace.path("assets/toy/core-v1").join(profile)).display().to_string(),
    });
    let mut metrics = match profile {
        "fastq" => json!({
            "schema_version": "bijux.toy.metrics.fastq.v1",
            "reads_total": 4,
            "bases_total": 40,
            "pairs": 2,
            "retention_ratio": 1.0,
            "input_checksums": checksum_subset(checksums, "fastq/"),
        }),
        "bam" => json!({
            "schema_version": "bijux.toy.metrics.bam.v1",
            "alignments": 4,
            "mapped": 4,
            "duplicate_rate": 0.0,
            "input_checksums": {
                "bam/toy.sam": checksums.get("bam/toy.sam").cloned().unwrap_or_default(),
            },
        }),
        "vcf" => json!({
            "schema_version": "bijux.toy.metrics.vcf.v1",
            "variants_total": 3,
            "snps": 2,
            "indels": 1,
            "ti_tv": 2.0,
            "filter_breakdown": {"PASS": 2, "LOWQUAL": 1},
            "input_checksums": {
                "vcf/toy.vcf": checksums.get("vcf/toy.vcf").cloned().unwrap_or_default(),
            },
        }),
        other => return Err(anyhow!("unknown toy profile: {other}")),
    };
    metrics["generated_at"] = Value::String(generated_at.clone());
    let report_html = format!(
        "<html><head><title>Bijux Toy Report</title></head><body><h1>{profile_id}</h1><p>generated_at={generated_at}</p><pre>{}</pre></body></html>\n",
        serde_json::to_string_pretty(&metrics)?
    );
    write_json_pretty(&out_dir.join("manifest.json"), &manifest)?;
    write_json_pretty(&out_dir.join("metrics.json"), &metrics)?;
    write_utf8(&out_dir.join("report.html"), &report_html)?;
    let artifact_hashes = json!({
        "manifest.json": stable_toy_digest(&out_dir.join("manifest.json"))?,
        "metrics.json": stable_toy_digest(&out_dir.join("metrics.json"))?,
        "report.html": stable_toy_digest(&out_dir.join("report.html"))?,
    });
    write_json_pretty(
        &out_dir.join("artifact_checksums.json"),
        &json!({
            "schema_version": "bijux.toy.artifact_checksums.v1",
            "profile_id": profile_id,
            "generated_at": generated_at,
            "artifacts": artifact_hashes,
        }),
    )?;
    Ok(out_dir)
}

fn checksum_subset(checksums: &BTreeMap<String, String>, prefix: &str) -> BTreeMap<String, String> {
    checksums
        .iter()
        .filter(|(key, _)| key.starts_with(prefix))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

fn stable_toy_digest(path: &Path) -> Result<String> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => {
            let payload = normalize_toy_json(&read_json_value(path)?);
            Ok(sha256_hex_bytes(serde_json::to_string(&payload)?.as_bytes()))
        }
        Some("html") => Ok(sha256_hex_bytes(normalize_toy_html(&read_utf8(path)?)?.as_bytes())),
        _ => sha256_hex(path),
    }
}

fn normalize_toy_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(key, _)| {
                    !matches!(
                        key.as_str(),
                        "generated_at" | "timestamp" | "started_at" | "finished_at"
                    )
                })
                .map(|(key, value)| (key.clone(), normalize_toy_json(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(normalize_toy_json).collect()),
        _ => value.clone(),
    }
}

fn normalize_toy_html(raw: &str) -> Result<String> {
    let generated_re = Regex::new(r"generated_at=[^<]+")?;
    let json_re = Regex::new(r#""generated_at"\s*:\s*"[^"]+""#)?;
    let text = generated_re.replace_all(raw, "generated_at=<normalized>");
    Ok(json_re.replace_all(&text, r#""generated_at":"<normalized>""#).into_owned())
}

pub(super) fn compare_toy_goldens(
    workspace: &Workspace,
    run_root: &Path,
    selected: &[&str],
) -> Result<()> {
    let golden_root = workspace.path("assets/golden/toy-runs-v1");
    let mut offenders = Vec::new();
    for profile in selected {
        let profile_id = toy_profile_id(profile);
        let produced = run_root.join(profile_id);
        let golden = golden_root.join(profile_id);
        for name in ["manifest.json", "metrics.json", "report.html", "artifact_checksums.json"] {
            let produced_path = produced.join(name);
            let golden_path = golden.join(name);
            if !produced_path.exists() || !golden_path.exists() {
                offenders.push(format!("missing counterpart for {profile_id}/{name}"));
                continue;
            }
            if stable_toy_digest(&produced_path)? != stable_toy_digest(&golden_path)? {
                offenders.push(format!("digest mismatch for {profile_id}/{name}"));
            }
        }
    }
    if offenders.is_empty() {
        return Ok(());
    }
    Err(anyhow!("golden mismatch:\n{}", offenders.join("\n")))
}

pub(super) fn build_combined_toy_report(run_root: &Path, selected: &[&str]) -> Result<PathBuf> {
    let mut rows = Vec::new();
    for profile in selected {
        let profile_id = toy_profile_id(profile);
        let metrics = read_json_value(&run_root.join(profile_id).join("metrics.json"))?;
        rows.push(format!(
            "<li><b>{profile_id}</b>: {}</li>",
            value_string(metrics.get("schema_version"))
        ));
    }
    let out = run_root.join("combined_demo_report.html");
    write_utf8(
        &out,
        &format!(
            "<html><head><title>Bijux Toy Demo</title></head><body><h1>Bijux Toy Demo</h1><ul>{}</ul></body></html>\n",
            rows.join("")
        ),
    )?;
    Ok(out)
}

pub(super) fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    bijux_dna_infra::ensure_dir(dst).with_context(|| format!("create {}", dst.display()))?;
    for entry in WalkDir::new(src).into_iter().filter_map(std::result::Result::ok) {
        let rel = match entry.path().strip_prefix(src) {
            Ok(rel) => rel,
            Err(_) => continue,
        };
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            bijux_dna_infra::ensure_dir(&target)
                .with_context(|| format!("create {}", target.display()))?;
        } else {
            if let Some(parent) = target.parent() {
                bijux_dna_infra::ensure_dir(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            fs::copy(entry.path(), &target).with_context(|| {
                format!("copy {} -> {}", entry.path().display(), target.display())
            })?;
        }
    }
    Ok(())
}

pub(super) fn temp_subdir(workspace: &Workspace, prefix: &str) -> Result<PathBuf> {
    let root = artifact_root_path(workspace)?.join("tmp");
    bijux_dna_infra::ensure_dir(&root)?;
    let path = root.join(format!("{prefix}.{}", std::process::id()));
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }
    bijux_dna_infra::ensure_dir(&path)?;
    Ok(path)
}
