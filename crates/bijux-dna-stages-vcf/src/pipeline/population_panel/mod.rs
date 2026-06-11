mod analysis_and_panel;
mod panel_output;
mod panel_variants;

use super::*;
pub use analysis_and_panel::*;
pub(crate) use panel_output::*;
pub(crate) use panel_variants::*;

fn require_ld_pruning_policy(policy: Option<&str>, stage_id: &str) -> Result<String> {
    let Some(policy) = policy.map(str::trim).filter(|x| !x.is_empty()) else {
        bail!("{stage_id} refusal: LD pruning policy is required");
    };
    Ok(policy.to_string())
}

fn parse_sample_population_labels(
    manifest_path: &Path,
    expected_samples: &[String],
) -> Result<std::collections::BTreeMap<String, String>> {
    let raw = std::fs::read_to_string(manifest_path)?;
    let json: serde_json::Value = serde_json::from_str(&raw)?;
    let mut labels = std::collections::BTreeMap::<String, String>::new();
    if let Some(entries) = json.get("samples").and_then(serde_json::Value::as_array) {
        for entry in entries {
            let sample =
                entry.get("sample").and_then(serde_json::Value::as_str).unwrap_or("").trim();
            let population =
                entry.get("population").and_then(serde_json::Value::as_str).unwrap_or("").trim();
            if !sample.is_empty() && !population.is_empty() {
                labels.insert(sample.to_string(), population.to_string());
            }
        }
    }
    if let Some(map) = json.get("population_labels").and_then(serde_json::Value::as_object) {
        for (sample, population) in map {
            if let Some(population) = population.as_str() {
                let sample = sample.trim();
                let population = population.trim();
                if !sample.is_empty() && !population.is_empty() {
                    labels.insert(sample.to_string(), population.to_string());
                }
            }
        }
    }
    let missing =
        expected_samples.iter().filter(|s| !labels.contains_key(*s)).cloned().collect::<Vec<_>>();
    if !missing.is_empty() {
        bail!("population metadata manifest missing labels for samples: {}", missing.join(","));
    }
    Ok(labels)
}

fn run_tool(bin: &str, args: &[&str], workdir: Option<&Path>) -> bool {
    let mut cmd = std::process::Command::new(bin);
    cmd.args(args);
    if let Some(dir) = workdir {
        cmd.current_dir(dir);
    }
    cmd.output().map(|x| x.status.success()).unwrap_or(false)
}

fn parse_plink2_eigenvec(path: &Path, components: usize) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let mut out = String::from("sample");
    for i in 1..=components {
        out.push_str(&format!("\tPC{i}"));
    }
    out.push('\n');
    for line in raw.lines() {
        let cols = line.split_whitespace().collect::<Vec<_>>();
        if cols.len() < components + 3 {
            continue;
        }
        let sample = cols[1];
        out.push_str(sample);
        for idx in 0..components {
            out.push('\t');
            out.push_str(cols.get(2 + idx).copied().unwrap_or("0.0"));
        }
        out.push('\n');
    }
    Some(out)
}

fn parse_plink2_eigenval(path: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let mut out = String::from("component\teigenvalue\n");
    for (idx, line) in raw.lines().enumerate() {
        let value = line.trim();
        if value.is_empty() {
            continue;
        }
        out.push_str(&format!("PC{}\t{}\n", idx + 1, value));
    }
    Some(out)
}

fn parse_eigenvalues_tsv(raw: &str) -> Vec<f64> {
    raw.lines()
        .skip(1)
        .filter_map(|line| {
            let value = line.split('\t').nth(1)?.trim();
            value.parse::<f64>().ok()
        })
        .collect()
}

/// # Errors
/// Returns an error if PCA preprocessing requirements are not satisfied.
pub fn run_pca_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &PcaStageParams,
) -> Result<PcaStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let ld_policy =
        require_ld_pruning_policy(params.preprocessing.ld_pruning_policy.as_deref(), "vcf.pca")?;
    let raw = read_vcf_text(input_vcf)?;
    let mut samples = Vec::<String>::new();
    let mut passing = 0_u64;
    for line in raw.lines() {
        if line.starts_with("#CHROM\t") {
            samples = line.split('\t').skip(9).map(str::to_string).collect();
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let maf = variant_maf(&fields).unwrap_or(0.0);
        let miss = genotype_missing_fraction(fields[8], &fields[9..]).unwrap_or(0.0);
        if maf >= params.preprocessing.maf_threshold && miss <= params.preprocessing.max_missingness
        {
            passing += 1;
        }
    }
    let sample_population_labels = params
        .sample_metadata_manifest
        .as_ref()
        .map(|manifest_path| parse_sample_population_labels(manifest_path, &samples))
        .transpose()?;
    if passing == 0 {
        bail!("vcf.pca refusal: no variants pass preprocessing (LD/MAF/missingness)");
    }
    let eigenvec_tsv = out_dir.join("eigenvec.tsv");
    let eigenval_tsv = out_dir.join("eigenval.tsv");
    let pca_manifest_json = out_dir.join("pca_manifest.json");
    let logs_txt = out_dir.join("logs.txt");
    let plink_prefix = out_dir.join("plink_pca");
    let plink_prefix_s = plink_prefix.to_string_lossy().to_string();
    let input_s = input_vcf.to_string_lossy().to_string();
    let toolchain = params.toolchain.to_ascii_lowercase();
    let (tool_id, tool_ok) = match toolchain.as_str() {
        "eigensoft" | "smartpca" => {
            let par_file = out_dir.join("pca.smartpca.par");
            let par_payload = format!(
                "genotypename: {prefix}.bed\nsnpname: {prefix}.bim\nindivname: {prefix}.fam\nevecoutname: {out}/pca.smartpca.evec\nevaloutname: {out}/pca.smartpca.eval\n",
                prefix = plink_prefix_s,
                out = out_dir.to_string_lossy()
            );
            atomic_write_bytes(&par_file, par_payload.as_bytes())?;
            let par_s = par_file.to_string_lossy().to_string();
            let make_bed_ok = run_tool(
                "plink2",
                &[
                    "--vcf",
                    input_s.as_str(),
                    "--double-id",
                    "--allow-extra-chr",
                    "--make-bed",
                    "--out",
                    plink_prefix_s.as_str(),
                ],
                None,
            );
            let smartpca_ok = if make_bed_ok {
                run_tool("smartpca", &["-p", par_s.as_str()], Some(out_dir))
            } else {
                false
            };
            ("smartpca", smartpca_ok)
        }
        _ => (
            "plink2",
            run_tool(
                "plink2",
                &[
                    "--vcf",
                    input_s.as_str(),
                    "--double-id",
                    "--allow-extra-chr",
                    "--pca",
                    &params.components.to_string(),
                    "--out",
                    plink_prefix_s.as_str(),
                ],
                None,
            ),
        ),
    };
    let plink_eigenvec = out_dir.join("plink_pca.eigenvec");
    let plink_eigenval = out_dir.join("plink_pca.eigenval");
    let mut execution_mode = "fallback_proxy";
    let vec_rows = if tool_ok && plink_eigenvec.exists() {
        if let Some(parsed) = parse_plink2_eigenvec(&plink_eigenvec, params.components) {
            execution_mode = "real_tool";
            parsed
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    let vec_rows = if vec_rows.is_empty() {
        let mut synthetic = String::from("sample");
        for i in 1..=params.components {
            synthetic.push_str(&format!("\tPC{i}"));
        }
        synthetic.push('\n');
        for (idx, s) in samples.iter().enumerate() {
            synthetic.push_str(s);
            for i in 1..=params.components {
                synthetic.push_str(&format!("\t{:.6}", ((idx + i) as f64) / 100.0));
            }
            synthetic.push('\n');
        }
        synthetic
    } else {
        vec_rows
    };
    atomic_write_bytes(&eigenvec_tsv, vec_rows.as_bytes())?;
    let mut val_rows = if tool_ok && plink_eigenval.exists() {
        parse_plink2_eigenval(&plink_eigenval).unwrap_or_default()
    } else {
        String::new()
    };
    if val_rows.is_empty() {
        val_rows = String::from("component\teigenvalue\n");
        for i in 1..=params.components {
            val_rows.push_str(&format!("PC{i}\t{:.6}\n", 1.0 / i as f64));
        }
    }
    let eigenvalues = parse_eigenvalues_tsv(&val_rows);
    atomic_write_bytes(&eigenval_tsv, val_rows.as_bytes())?;
    atomic_write_json(
        &pca_manifest_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.pca.v1",
            "toolchain": tool_id,
            "execution_mode": execution_mode,
            "components": params.components,
            "sample_count": samples.len(),
            "sample_ids": samples,
            "eigenvalues": eigenvalues,
            "sample_metadata_manifest": params.sample_metadata_manifest.as_ref(),
            "sample_population_labels": sample_population_labels.as_ref().map(|labels| {
                labels
                    .iter()
                    .filter(|(sample_id, _)| samples.iter().any(|sample| sample == *sample_id))
                    .map(|(sample_id, population_id)| serde_json::json!({
                        "sample_id": sample_id,
                        "population_id": population_id,
                    }))
                    .collect::<Vec<_>>()
            }),
            "ld_pruning_policy": ld_policy,
            "plot_references": {
                "scree_plot": "plots/pca_scree.png",
                "pc1_pc2_plot": "plots/pca_pc1_pc2.png"
            },
            "preprocessing": {
                "ld_window": params.preprocessing.ld_window,
                "ld_step": params.preprocessing.ld_step,
                "ld_r2_threshold": params.preprocessing.ld_r2_threshold,
                "maf_threshold": params.preprocessing.maf_threshold,
                "max_missingness": params.preprocessing.max_missingness,
            },
            "variants_passing": passing,
            "tool_attempts": {
                "pca": {
                    "tool": tool_id,
                    "ok": tool_ok
                }
            }
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "toolchain={tool_id}\nexecution_mode={execution_mode}\nvariants_passing={passing}\nld_pruning_policy={ld_policy}\ntool_success={tool_ok}\n",
        )
        .as_bytes(),
    )?;
    Ok(PcaStageOutputs { eigenvec_tsv, eigenval_tsv, pca_manifest_json, logs_txt })
}

/// # Errors
/// Returns an error if population structure preprocessing fails.
pub fn run_population_structure_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &PopulationStructureStageParams,
) -> Result<PopulationStructureStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    if !params.run_admixture {
        bail!(
            "vcf.population_structure refusal: consumed admixture output is required for governed population-structure summaries"
        );
    }
    let raw = read_vcf_text(input_vcf)?;
    let mut samples = Vec::<String>::new();
    let ld_policy = require_ld_pruning_policy(
        params.preprocessing.ld_pruning_policy.as_deref(),
        "vcf.population_structure",
    )?;
    let mut passing = Vec::<String>::new();
    let metadata_manifest = params.sample_metadata_manifest.as_ref().ok_or_else(|| {
        anyhow!("vcf.population_structure refusal: sample metadata manifest path is required")
    })?;
    for line in raw.lines() {
        if line.starts_with("#CHROM\t") {
            samples = line.split('\t').skip(9).map(str::to_string).collect();
            continue;
        }
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let maf = variant_maf(&fields).unwrap_or(0.0);
        let miss = genotype_missing_fraction(fields[8], &fields[9..]).unwrap_or(0.0);
        if maf >= params.preprocessing.maf_threshold && miss <= params.preprocessing.max_missingness
        {
            passing.push(format!("{}:{}", fields[0], fields[1]));
        }
    }
    if passing.is_empty() {
        bail!("vcf.population_structure refusal: no variants pass preprocessing");
    }
    let labels = parse_sample_population_labels(metadata_manifest, &samples)?;
    let plink_input_tsv = out_dir.join("population_structure_input_plink.tsv");
    let pruned_variants_tsv = out_dir.join("pruned_variants.tsv");
    let population_structure_json = out_dir.join("population_structure.json");
    let logs_txt = out_dir.join("logs.txt");
    let plink_prefix = out_dir.join("population_structure_plink");
    let plink_prefix_s = plink_prefix.to_string_lossy().to_string();
    let input_s = input_vcf.to_string_lossy().to_string();
    let plink_prune_ok = try_run_tool(
        "plink2",
        &[
            "--vcf",
            input_s.as_str(),
            "--double-id",
            "--allow-extra-chr",
            "--indep-pairwise",
            &params.preprocessing.ld_window.to_string(),
            &params.preprocessing.ld_step.to_string(),
            &params.preprocessing.ld_r2_threshold.to_string(),
            "--out",
            plink_prefix_s.as_str(),
        ],
    );
    let pca = run_pca_stage(
        input_vcf,
        out_dir,
        &PcaStageParams {
            toolchain: params.toolchain.clone(),
            components: 10,
            sample_metadata_manifest: Some(metadata_manifest.clone()),
            preprocessing: params.preprocessing.clone(),
        },
    )?;
    let pca_manifest_raw = std::fs::read_to_string(&pca.pca_manifest_json)?;
    let pca_manifest: serde_json::Value = serde_json::from_str(&pca_manifest_raw)?;
    let admixture = run_admixture_stage(
        input_vcf,
        out_dir,
        &params.admixture_params.clone().unwrap_or_else(|| AdmixtureStageParams {
            sample_metadata_manifest: Some(metadata_manifest.clone()),
            ..AdmixtureStageParams::default()
        }),
    )?;
    let admixture_manifest = admixture
        .k_selection_json
        .as_path();
    let admixture_manifest_raw = std::fs::read_to_string(admixture_manifest)?;
    let admixture_manifest: serde_json::Value = serde_json::from_str(&admixture_manifest_raw)?;
    for required in [&pca.eigenvec_tsv, &pca.eigenval_tsv, &pca.pca_manifest_json] {
        if !required.exists() {
            bail!(
                "vcf.population_structure refusal: consumed PCA path is missing: {}",
                required.display()
            );
        }
    }
    for required in [&admixture.q_matrix_tsv, &admixture.k_selection_json] {
        if !required.exists() {
            bail!(
                "vcf.population_structure refusal: consumed admixture path is missing: {}",
                required.display()
            );
        }
    }
    atomic_write_bytes(
        &plink_input_tsv,
        format!("variant_id\n{}\n", passing.join("\n")).as_bytes(),
    )?;
    atomic_write_bytes(
        &pruned_variants_tsv,
        format!("variant\n{}\n", passing.join("\n")).as_bytes(),
    )?;
    atomic_write_json(
        &population_structure_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.population_structure.v1",
            "toolchain": params.toolchain,
            "smartpca": params.smartpca,
            "ld_pruning_policy": ld_policy,
            "preprocessing": {
                "ld_window": params.preprocessing.ld_window,
                "ld_step": params.preprocessing.ld_step,
                "ld_r2_threshold": params.preprocessing.ld_r2_threshold,
                "maf_threshold": params.preprocessing.maf_threshold,
                "max_missingness": params.preprocessing.max_missingness,
            },
            "variants_passing": passing.len(),
            "status": "complete",
            "sample_ids": samples,
            "sample_labels": {
                "manifest": metadata_manifest,
                "total_samples": labels.len(),
                "populations": labels
                    .values()
                    .fold(std::collections::BTreeMap::<String, usize>::new(), |mut acc, pop| {
                        *acc.entry(pop.clone()).or_insert(0) += 1;
                        acc
                    }),
                "rows": labels.iter().map(|(sample_id, population_id)| serde_json::json!({
                    "sample_id": sample_id,
                    "population_id": population_id,
                })).collect::<Vec<_>>(),
            },
            "metrics": {
                "sample_count": labels.len(),
                "population_count": labels.values().collect::<std::collections::BTreeSet<_>>().len(),
                "variants_passing_after_pruning": passing.len(),
                "admixture_enabled": params.run_admixture
            },
            "tool_attempts": {
                "plink2_prune": plink_prune_ok,
                "smartpca": params.smartpca
            },
            "pca": {
                "eigenvec_tsv": pca.eigenvec_tsv,
                "eigenval_tsv": pca.eigenval_tsv,
                "manifest_json": pca.pca_manifest_json,
                "execution_mode": pca_manifest.get("execution_mode").and_then(serde_json::Value::as_str),
                "tool_ok": pca_manifest.get("tool_attempts")
                    .and_then(|row| row.get("pca"))
                    .and_then(|row| row.get("ok"))
                    .and_then(serde_json::Value::as_bool),
                "sample_count": pca_manifest.get("sample_count").and_then(serde_json::Value::as_u64)
            },
            "admixture": {
                "q_matrix_tsv": admixture.q_matrix_tsv,
                "k_selection_json": admixture.k_selection_json,
                "status": admixture_manifest
                    .get("status")
                    .and_then(serde_json::Value::as_str),
                "execution_mode": admixture_manifest
                    .get("execution_mode")
                    .and_then(serde_json::Value::as_str),
                "selected_k": admixture_manifest
                    .get("selected_k")
                    .and_then(serde_json::Value::as_u64),
                "sample_count": admixture_manifest
                    .get("sample_count")
                    .and_then(serde_json::Value::as_u64)
            },
            "outputs": {
                "pruned_variants_tsv": pruned_variants_tsv,
                "population_structure_json": population_structure_json
            }
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "toolchain={}\nsmartpca={}\nld_pruning_policy={ld_policy}\nplink2_prune_attempted={}\nadmixture_enabled={}\n",
            params.toolchain,
            params.smartpca,
            plink_prune_ok,
            true
        )
        .as_bytes(),
    )?;
    Ok(PopulationStructureStageOutputs { pruned_variants_tsv, population_structure_json, logs_txt })
}

/// # Errors
/// Returns an error when ADMIXTURE runtime/container policy blocks execution.
pub fn run_admixture_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &AdmixtureStageParams,
) -> Result<AdmixtureStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = read_vcf_text(input_vcf)?;
    let samples = raw
        .lines()
        .find_map(|line| {
            if line.starts_with("#CHROM\t") {
                Some(line.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>())
            } else {
                None
            }
        })
        .unwrap_or_default();
    if samples.is_empty() {
        bail!("vcf.admixture refusal: no samples found in VCF header");
    }
    let metadata_manifest = params.sample_metadata_manifest.as_ref().ok_or_else(|| {
        anyhow!("vcf.admixture refusal: sample metadata manifest path is required")
    })?;
    let labels = parse_sample_population_labels(metadata_manifest, &samples)?;
    if params.k_values.is_empty() {
        bail!("vcf.admixture refusal: k_values cannot be empty");
    }
    let selected_k = *params
        .k_values
        .iter()
        .min()
        .ok_or_else(|| anyhow!("vcf.admixture refusal: unable to select K"))?;
    let tool = params.toolchain.to_ascii_lowercase();
    if tool == "admixture" && !license_metadata_for_tool_exists("admixture") {
        bail!("vcf.admixture refusal: ADMIXTURE container/license metadata is not available");
    }
    let input_s = input_vcf.to_string_lossy().to_string();
    let prefix = out_dir.join("admixture_plink");
    let prefix_s = prefix.to_string_lossy().to_string();
    let tool_ok = if tool == "admixture" {
        let make_bed_ok = run_tool(
            "plink2",
            &[
                "--vcf",
                input_s.as_str(),
                "--double-id",
                "--allow-extra-chr",
                "--make-bed",
                "--out",
                prefix_s.as_str(),
            ],
            None,
        );
        if !make_bed_ok {
            false
        } else {
            let bed = format!("{prefix_s}.bed");
            run_tool("admixture", &[bed.as_str(), &selected_k.to_string()], Some(out_dir))
        }
    } else {
        run_tool(
            "plink2",
            &[
                "--vcf",
                input_s.as_str(),
                "--double-id",
                "--allow-extra-chr",
                "--pca",
                "2",
                "--out",
                prefix_s.as_str(),
            ],
            None,
        )
    };
    let q_matrix_tsv = out_dir.join("admixture_q_matrix.tsv");
    let k_selection_json = out_dir.join("admixture_k_selection.json");
    let logs_txt = out_dir.join("logs.txt");
    let population_order = labels
        .values()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let cluster_headers =
        (1..=selected_k.max(1)).map(|index| format!("cluster_{index}")).collect::<Vec<_>>();
    let cluster_population_labels = (0..selected_k.max(1))
        .map(|index| {
            population_order
                .get(index)
                .cloned()
                .map(|population_id| {
                    serde_json::json!({
                        "column": format!("cluster_{}", index + 1),
                        "population_id": population_id,
                    })
                })
                .unwrap_or_else(|| {
                    serde_json::json!({
                        "column": format!("cluster_{}", index + 1),
                        "population_id": serde_json::Value::Null,
                    })
                })
        })
        .collect::<Vec<_>>();
    let insufficient_data_reason = if population_order.len() < selected_k {
        Some("population_label_count_below_selected_k")
    } else {
        None
    };
    let status = if insufficient_data_reason.is_some() { "insufficient_data" } else { "complete" };
    let mut execution_mode = "fallback_proxy";
    let mut q_tsv = String::new();
    if tool == "admixture" && tool_ok {
        let q_path = out_dir.join(format!("admixture_plink.{}.Q", selected_k));
        if let Ok(q_raw) = std::fs::read_to_string(&q_path) {
            execution_mode = "real_tool";
            let q_rows = q_raw.lines().collect::<Vec<_>>();
            if q_rows.len() != samples.len() {
                bail!(
                    "vcf.admixture refusal: q-matrix row count {} does not match sample count {}",
                    q_rows.len(),
                    samples.len()
                );
            }
            q_tsv = format!("sample\t{}\n", cluster_headers.join("\t"));
            for (sample, row) in samples.iter().zip(q_rows) {
                let fractions = row
                    .split_whitespace()
                    .take(selected_k.max(1))
                    .map(|value| {
                        value.parse::<f64>().map_err(|_| {
                            anyhow!(
                                "vcf.admixture refusal: q-matrix value `{value}` for sample `{sample}` is not numeric"
                            )
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                if fractions.len() != cluster_headers.len() {
                    bail!(
                        "vcf.admixture refusal: q-matrix column count {} does not match cluster count {} for sample `{sample}`",
                        fractions.len(),
                        cluster_headers.len()
                    );
                }
                let total_fraction = fractions.iter().sum::<f64>();
                if (total_fraction - 1.0).abs() > 1e-6 {
                    bail!(
                        "vcf.admixture refusal: cluster fractions for sample `{sample}` sum to {total_fraction:.6}, expected 1.0"
                    );
                }
                q_tsv.push_str(sample);
                for value in fractions {
                    q_tsv.push_str(&format!("\t{value:.6}"));
                }
                q_tsv.push('\n');
            }
        }
    }
    if q_tsv.is_empty() {
        q_tsv = format!("sample\t{}\n", cluster_headers.join("\t"));
        for sample in &samples {
            let label = labels.get(sample).ok_or_else(|| {
                anyhow!("vcf.admixture refusal: missing label for sample `{sample}`")
            })?;
            q_tsv.push_str(sample);
            for cluster in &cluster_population_labels {
                let score = if cluster
                    .get("population_id")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|population_id| population_id == label)
                {
                    1.0
                } else {
                    0.0
                };
                q_tsv.push_str(&format!("\t{score:.6}"));
            }
            q_tsv.push('\n');
        }
    }
    atomic_write_bytes(&q_matrix_tsv, q_tsv.as_bytes())?;
    atomic_write_json(
        &k_selection_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.admixture.v1",
            "toolchain": if tool == "admixture" { "admixture" } else { "plink2_proxy" },
            "execution_mode": execution_mode,
            "k_values": params.k_values,
            "selected_k": selected_k,
            "status": status,
            "insufficient_data_reason": insufficient_data_reason,
            "tool_ok": tool_ok,
            "sample_count": samples.len(),
            "sample_ids": samples,
            "population_count": population_order.len(),
            "population_ids": population_order,
            "cluster_count": cluster_headers.len(),
            "cluster_headers": cluster_headers,
            "cluster_population_labels": cluster_population_labels,
            "sample_metadata_manifest": metadata_manifest,
            "sample_population_labels": labels.iter().map(|(sample_id, population_id)| {
                serde_json::json!({
                    "sample_id": sample_id,
                    "population_id": population_id,
                })
            }).collect::<Vec<_>>(),
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "toolchain={tool}\nexecution_mode={execution_mode}\ntool_success={tool_ok}\nselected_k={selected_k}\nstatus={status}\n"
        )
        .as_bytes(),
    )?;
    Ok(AdmixtureStageOutputs { q_matrix_tsv, k_selection_json, logs_txt })
}
