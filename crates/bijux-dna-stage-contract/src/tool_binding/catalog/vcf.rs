//! Product-supported distributable VCF tool bindings.

use bijux_dna_core::contract::ToolConstraints;

use super::{
    input, output, resources, retained_binding, ToolBindingV1, BAM, FASTA, TABLE, VCF, VCF_PANEL,
};

pub(super) fn canonical_distributable_vcf_bindings() -> Vec<ToolBindingV1> {
    vec![
        vcf_binding(
            "vcf.admixture",
            "plink",
            &[input("cohort_vcf", VCF), input("sample_metadata", TABLE)],
            &[prefix_output("admixture_report", TABLE)],
            &["${executable}", "--vcf", "${input:cohort_vcf}", "--within", "${input:sample_metadata}", "--cluster", "--mds-plot", "10", "--out", "${output:admixture_report}"],
            resources(2, 4, 8),
        ),
        vcf_binding(
            "vcf.admixture",
            "plink2",
            &[input("cohort_vcf", VCF), input("sample_metadata", TABLE)],
            &[prefix_output("admixture_report", TABLE)],
            &["${executable}", "--vcf", "${input:cohort_vcf}", "--pheno", "${input:sample_metadata}", "--pca", "10", "approx", "--out", "${output:admixture_report}"],
            resources(2, 4, 8),
        ),
        bcftools_call("vcf.call", "called_vcf", &["-m", "-v"]),
        bcftools_call("vcf.call_diploid", "diploid_vcf", &["-m", "-v", "--ploidy", "2"]),
        bcftools_call("vcf.call_gl", "gl_sites_vcf", &["-m", "-v", "-a", "FORMAT/GL"]),
        bcftools_call(
            "vcf.call_pseudohaploid",
            "pseudohaploid_vcf",
            &["-m", "-v", "--ploidy", "1"],
        ),
        bcftools_transform(
            "vcf.damage_filter",
            "damage_filtered_vcf",
            &["filter", "--exclude", "REF='C'&&ALT='T'||REF='G'&&ALT='A'"],
        ),
        vcf_binding(
            "vcf.demography",
            "ibdne",
            &[input("ibd_segments", TABLE), input("genetic_map", TABLE)],
            &[prefix_output("demography_report", TABLE)],
            &["/bin/sh", "-c", "cat '${input:ibd_segments}' | '${executable}' map='${input:genetic_map}' out='${output:demography_report}' nthreads=8 nboots=0 seed=15052011"],
            resources(8, 16, 8),
        ),
        bcftools_transform(
            "vcf.filter",
            "filtered_vcf",
            &["filter", "--include", "QUAL>=20 && F_MISSING<0.1"],
        ),
        bcftools_transform(
            "vcf.gl_propagation",
            "gl_propagated_vcf",
            &["annotate", "--include", "FORMAT/GL!='.' || FORMAT/PL!='.'"],
        ),
        vcf_binding(
            "vcf.ibd",
            "germline",
            &[input("cohort_vcf", VCF)],
            &[output("ibd_segments", TABLE)],
            &["/bin/sh", "-c", "prefix='${output:ibd_segments}.cohort'; plink2 --vcf '${input:cohort_vcf}' --double-id --allow-extra-chr --recode ped --out \"$prefix\" && '${executable}' -silent -input \"$prefix.ped\" \"$prefix.map\" -output '${output:ibd_segments}' -bits 50 -min_m 1 -err_hom 2 -err_het 0 && mv '${output:ibd_segments}.match' '${output:ibd_segments}'"],
            resources(2, 4, 8),
        ),
        vcf_binding(
            "vcf.ibd",
            "ibdhap",
            &[input("cohort_vcf", VCF), input("genetic_map", TABLE)],
            &[output("ibd_segments", TABLE)],
            &["${executable}", "--vcf", "${input:cohort_vcf}", "--out", "${output:ibd_segments}", "--genetic-map", "${input:genetic_map}", "--sample", "sample_a", "--sample", "sample_b", "--region", "chr1:2-10", "--max-markers", "500"],
            resources(2, 4, 8),
        ),
        vcf_binding(
            "vcf.ibd",
            "ibdseq",
            &[input("cohort_vcf", VCF)],
            &[output("ibd_segments", TABLE)],
            &["${executable}", "gt=${input:cohort_vcf}", "out=${output:ibd_segments}", "nthreads=8"],
            resources(8, 16, 8),
        ),
        imputation_metrics("beagle"),
        imputation_metrics("glimpse"),
        imputation_metrics("minimac4"),
        imputation("beagle"),
        imputation("glimpse"),
        imputation("minimac4"),
        eigensoft_binding("vcf.pca", "pca_report"),
        vcf_binding(
            "vcf.pca",
            "plink2",
            &[input("cohort_vcf", VCF)],
            &[prefix_output("pca_report", TABLE)],
            &["${executable}", "--vcf", "${input:cohort_vcf}", "--pca", "10", "approx", "--out", "${output:pca_report}"],
            resources(2, 4, 8),
        ),
        phasing("beagle"),
        phasing("eagle"),
        phasing("shapeit"),
        phasing("shapeit5"),
        population_structure("eigensoft"),
        population_structure("plink"),
        population_structure("plink2"),
        bcftools_transform(
            "vcf.postprocess",
            "postprocessed_vcf",
            &["norm", "--multiallelics", "-both"],
        ),
        bcftools_transform(
            "vcf.prepare_reference_panel",
            "reference_panel_vcf",
            &["norm", "--check-ref", "warn"],
        ),
        vcf_binding(
            "vcf.qc",
            "bcftools",
            &[input("cohort_vcf", VCF)],
            &[prefix_output("qc_report", TABLE)],
            &["${executable}", "stats", "--threads", "4", "--output", "${output:qc_report}", "${input:cohort_vcf}"],
            resources(4, 8, 8),
        ),
        plink_qc("plink"),
        plink_qc("plink2"),
        vcf_binding(
            "vcf.roh",
            "plink2",
            &[input("cohort_vcf", VCF)],
            &[prefix_output("roh_report", TABLE)],
            &["${executable}", "--vcf", "${input:cohort_vcf}", "--homozyg", "--out", "${output:roh_report}"],
            resources(2, 4, 8),
        ),
        vcf_binding(
            "vcf.stats",
            "bcftools",
            &[input("cohort_vcf", VCF)],
            &[output("stats_report", TABLE)],
            &["${executable}", "stats", "--threads", "4", "--output", "${output:stats_report}", "${input:cohort_vcf}"],
            resources(4, 8, 8),
        ),
    ]
}

fn bcftools_call(stage_id: &str, output_name: &str, call_arguments: &[&str]) -> ToolBindingV1 {
    let call_arguments = call_arguments.join(" ");
    vcf_binding_owned(
        stage_id,
        "bcftools",
        &[input("bam", BAM), input("reference", FASTA)],
        &[output(output_name, VCF)],
        &[
            "/bin/sh".to_string(),
            "-c".to_string(),
            format!(
                "'${{executable}}' mpileup -f '${{input:reference}}' -Ou '${{input:bam}}' | '${{executable}}' call {call_arguments} -Oz -o '${{output:{output_name}}}' && '${{executable}}' index -f '${{output:{output_name}}}'"
            ),
        ],
        resources(8, 16, 16),
    )
}

fn bcftools_transform(stage_id: &str, output_name: &str, arguments: &[&str]) -> ToolBindingV1 {
    let inputs = [input("cohort_vcf", VCF)];
    let outputs = [output(output_name, VCF)];
    let arguments = arguments.join(" ");
    let command = vec![
        "/bin/sh".to_string(),
        "-c".to_string(),
        format!(
            "'${{executable}}' {arguments} --output-type z --output '${{output:{output_name}}}' '${{input:cohort_vcf}}' && '${{executable}}' index -f '${{output:{output_name}}}'"
        ),
    ];
    vcf_binding_owned(stage_id, "bcftools", &inputs, &outputs, &command, resources(4, 8, 8))
}

fn imputation_metrics(tool_id: &str) -> ToolBindingV1 {
    let inputs = if tool_id == "minimac4" {
        vec![
            input("imputed_vcf", VCF),
            input("reference_panel_msav", VCF_PANEL),
            input("genetic_map", TABLE),
        ]
    } else {
        vec![
            input("imputed_vcf", VCF),
            input("reference_panel_vcf", VCF_PANEL),
            input("genetic_map", TABLE),
        ]
    };
    let command = match tool_id {
        "beagle" => vec![
            "/bin/sh",
            "-c",
            "'${executable}' gt='${input:imputed_vcf}' ref='${input:reference_panel_vcf}' map='${input:genetic_map}' out='${output:metrics_source}.prefix' nthreads=8 seed=42 && mv '${output:metrics_source}.prefix.vcf.gz' '${output:metrics_source}'",
        ],
        "glimpse" => vec![
            "${executable}",
            "phase",
            "--input",
            "${input:imputed_vcf}",
            "--reference",
            "${input:reference_panel_vcf}",
            "--map",
            "${input:genetic_map}",
            "--output",
            "${output:metrics_source}",
            "--threads",
            "8",
        ],
        "minimac4" => vec![
            "${executable}",
            "--threads",
            "8",
            "--map",
            "${input:genetic_map}",
            "--output",
            "${output:metrics_source}",
            "${input:reference_panel_msav}",
            "${input:imputed_vcf}",
        ],
        _ => unreachable!("closed imputation metrics tool set"),
    };
    vcf_binding(
        "vcf.imputation_metrics",
        tool_id,
        &inputs,
        &[output("metrics_source", VCF)],
        &command,
        resources(8, 16, 8),
    )
}

fn imputation(tool_id: &str) -> ToolBindingV1 {
    let inputs = if tool_id == "minimac4" {
        vec![
            input("phased_vcf", VCF),
            input("reference_panel_msav", VCF_PANEL),
            input("genetic_map", TABLE),
        ]
    } else {
        vec![
            input("phased_vcf", VCF),
            input("reference_panel_vcf", VCF_PANEL),
            input("genetic_map", TABLE),
        ]
    };
    let command = match tool_id {
        "beagle" => vec![
            "/bin/sh",
            "-c",
            "'${executable}' gt='${input:phased_vcf}' ref='${input:reference_panel_vcf}' map='${input:genetic_map}' out='${output:imputed_vcf}.prefix' impute=true nthreads=8 seed=42 && mv '${output:imputed_vcf}.prefix.vcf.gz' '${output:imputed_vcf}'",
        ],
        "glimpse" => vec![
            "${executable}",
            "phase",
            "--input",
            "${input:phased_vcf}",
            "--reference",
            "${input:reference_panel_vcf}",
            "--map",
            "${input:genetic_map}",
            "--output",
            "${output:imputed_vcf}",
            "--threads",
            "8",
        ],
        "minimac4" => vec![
            "${executable}",
            "--threads",
            "8",
            "--map",
            "${input:genetic_map}",
            "--output",
            "${output:imputed_vcf}",
            "${input:reference_panel_msav}",
            "${input:phased_vcf}",
        ],
        _ => unreachable!("closed imputation tool set"),
    };
    vcf_binding(
        "vcf.impute",
        tool_id,
        &inputs,
        &[output("imputed_vcf", VCF)],
        &command,
        resources(8, 32, 32),
    )
}

fn phasing(tool_id: &str) -> ToolBindingV1 {
    let command = match tool_id {
        "beagle" => vec![
            "/bin/sh",
            "-c",
            "'${executable}' gt='${input:cohort_vcf}' ref='${input:reference_panel_vcf}' map='${input:genetic_map}' out='${output:phased_vcf}.prefix' nthreads=8 seed=42 && mv '${output:phased_vcf}.prefix.vcf.gz' '${output:phased_vcf}'",
        ],
        "eagle" => vec![
            "/bin/sh",
            "-c",
            "'${executable}' --vcfTarget '${input:cohort_vcf}' --vcfRef '${input:reference_panel_vcf}' --geneticMapFile '${input:genetic_map}' --outPrefix '${output:phased_vcf}.prefix' --numThreads 8 && mv '${output:phased_vcf}.prefix.vcf.gz' '${output:phased_vcf}'",
        ],
        "shapeit" => vec![
            "${executable}",
            "--input",
            "${input:cohort_vcf}",
            "--reference",
            "${input:reference_panel_vcf}",
            "--map",
            "${input:genetic_map}",
            "--region",
            "1:1-1000000",
            "--thread",
            "8",
            "--seed",
            "42",
            "--output",
            "${output:phased_vcf}",
        ],
        "shapeit5" => vec![
            "${executable}",
            "phase_common",
            "--input",
            "${input:cohort_vcf}",
            "--reference",
            "${input:reference_panel_vcf}",
            "--map",
            "${input:genetic_map}",
            "--region",
            "1:1-1000000",
            "--thread",
            "8",
            "--seed",
            "42",
            "--output",
            "${output:phased_vcf}",
        ],
        _ => unreachable!("closed phasing tool set"),
    };
    vcf_binding(
        "vcf.phasing",
        tool_id,
        &[
            input("cohort_vcf", VCF),
            input("reference_panel_vcf", VCF_PANEL),
            input("genetic_map", TABLE),
        ],
        &[output("phased_vcf", VCF)],
        &command,
        resources(8, 16, 8),
    )
}

fn population_structure(tool_id: &str) -> ToolBindingV1 {
    if tool_id == "eigensoft" {
        return eigensoft_binding("vcf.population_structure", "population_structure_report");
    }
    let command = match tool_id {
        "plink" => vec![
            "${executable}",
            "--vcf",
            "${input:cohort_vcf}",
            "--within",
            "${input:sample_metadata}",
            "--cluster",
            "--mds-plot",
            "10",
            "--out",
            "${output:population_structure_report}",
        ],
        "plink2" => vec![
            "${executable}",
            "--vcf",
            "${input:cohort_vcf}",
            "--pheno",
            "${input:sample_metadata}",
            "--pca",
            "10",
            "approx",
            "--out",
            "${output:population_structure_report}",
        ],
        _ => unreachable!("closed population structure tool set"),
    };
    vcf_binding(
        "vcf.population_structure",
        tool_id,
        &[input("cohort_vcf", VCF), input("sample_metadata", TABLE)],
        &[prefix_output("population_structure_report", TABLE)],
        &command,
        resources(2, 4, 8),
    )
}

fn plink_qc(tool_id: &str) -> ToolBindingV1 {
    vcf_binding(
        "vcf.qc",
        tool_id,
        &[input("cohort_vcf", VCF)],
        &[prefix_output("qc_report", TABLE)],
        &[
            "${executable}",
            "--vcf",
            "${input:cohort_vcf}",
            "--missing",
            "--freq",
            "--hardy",
            "--out",
            "${output:qc_report}",
        ],
        resources(2, 4, 8),
    )
}

fn eigensoft_binding(stage_id: &str, output_name: &str) -> ToolBindingV1 {
    let inputs = [input("cohort_vcf", VCF), input("sample_metadata", TABLE)];
    let outputs = [output(output_name, TABLE)];
    let command = vec![
        "/bin/sh".to_string(),
        "-c".to_string(),
        format!(
            "convertf_par='${{output:{output_name}}}.convertf.par'; smartpca_par='${{output:{output_name}}}.smartpca.par'; geno='${{output:{output_name}}}.geno'; snp='${{output:{output_name}}}.snp'; ind='${{output:{output_name}}}.ind'; printf 'genotypename: %s\\nsnpname: %s\\nindivname: %s\\noutputformat: EIGENSTRAT\\ngenotypeoutname: %s\\nsnpoutname: %s\\nindivoutname: %s\\nfamilynames: NO\\n' '${{input:cohort_vcf}}' \"$snp\" \"$ind\" \"$geno\" \"$snp\" \"$ind\" > \"$convertf_par\" && convertf -p \"$convertf_par\" && printf 'genotypename: %s\\nsnpname: %s\\nindivname: %s\\nevecoutname: %s\\nevaloutname: %s.eval\\npoplistname: %s\\nnumoutevec: 10\\nfamilynames: NO\\n' \"$geno\" \"$snp\" \"$ind\" '${{output:{output_name}}}' '${{output:{output_name}}}' '${{input:sample_metadata}}' > \"$smartpca_par\" && '${{executable}}' -p \"$smartpca_par\""
        ),
    ];
    vcf_binding_owned(stage_id, "eigensoft", &inputs, &outputs, &command, resources(2, 4, 8))
}

fn prefix_output(name: &str, artifact_contract_id: &str) -> super::BindingOutputV1 {
    super::BindingOutputV1 {
        name: name.to_string(),
        artifact_contract_id: artifact_contract_id.to_string(),
        discovery: super::OutputDiscoveryV1::Glob { pattern: format!("${{output:{name}}}*") },
    }
}

fn vcf_binding(
    stage_id: &str,
    tool_id: &str,
    inputs: &[super::BindingInputV1],
    outputs: &[super::BindingOutputV1],
    command_template: &[&str],
    resources: ToolConstraints,
) -> ToolBindingV1 {
    let normalized_output_contract_id = format!("bijux.benchmark.{stage_id}.bundle.v1");
    let normalizer_id = vcf_normalizer_id(stage_id, tool_id);
    let downstream_probe_id = format!("bijux.stage_probe.{stage_id}.v1");
    retained_binding(
        stage_id,
        tool_id,
        &["vcf_stage_input", "tool_native_output", "normalized_stage_result"],
        inputs,
        &[],
        outputs,
        command_template,
        &normalized_output_contract_id,
        normalizer_id,
        &downstream_probe_id,
        resources,
        &["raw_tool_output_is_preserved_before_normalization"],
    )
}

fn vcf_normalizer_id(stage_id: &str, tool_id: &str) -> &'static str {
    match (stage_id, tool_id) {
        ("vcf.admixture", "plink") => "parse_plink_admixture_prep_metrics",
        ("vcf.admixture", "plink2") => "parse_plink2_admixture_metrics",
        ("vcf.call", "bcftools") => "parse_bcftools_call_metrics",
        ("vcf.call_diploid", "bcftools") => "parse_bcftools_call_diploid_metrics",
        ("vcf.call_gl", "bcftools") => "parse_bcftools_call_gl_metrics",
        ("vcf.call_pseudohaploid", "bcftools") => "parse_bcftools_call_pseudohaploid_metrics",
        ("vcf.damage_filter", "bcftools") => "parse_bcftools_damage_filter_metrics",
        ("vcf.demography", "ibdne") => "parse_ibdne_demography_metrics",
        ("vcf.filter", "bcftools") => "parse_bcftools_filter_metrics",
        ("vcf.gl_propagation", "bcftools") => "parse_bcftools_gl_propagation_metrics",
        ("vcf.ibd", "germline") => "parse_germline_ibd_segment_metrics",
        ("vcf.ibd", "ibdhap") => "parse_ibdhap_ibd_segment_metrics",
        ("vcf.ibd", "ibdseq") => "parse_ibdseq_ibd_segment_metrics",
        ("vcf.imputation_metrics", "beagle") => "parse_beagle_imputation_metrics",
        ("vcf.imputation_metrics", "glimpse") => "parse_glimpse_imputation_metrics",
        ("vcf.imputation_metrics", "minimac4") => "parse_minimac4_imputation_metrics",
        ("vcf.impute", "beagle") => "parse_beagle_impute_metrics",
        ("vcf.impute", "glimpse") => "parse_glimpse_impute_metrics",
        ("vcf.impute", "minimac4") => "parse_minimac4_impute_metrics",
        ("vcf.pca", "eigensoft") => "parse_eigensoft_pca_metrics",
        ("vcf.pca", "plink2") => "parse_plink2_pca_metrics",
        ("vcf.phasing", "beagle") => "parse_beagle_phasing_metrics",
        ("vcf.phasing", "eagle") => "parse_eagle_phasing_metrics",
        ("vcf.phasing", "shapeit") => "parse_shapeit_phasing_metrics",
        ("vcf.phasing", "shapeit5") => "parse_shapeit5_phasing_metrics",
        ("vcf.population_structure", "eigensoft") => "parse_eigensoft_population_structure_metrics",
        ("vcf.population_structure", "plink") => "parse_plink_population_structure_metrics",
        ("vcf.population_structure", "plink2") => "parse_plink2_population_structure_metrics",
        ("vcf.postprocess", "bcftools") => "parse_bcftools_postprocess_metrics",
        ("vcf.prepare_reference_panel", "bcftools") => {
            "parse_bcftools_prepare_reference_panel_metrics"
        }
        ("vcf.qc", "bcftools") => "parse_bcftools_qc_metrics",
        ("vcf.qc", "plink") => "parse_plink_qc_metrics",
        ("vcf.qc", "plink2") => "parse_plink2_qc_metrics",
        ("vcf.roh", "plink2") => "parse_plink2_roh_segment_metrics",
        ("vcf.stats", "bcftools") => "parse_bcftools_stats_metrics",
        _ => unreachable!("closed distributable VCF binding set: {stage_id}/{tool_id}"),
    }
}

fn vcf_binding_owned(
    stage_id: &str,
    tool_id: &str,
    inputs: &[super::BindingInputV1],
    outputs: &[super::BindingOutputV1],
    command_template: &[String],
    resources: ToolConstraints,
) -> ToolBindingV1 {
    let command_refs = command_template.iter().map(String::as_str).collect::<Vec<_>>();
    vcf_binding(stage_id, tool_id, inputs, outputs, &command_refs, resources)
}
