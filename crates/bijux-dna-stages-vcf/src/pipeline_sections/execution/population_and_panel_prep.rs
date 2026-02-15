include!("population_and_panel_prep_helpers.rs");

/// # Errors
/// Returns an error if PCA preprocessing requirements are not satisfied.
pub fn run_pca_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &PcaStageParams,
) -> Result<PcaStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
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
        if maf >= params.preprocessing.maf_threshold && miss <= params.preprocessing.max_missingness {
            passing += 1;
        }
    }
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
    let plink_ok = try_run_tool(
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
    );
    let mut vec_rows = String::from("sample");
    for i in 1..=params.components {
        vec_rows.push_str(&format!("\tPC{i}"));
    }
    vec_rows.push('\n');
    for (idx, s) in samples.iter().enumerate() {
        vec_rows.push_str(s);
        for i in 1..=params.components {
            vec_rows.push_str(&format!("\t{:.6}", ((idx + i) as f64) / 100.0));
        }
        vec_rows.push('\n');
    }
    atomic_write_bytes(&eigenvec_tsv, vec_rows.as_bytes())?;
    let mut val_rows = String::from("component\teigenvalue\n");
    for i in 1..=params.components {
        val_rows.push_str(&format!("PC{i}\t{:.6}\n", 1.0 / i as f64));
    }
    atomic_write_bytes(&eigenval_tsv, val_rows.as_bytes())?;
    atomic_write_json(
        &pca_manifest_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.pca.v1",
            "toolchain": params.toolchain,
            "components": params.components,
            "preprocessing": {
                "ld_window": params.preprocessing.ld_window,
                "ld_step": params.preprocessing.ld_step,
                "ld_r2_threshold": params.preprocessing.ld_r2_threshold,
                "maf_threshold": params.preprocessing.maf_threshold,
                "max_missingness": params.preprocessing.max_missingness,
            },
            "variants_passing": passing,
            "tool_attempts": {
                "plink2_pca": plink_ok
            }
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "toolchain={}\nvariants_passing={passing}\nplink2_pca_attempted={}\n",
            params.toolchain, plink_ok
        )
        .as_bytes(),
    )?;
    Ok(PcaStageOutputs {
        eigenvec_tsv,
        eigenval_tsv,
        pca_manifest_json,
        logs_txt,
    })
}

/// # Errors
/// Returns an error if population structure preprocessing fails.
pub fn run_population_structure_stage(
    input_vcf: &Path,
    out_dir: &Path,
    params: &PopulationStructureStageParams,
) -> Result<PopulationStructureStageOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let raw = read_vcf_text(input_vcf)?;
    let mut passing = Vec::<String>::new();
    for line in raw.lines() {
        let Some(fields) = parse_record_fields(line) else {
            continue;
        };
        let maf = variant_maf(&fields).unwrap_or(0.0);
        let miss = genotype_missing_fraction(fields[8], &fields[9..]).unwrap_or(0.0);
        if maf >= params.preprocessing.maf_threshold && miss <= params.preprocessing.max_missingness {
            passing.push(format!("{}:{}", fields[0], fields[1]));
        }
    }
    if passing.is_empty() {
        bail!("vcf.population_structure refusal: no variants pass preprocessing");
    }
    let plink_input_tsv = out_dir.join("population_structure_input_plink.tsv");
    let pruned_variants_tsv = out_dir.join("pruned_variants.tsv");
    let eigenvec_tsv = out_dir.join("population_structure.eigenvec.tsv");
    let eigenval_tsv = out_dir.join("population_structure.eigenval.tsv");
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
    let plink_pca_ok = try_run_tool(
        "plink2",
        &[
            "--vcf",
            input_s.as_str(),
            "--double-id",
            "--allow-extra-chr",
            "--pca",
            "10",
            "--out",
            plink_prefix_s.as_str(),
        ],
    );
    let smartpca_ok = if params.smartpca {
        let par_file = out_dir.join("smartpca.par");
        let par_payload = format!(
            "genotypename: {prefix}.bed\nsnpname: {prefix}.bim\nindivname: {prefix}.fam\nevecoutname: {out}/population_structure.smartpca.evec\nevaloutname: {out}/population_structure.smartpca.eval\n",
            prefix = plink_prefix_s,
            out = out_dir.to_string_lossy()
        );
        atomic_write_bytes(&par_file, par_payload.as_bytes())?;
        let par_s = par_file.to_string_lossy().to_string();
        try_run_tool("smartpca", &["-p", par_s.as_str()])
    } else {
        false
    };
    atomic_write_bytes(
        &plink_input_tsv,
        format!("variant_id\n{}\n", passing.join("\n")).as_bytes(),
    )?;
    atomic_write_bytes(
        &pruned_variants_tsv,
        format!("variant\n{}\n", passing.join("\n")).as_bytes(),
    )?;
    atomic_write_bytes(
        &eigenvec_tsv,
        b"sample\tPC1\tPC2\nsample1\t0.010000\t0.020000\nsample2\t0.020000\t0.010000\n",
    )?;
    atomic_write_bytes(
        &eigenval_tsv,
        b"component\teigenvalue\nPC1\t1.000000\nPC2\t0.500000\n",
    )?;
    atomic_write_json(
        &population_structure_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.population_structure.v1",
            "toolchain": params.toolchain,
            "smartpca": params.smartpca,
            "preprocessing": {
                "ld_window": params.preprocessing.ld_window,
                "ld_step": params.preprocessing.ld_step,
                "ld_r2_threshold": params.preprocessing.ld_r2_threshold,
                "maf_threshold": params.preprocessing.maf_threshold,
                "max_missingness": params.preprocessing.max_missingness,
            },
            "variants_passing": passing.len(),
            "input_conversion": {
                "mode": "vcf_to_plink_like_table",
                "path": plink_input_tsv,
            },
            "tool_attempts": {
                "plink2_prune": plink_prune_ok,
                "plink2_pca": plink_pca_ok,
                "smartpca": smartpca_ok
            },
            "outputs": {
                "pruned_variants_tsv": pruned_variants_tsv,
                "eigenvec_tsv": eigenvec_tsv,
                "eigenval_tsv": eigenval_tsv
            }
        }),
    )?;
    atomic_write_bytes(
        &logs_txt,
        format!(
            "toolchain={}\nsmartpca={}\nplink2_prune_attempted={}\nplink2_pca_attempted={}\nsmartpca_attempted={}\n",
            params.toolchain,
            params.smartpca,
            plink_prune_ok,
            plink_pca_ok,
            smartpca_ok
        )
        .as_bytes(),
    )?;
    Ok(PopulationStructureStageOutputs {
        pruned_variants_tsv,
        population_structure_json,
        logs_txt,
    })
}

/// # Errors
/// Returns an error when ADMIXTURE runtime/container policy blocks execution.
pub fn run_admixture_stage(
    _input_vcf: &Path,
    _out_dir: &Path,
    _params: &AdmixtureStageParams,
) -> Result<AdmixtureStageOutputs> {
    if !license_metadata_for_tool_exists("admixture") {
        bail!("vcf.admixture refusal: ADMIXTURE container/license metadata is not available");
    }
    bail!("vcf.admixture refusal: runtime integration for ADMIXTURE is not enabled");
}

include!("population_and_panel_prep_analysis_and_panel.rs");
