use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_core::ids::ArtifactId;
use bijux_dna_core::prelude::{
    ArtifactRole, ArtifactSpec, CommandSpecV1, ContainerImageRefV1,
};
use bijux_dna_domain_vcf::taxonomy::{CoverageRegime, VcfDomainStage};

pub(crate) fn stage_compat_tools(stage: VcfDomainStage) -> &'static [&'static str] {
    match stage {
        VcfDomainStage::Call => &["bcftools"],
        VcfDomainStage::CallDiploid => &["bcftools"],
        VcfDomainStage::CallGl => &["angsd", "bcftools"],
        VcfDomainStage::CallPseudohaploid => &["angsd", "bcftools"],
        VcfDomainStage::DamageFilter => &["bcftools", "angsd"],
        VcfDomainStage::Filter => &["bcftools"],
        VcfDomainStage::GlPropagation => &["bcftools", "angsd"],
        VcfDomainStage::PrepareReferencePanel => &["bcftools"],
        VcfDomainStage::Phasing => &["beagle", "eagle", "shapeit5"],
        VcfDomainStage::Imputation | VcfDomainStage::Impute => {
            &["glimpse", "impute5", "minimac4", "beagle"]
        }
        VcfDomainStage::Postprocess => &["bcftools"],
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca => &["plink2", "eigensoft"],
        VcfDomainStage::Admixture => &["plink2"],
        VcfDomainStage::Ibd => &["germline", "ibdhap"],
        VcfDomainStage::Roh => &["plink2"],
        VcfDomainStage::Demography => &["ibdne"],
        VcfDomainStage::Qc => &["plink2", "plink"],
        VcfDomainStage::Stats => &["bcftools"],
    }
}

pub(crate) fn default_tool(stage: VcfDomainStage, coverage: CoverageRegime) -> &'static str {
    match stage {
        VcfDomainStage::CallGl => "angsd",
        VcfDomainStage::CallPseudohaploid => "angsd",
        VcfDomainStage::Phasing => match coverage {
            CoverageRegime::Diploid => "shapeit5",
            CoverageRegime::LowCovGl => "beagle",
            CoverageRegime::Pseudohaploid => "beagle",
        },
        VcfDomainStage::Imputation | VcfDomainStage::Impute => match coverage {
            CoverageRegime::Diploid => "minimac4",
            CoverageRegime::LowCovGl => "glimpse",
            CoverageRegime::Pseudohaploid => "beagle",
        },
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca | VcfDomainStage::Roh => {
            "plink2"
        }
        VcfDomainStage::Ibd => "germline",
        VcfDomainStage::Demography => "ibdne",
        _ => "bcftools",
    }
}

pub(crate) fn image_for_tool(tool: &str) -> ContainerImageRefV1 {
    let image = match tool {
        "angsd" => "quay.io/biocontainers/angsd:0.940--h2e03b76_2",
        "shapeit5" => "quay.io/biocontainers/shapeit5:5.1.1--h9948957_0",
        "eagle" => "quay.io/biocontainers/eagle:2.4.1--h8b12597_2",
        "beagle" => "quay.io/biocontainers/beagle:5.4--hdfd78af_0",
        "glimpse" => "quay.io/biocontainers/glimpse:2.0.0--h9ee0642_0",
        "impute5" => "quay.io/biocontainers/impute5:1.2.0--h43eeafb_4",
        "minimac4" => "quay.io/biocontainers/minimac4:4.1.6--h7d875b9_4",
        "plink" => "quay.io/biocontainers/plink:1.90b6.21--h0a44026_2",
        "plink2" => "quay.io/biocontainers/plink2:2.00a3.7--h5ef6573_0",
        "eigensoft" => "quay.io/biocontainers/eigensoft:7.2.1--h9ee0642_4",
        "germline" => "quay.io/biocontainers/germline:1.5.3--hdfd78af_0",
        "ibdhap" => "quay.io/biocontainers/ibdhap:1.0.0--h9ee0642_0",
        "ibdne" => "quay.io/biocontainers/ibdne:23.05.23.ae9f5b3--hdfd78af_0",
        _ => "quay.io/biocontainers/bcftools:1.20--h8b25389_0",
    };
    ContainerImageRefV1 {
        image: image.to_string(),
        digest: None,
    }
}

pub(crate) fn stage_output_name(stage: VcfDomainStage) -> &'static str {
    match stage {
        VcfDomainStage::PrepareReferencePanel => "prepared_panel",
        VcfDomainStage::Call => "called_vcf",
        VcfDomainStage::CallDiploid => "diploid_vcf",
        VcfDomainStage::CallGl => "gl_sites_vcf",
        VcfDomainStage::CallPseudohaploid => "pseudohaploid_vcf",
        VcfDomainStage::DamageFilter => "damage_filtered_vcf",
        VcfDomainStage::Filter => "filtered_vcf",
        VcfDomainStage::GlPropagation => "gl_propagated_vcf",
        VcfDomainStage::Phasing => "phased_vcf",
        VcfDomainStage::Imputation | VcfDomainStage::Impute => "imputed_vcf",
        VcfDomainStage::Postprocess => "postprocess_vcf",
        VcfDomainStage::PopulationStructure => "population_structure_report",
        VcfDomainStage::Pca => "pca_report",
        VcfDomainStage::Admixture => "admixture_report",
        VcfDomainStage::Ibd => "ibd_segments",
        VcfDomainStage::Roh => "roh_report",
        VcfDomainStage::Demography => "demography_report",
        VcfDomainStage::Qc => "qc_report",
        VcfDomainStage::Stats => "stats_json",
    }
}

pub(crate) fn stage_inputs_for(
    stage: VcfDomainStage,
    current_vcf: &Path,
    out_dir: &Path,
) -> Vec<ArtifactSpec> {
    let input_path = match stage {
        VcfDomainStage::PrepareReferencePanel => out_dir.join("panel.vcf.gz"),
        VcfDomainStage::Demography => out_dir.join("ibd_segments.json"),
        _ => current_vcf.to_path_buf(),
    };
    let role = if matches!(stage, VcfDomainStage::Demography) {
        ArtifactRole::MetricsJson
    } else {
        ArtifactRole::Reads
    };
    vec![ArtifactSpec::required(
        ArtifactId::new("vcf"),
        input_path,
        role,
    )]
}

pub(crate) fn stage_outputs_for(stage: VcfDomainStage, out_dir: &Path) -> Vec<ArtifactSpec> {
    let output = stage_output_name(stage);
    let path =
        if output.ends_with("json") || output.contains("report") || output.contains("segments") {
            out_dir.join(format!("{output}.json"))
        } else {
            out_dir.join(format!("{output}.vcf.gz"))
        };
    let role = if path.extension().and_then(|e| e.to_str()) == Some("json") {
        ArtifactRole::MetricsJson
    } else {
        ArtifactRole::Reads
    };
    let mut outputs = vec![ArtifactSpec::required(ArtifactId::new(output), path, role)];
    if stage == VcfDomainStage::PrepareReferencePanel {
        outputs.push(ArtifactSpec::required(
            ArtifactId::new("chunks_json"),
            out_dir.join("chunks.json"),
            ArtifactRole::MetricsJson,
        ));
    }
    outputs
}

pub(crate) fn stage_command(stage: VcfDomainStage, tool: &str) -> CommandSpecV1 {
    let mut template = vec![tool.to_string()];
    match stage {
        VcfDomainStage::PrepareReferencePanel => {
            template.extend(["prepare-panel", "--lock"].into_iter().map(str::to_string))
        }
        VcfDomainStage::Phasing => {
            template.extend(["phase", "--input", "vcf"].into_iter().map(str::to_string))
        }
        VcfDomainStage::Imputation | VcfDomainStage::Impute => {
            template.extend(["impute", "--input", "vcf"].into_iter().map(str::to_string))
        }
        VcfDomainStage::GlPropagation => template.extend(
            ["annotate", "--retain", "GL,PL,GP"]
                .into_iter()
                .map(str::to_string),
        ),
        VcfDomainStage::DamageFilter => {
            template.extend(["filter", "--damage-aware"].into_iter().map(str::to_string))
        }
        VcfDomainStage::PopulationStructure => {
            template.extend(["pca", "--structure"].into_iter().map(str::to_string))
        }
        VcfDomainStage::Ibd => {
            template.extend(["ibd", "--min-seg", "3.0"].into_iter().map(str::to_string))
        }
        VcfDomainStage::Roh => {
            template.extend(["roh", "--min-kb", "500"].into_iter().map(str::to_string))
        }
        VcfDomainStage::Demography => template.extend(
            ["estimate-ne", "--from-ibd"]
                .into_iter()
                .map(str::to_string),
        ),
        _ => template.push("--help".to_string()),
    }
    CommandSpecV1 { template }
}

pub(crate) fn phasing_backend_supports_gl_only_input(tool: &str) -> bool {
    tool == "beagle"
}

pub(crate) fn eagle_license_metadata_present() -> bool {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("containers/licenses/eagle.license.toml")
        .exists()
}

pub(crate) fn resolve_requested_stages(
    requested_stages: &Option<Vec<String>>,
    resolved_coverage: CoverageRegime,
) -> Result<Vec<VcfDomainStage>> {
    if let Some(requested) = requested_stages {
        let mut out = Vec::new();
        for stage_id in requested {
            let stage = VcfDomainStage::try_from(stage_id.as_str())?;
            out.push(stage);
        }
        if out.is_empty() {
            anyhow::bail!("requested_stages resolved to empty set");
        }
        return Ok(out);
    }
    Ok(match resolved_coverage {
        CoverageRegime::LowCovGl => vec![
            VcfDomainStage::PrepareReferencePanel,
            VcfDomainStage::CallGl,
            VcfDomainStage::DamageFilter,
            VcfDomainStage::GlPropagation,
            VcfDomainStage::Filter,
            VcfDomainStage::Impute,
            VcfDomainStage::Postprocess,
            VcfDomainStage::PopulationStructure,
            VcfDomainStage::Ibd,
            VcfDomainStage::Demography,
            VcfDomainStage::Stats,
        ],
        CoverageRegime::Diploid => vec![
            VcfDomainStage::PrepareReferencePanel,
            VcfDomainStage::CallDiploid,
            VcfDomainStage::DamageFilter,
            VcfDomainStage::Filter,
            VcfDomainStage::Phasing,
            VcfDomainStage::Impute,
            VcfDomainStage::Postprocess,
            VcfDomainStage::PopulationStructure,
            VcfDomainStage::Roh,
            VcfDomainStage::Ibd,
            VcfDomainStage::Demography,
            VcfDomainStage::Stats,
        ],
        CoverageRegime::Pseudohaploid => vec![
            VcfDomainStage::CallPseudohaploid,
            VcfDomainStage::DamageFilter,
            VcfDomainStage::Filter,
            VcfDomainStage::Roh,
            VcfDomainStage::Stats,
        ],
    })
}
