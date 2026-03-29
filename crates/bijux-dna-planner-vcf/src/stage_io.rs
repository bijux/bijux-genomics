use std::path::Path;

use bijux_dna_core::ids::ArtifactId;
use bijux_dna_core::prelude::{ArtifactRole, ArtifactSpec, CommandSpecV1};
use bijux_dna_domain_vcf::taxonomy::VcfDomainStage;

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
    let role = if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
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
