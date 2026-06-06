use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
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
    call_bam: Option<&Path>,
    call_bam_index: Option<&Path>,
    reference_fasta: Option<&Path>,
    reference_panel_vcf: Option<&Path>,
) -> Result<Vec<ArtifactSpec>> {
    match stage {
        VcfDomainStage::PrepareReferencePanel => Ok(vec![ArtifactSpec::required(
            ArtifactId::new("reference_panel_vcf"),
            require_path("reference_panel_vcf", reference_panel_vcf, stage)?,
            ArtifactRole::Variant,
        )]),
        VcfDomainStage::Call
        | VcfDomainStage::CallDiploid
        | VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid => Ok(vec![
            ArtifactSpec::required(
                ArtifactId::new("input_bam"),
                require_path("call_bam", call_bam, stage)?,
                ArtifactRole::Bam,
            ),
            ArtifactSpec::required(
                ArtifactId::new("input_bam_index"),
                require_path("call_bam_index", call_bam_index, stage)?,
                ArtifactRole::Index,
            ),
            ArtifactSpec::required(
                ArtifactId::new("reference_fasta"),
                require_path("reference_fasta", reference_fasta, stage)?,
                ArtifactRole::Reference,
            ),
        ]),
        VcfDomainStage::Demography => Ok(vec![ArtifactSpec::required(
            ArtifactId::new("ibd_segments"),
            out_dir.join("ibd_segments.json"),
            ArtifactRole::MetricsJson,
        )]),
        _ => Ok(vec![ArtifactSpec::required(
            ArtifactId::new("vcf"),
            current_vcf.to_path_buf(),
            ArtifactRole::Variant,
        )]),
    }
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
        ArtifactRole::Variant
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

pub(crate) fn stage_command(
    stage: VcfDomainStage,
    tool: &str,
    inputs: &[ArtifactSpec],
    outputs: &[ArtifactSpec],
) -> Result<CommandSpecV1> {
    if tool == "bcftools" {
        if let Some(template) = bcftools_stage_command(stage, inputs, outputs)? {
            return Ok(CommandSpecV1 { template });
        }
    }

    let mut template = vec![tool.to_string()];
    match stage {
        VcfDomainStage::Phasing => {
            template.extend(["phase", "--input", "vcf"].into_iter().map(str::to_string))
        }
        VcfDomainStage::Imputation | VcfDomainStage::Impute => {
            template.extend(["impute", "--input", "vcf"].into_iter().map(str::to_string))
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
        VcfDomainStage::Demography => {
            template.extend(["estimate-ne", "--from-ibd"].into_iter().map(str::to_string))
        }
        _ => template.push("--help".to_string()),
    }
    Ok(CommandSpecV1 { template })
}

fn require_path(field: &str, path: Option<&Path>, stage: VcfDomainStage) -> Result<PathBuf> {
    path.map(Path::to_path_buf).ok_or_else(|| {
        anyhow!(
            "planner refusal: {} requires `{field}` in VcfPipelineInputs",
            stage.as_str()
        )
    })
}

fn input_path<'a>(inputs: &'a [ArtifactSpec], artifact_id: &str) -> Result<&'a Path> {
    inputs
        .iter()
        .find(|input| input.name.as_str() == artifact_id)
        .map(|input| input.path.as_path())
        .ok_or_else(|| anyhow!("VCF stage command is missing required input `{artifact_id}`"))
}

fn output_path<'a>(outputs: &'a [ArtifactSpec], artifact_id: &str) -> Result<&'a Path> {
    outputs
        .iter()
        .find(|output| output.name.as_str() == artifact_id)
        .map(|output| output.path.as_path())
        .ok_or_else(|| anyhow!("VCF stage command is missing required output `{artifact_id}`"))
}

fn bcftools_stage_command(
    stage: VcfDomainStage,
    inputs: &[ArtifactSpec],
    outputs: &[ArtifactSpec],
) -> Result<Option<Vec<String>>> {
    let template = match stage {
        VcfDomainStage::PrepareReferencePanel => vec![
            "bcftools".to_string(),
            "norm".to_string(),
            "-m-any".to_string(),
            input_path(inputs, "reference_panel_vcf")?.display().to_string(),
            "-Oz".to_string(),
            "-o".to_string(),
            output_path(outputs, "prepared_panel")?.display().to_string(),
        ],
        VcfDomainStage::Call => shell_pipeline_command(
            "bcftools mpileup -Ou -f '{reference}' '{bam}' | bcftools call -c -Oz -o '{output}'",
            inputs,
            outputs,
            "called_vcf",
        )?,
        VcfDomainStage::CallDiploid => shell_pipeline_command(
            "bcftools mpileup -Ou -f '{reference}' '{bam}' | bcftools call -mv -Oz -o '{output}'",
            inputs,
            outputs,
            "diploid_vcf",
        )?,
        VcfDomainStage::CallGl => shell_pipeline_command(
            "bcftools mpileup -Ou -f '{reference}' '{bam}' | bcftools call -Aim -Oz -o '{output}'",
            inputs,
            outputs,
            "gl_sites_vcf",
        )?,
        VcfDomainStage::CallPseudohaploid => shell_pipeline_command(
            "bcftools mpileup -Ou -f '{reference}' '{bam}' | bcftools call --ploidy 1 -mv -Oz -o '{output}'",
            inputs,
            outputs,
            "pseudohaploid_vcf",
        )?,
        VcfDomainStage::DamageFilter => vec![
            "bcftools".to_string(),
            "filter".to_string(),
            "-e".to_string(),
            "((REF=\"C\" && ALT=\"T\") || (REF=\"G\" && ALT=\"A\")) && INFO/PMD>3".to_string(),
            input_path(inputs, "vcf")?.display().to_string(),
            "-Oz".to_string(),
            "-o".to_string(),
            output_path(outputs, "damage_filtered_vcf")?.display().to_string(),
        ],
        VcfDomainStage::Filter => vec![
            "bcftools".to_string(),
            "filter".to_string(),
            "-s".to_string(),
            "LOWQUAL".to_string(),
            "-e".to_string(),
            "QUAL<30".to_string(),
            input_path(inputs, "vcf")?.display().to_string(),
            "-Oz".to_string(),
            "-o".to_string(),
            output_path(outputs, "filtered_vcf")?.display().to_string(),
        ],
        VcfDomainStage::GlPropagation => vec![
            "bcftools".to_string(),
            "annotate".to_string(),
            "-x".to_string(),
            "INFO,^FORMAT/GL,^FORMAT/PL,^FORMAT/GP".to_string(),
            input_path(inputs, "vcf")?.display().to_string(),
            "-Oz".to_string(),
            "-o".to_string(),
            output_path(outputs, "gl_propagated_vcf")?.display().to_string(),
        ],
        VcfDomainStage::Postprocess => vec![
            "bcftools".to_string(),
            "+fill-tags".to_string(),
            input_path(inputs, "vcf")?.display().to_string(),
            "-Oz".to_string(),
            "-o".to_string(),
            output_path(outputs, "postprocess_vcf")?.display().to_string(),
            "--".to_string(),
            "-t".to_string(),
            "AC,AN,AF".to_string(),
        ],
        VcfDomainStage::Stats => vec![
            "bcftools".to_string(),
            "stats".to_string(),
            "-s".to_string(),
            "-".to_string(),
            "-o".to_string(),
            output_path(outputs, "stats_json")?.display().to_string(),
            input_path(inputs, "vcf")?.display().to_string(),
        ],
        _ => return Ok(None),
    };
    Ok(Some(template))
}

fn shell_pipeline_command(
    template: &str,
    inputs: &[ArtifactSpec],
    outputs: &[ArtifactSpec],
    output_artifact_id: &str,
) -> Result<Vec<String>> {
    let command = template
        .replace(
            "{reference}",
            &input_path(inputs, "reference_fasta")?.display().to_string(),
        )
        .replace("{bam}", &input_path(inputs, "input_bam")?.display().to_string())
        .replace(
            "{output}",
            &output_path(outputs, output_artifact_id)?.display().to_string(),
        );
    Ok(vec!["sh".to_string(), "-lc".to_string(), command])
}
