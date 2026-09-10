//! Owner: bijux-dna-bench-population
//! Pca fixtures proxy.
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_corpus::vcf::SampleMetadataRow;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) fn materialize_proxy_eigensoft_outputs(
    input_vcf_path: &Path,
    metadata_by_sample: &BTreeMap<String, SampleMetadataRow>,
    stage_root: &Path,
) -> Result<()> {
    let convertf_par_path = stage_root.join("pca.convertf.par");
    let smartpca_par_path = stage_root.join("pca.smartpca.par");
    let geno_path = stage_root.join("pca.geno");
    let snp_path = stage_root.join("pca.snp");
    let ind_path = stage_root.join("pca.ind");

    let convertf_payload = format!(
        "genotypename: {input}\n\
snpname: {snp}\n\
indivname: {ind}\n\
outputformat: EIGENSTRAT\n\
genotypeoutname: {geno}\n\
snpoutname: {snp}\n\
indivoutname: {ind}\n\
familynames: NO\n",
        input = input_vcf_path.display(),
        geno = geno_path.display(),
        snp = snp_path.display(),
        ind = ind_path.display(),
    );
    bijux_dna_infra::atomic_write_bytes(&convertf_par_path, convertf_payload.as_bytes())?;

    if !smartpca_par_path.is_file() {
        let smartpca_payload = format!(
            "genotypename: {geno}\n\
snpname: {snp}\n\
indivname: {ind}\n\
evecoutname: {evec}\n\
evaloutname: {eval}\n\
numoutevec: 2\n\
familynames: NO\n",
            geno = geno_path.display(),
            snp = snp_path.display(),
            ind = ind_path.display(),
            evec = stage_root.join("eigenvec.tsv").display(),
            eval = stage_root.join("eigenval.tsv").display(),
        );
        bijux_dna_infra::atomic_write_bytes(&smartpca_par_path, smartpca_payload.as_bytes())?;
    }

    let raw = fs::read_to_string(input_vcf_path)
        .with_context(|| format!("read {}", input_vcf_path.display()))?;
    let mut sample_ids = Vec::<String>::new();
    let mut geno_lines = Vec::<String>::new();
    let mut snp_lines = Vec::<String>::new();
    for line in raw.lines() {
        if line.starts_with("#CHROM\t") {
            sample_ids = line.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>();
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            continue;
        }
        let variant_id = format!("{}_{}", fields[0], fields[1]);
        snp_lines.push(format!(
            "{variant_id}\t{}\t0.0\t{}\t{}\t{}",
            fields[0], fields[1], fields[3], fields[4]
        ));
        let genotype_codes = fields[9..]
            .iter()
            .map(|sample| {
                match sample.split(':').next().unwrap_or_default().replace('|', "/").as_str() {
                    "0/0" => '0',
                    "0/1" | "1/0" => '1',
                    "1/1" => '2',
                    _ => '9',
                }
            })
            .collect::<String>();
        geno_lines.push(genotype_codes);
    }
    let ind_lines = sample_ids
        .iter()
        .map(|sample_id| {
            let population_id = metadata_by_sample
                .get(sample_id)
                .map_or("unknown", |row| row.population_id.as_str());
            format!("{sample_id}\tU\t{population_id}")
        })
        .collect::<Vec<_>>();
    bijux_dna_infra::atomic_write_bytes(
        &geno_path,
        format!("{}\n", geno_lines.join("\n")).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &snp_path,
        format!("{}\n", snp_lines.join("\n")).as_bytes(),
    )?;
    bijux_dna_infra::atomic_write_bytes(
        &ind_path,
        format!("{}\n", ind_lines.join("\n")).as_bytes(),
    )?;
    Ok(())
}
