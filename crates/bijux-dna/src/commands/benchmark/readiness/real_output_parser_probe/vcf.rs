use crate::commands::benchmark::local_vcf_call_smoke::run_local_vcf_call_smoke;
use crate::commands::benchmark::local_vcf_demography_smoke::run_local_vcf_demography_smoke;
use crate::commands::benchmark::local_vcf_ibd_smoke::run_local_vcf_ibd_smoke;
use crate::commands::benchmark::local_vcf_impute_smoke::run_local_vcf_impute_smoke;
use crate::commands::benchmark::local_vcf_phasing_smoke::run_local_vcf_phasing_smoke;
use crate::commands::benchmark::local_vcf_population_structure_smoke::run_local_vcf_population_structure_smoke;
use crate::commands::benchmark::local_vcf_qc_smoke::run_local_vcf_qc_smoke;
use anyhow::Result;
use bijux_dna_bench_parser_proofs::ports::MaterializedProof;
use std::path::Path;

pub(super) fn generate_vcf_call_probe(repo_root: &Path) -> Result<MaterializedProof> {
    let report = run_local_vcf_call_smoke(repo_root, "bcftools")?;
    Ok(MaterializedProof {
        proof_path: repo_root.join(&report.metrics_path),
        observed_tool_id: report.tool_id,
    })
}

pub(super) fn generate_vcf_demography_probe(repo_root: &Path) -> Result<MaterializedProof> {
    let report = run_local_vcf_demography_smoke(repo_root, "ibdne")?;
    Ok(MaterializedProof {
        proof_path: repo_root.join(&report.demography_json_path),
        observed_tool_id: report.tool_id,
    })
}

pub(super) fn generate_vcf_ibd_probe(repo_root: &Path) -> Result<MaterializedProof> {
    let report = run_local_vcf_ibd_smoke(repo_root, "germline")?;
    Ok(MaterializedProof {
        proof_path: repo_root.join(&report.ibd_json_path),
        observed_tool_id: report.tool_id,
    })
}

pub(super) fn generate_vcf_impute_probe(repo_root: &Path) -> Result<MaterializedProof> {
    let report = run_local_vcf_impute_smoke(repo_root, "beagle")?;
    Ok(MaterializedProof {
        proof_path: repo_root.join(&report.metrics_path),
        observed_tool_id: report.tool_id,
    })
}

pub(super) fn generate_vcf_phasing_probe(repo_root: &Path) -> Result<MaterializedProof> {
    let report = run_local_vcf_phasing_smoke(repo_root, "shapeit5")?;
    Ok(MaterializedProof {
        proof_path: repo_root.join(&report.metrics_path),
        observed_tool_id: report.tool_id,
    })
}

pub(super) fn generate_vcf_population_structure_probe(
    repo_root: &Path,
) -> Result<MaterializedProof> {
    let report = run_local_vcf_population_structure_smoke(repo_root, "plink2")?;
    Ok(MaterializedProof {
        proof_path: repo_root.join(&report.population_structure_json_path),
        observed_tool_id: report.tool_id,
    })
}

pub(super) fn generate_vcf_qc_probe(repo_root: &Path) -> Result<MaterializedProof> {
    let report = run_local_vcf_qc_smoke(repo_root, "plink")?;
    Ok(MaterializedProof {
        proof_path: repo_root.join(&report.metrics_path),
        observed_tool_id: report.tool_id,
    })
}
