//! Owner: bijux-dna-bench-corpus
//! VCF corpus variants ownership.
use super::{
    anyhow, execute_vcf_validation, fs, path_relative_to_repo, resolve_manifest_relative_path,
    BTreeSet, Path, ReferenceContig, Result, VcfDocumentSummary, VcfVariantRole,
    VcfVariantSetValidationReport,
};
use anyhow::Context;

pub(super) fn validate_variant_set(
    repo_root: &Path,
    manifest_dir: &Path,
    role: VcfVariantRole,
    relative_path: &Path,
    expected_sample_ids: &[String],
    reference_contigs: &[ReferenceContig],
) -> Result<(VcfVariantSetValidationReport, BTreeSet<String>)> {
    let vcf_path = resolve_manifest_relative_path(manifest_dir, relative_path);
    if !vcf_path.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture {} VCF is missing: {}",
            role.as_str(),
            vcf_path.display()
        ));
    }
    let expected_contigs =
        reference_contigs.iter().map(|contig| contig.name.as_str()).collect::<Vec<_>>();
    let validation_summary =
        execute_vcf_validation(&vcf_path, &expected_contigs, false, false, None, None)
            .with_context(|| format!("validate {}", vcf_path.display()))?;
    if !validation_summary.header_valid || !validation_summary.refusal_codes.is_empty() {
        return Err(anyhow!(
            "VCF corpus fixture {} VCF failed validation: {}",
            role.as_str(),
            validation_summary.refusal_codes.join(", ")
        ));
    }
    if validation_summary.record_count == 0 {
        return Err(anyhow!(
            "VCF corpus fixture {} VCF must contain at least one variant record",
            role.as_str()
        ));
    }

    let document = parse_vcf_document(&vcf_path, role)?;
    let expected_sample_ids_set = expected_sample_ids.iter().cloned().collect::<BTreeSet<_>>();
    let observed_sample_ids_set = document.sample_ids.iter().cloned().collect::<BTreeSet<_>>();
    if document.sample_ids != expected_sample_ids {
        return Err(anyhow!(
            "VCF corpus fixture {} VCF sample ids {:?} do not match expected {:?}",
            role.as_str(),
            document.sample_ids,
            expected_sample_ids
        ));
    }
    let expected_sample_count = u32::try_from(expected_sample_ids.len()).map_err(|_| {
        anyhow!("VCF corpus fixture {} expected sample count exceeds u32 capacity", role.as_str())
    })?;
    if validation_summary.sample_count != expected_sample_count {
        return Err(anyhow!(
            "VCF corpus fixture {} VCF reported {} samples, expected {}",
            role.as_str(),
            validation_summary.sample_count,
            expected_sample_ids.len()
        ));
    }

    let reference_contig_names =
        reference_contigs.iter().map(|contig| contig.name.as_str()).collect::<BTreeSet<_>>();
    for contig in &document.contigs {
        if !reference_contig_names.contains(contig.as_str()) {
            return Err(anyhow!(
                "VCF corpus fixture {} VCF references unknown contig `{contig}`",
                role.as_str()
            ));
        }
    }

    Ok((
        VcfVariantSetValidationReport {
            variant_role: role.as_str().to_string(),
            vcf_path: path_relative_to_repo(repo_root, &vcf_path),
            observed_sample_ids: document.sample_ids,
            observed_variant_count: document.variant_count,
            observed_contigs: document.contigs,
            phased_genotypes_only: document.phased_genotypes_only,
            valid: true,
        },
        observed_sample_ids_set.union(&expected_sample_ids_set).cloned().collect(),
    ))
}

pub(super) fn parse_vcf_document(
    vcf_path: &Path,
    role: VcfVariantRole,
) -> Result<VcfDocumentSummary> {
    let raw =
        fs::read_to_string(vcf_path).with_context(|| format!("read {}", vcf_path.display()))?;
    let mut sample_ids = None;
    let mut contigs = BTreeSet::new();
    let mut variant_count = 0u64;
    let mut phased_genotypes_only = false;

    for (line_number, line) in raw.lines().enumerate() {
        if let Some(payload) = line.strip_prefix("#CHROM\t") {
            let fields = payload.split('\t').collect::<Vec<_>>();
            if fields.len() < 9 {
                return Err(anyhow!(
                    "VCF corpus fixture {} is missing sample columns on #CHROM line",
                    vcf_path.display()
                ));
            }
            let observed = fields[8..].iter().map(|value| (*value).to_string()).collect::<Vec<_>>();
            let mut seen = BTreeSet::new();
            for sample_id in &observed {
                if sample_id.trim().is_empty() {
                    return Err(anyhow!(
                        "VCF corpus fixture {} contains an empty sample id in the #CHROM header",
                        vcf_path.display()
                    ));
                }
                if !seen.insert(sample_id.clone()) {
                    return Err(anyhow!(
                        "VCF corpus fixture {} repeats sample id `{sample_id}`",
                        vcf_path.display()
                    ));
                }
            }
            sample_ids = Some(observed);
            continue;
        }
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            return Err(anyhow!(
                "VCF corpus fixture {} line {} must contain at least 10 tab-delimited fields",
                vcf_path.display(),
                line_number + 1
            ));
        }
        contigs.insert(fields[0].to_string());
        variant_count += 1;
        if role == VcfVariantRole::Phased {
            let format_keys = fields[8].split(':').collect::<Vec<_>>();
            let gt_index = format_keys
                .iter()
                .position(|token| *token == "GT")
                .ok_or_else(|| anyhow!("VCF corpus fixture phased VCF is missing GT in FORMAT"))?;
            for sample_field in &fields[9..] {
                let sample_tokens = sample_field.split(':').collect::<Vec<_>>();
                let genotype = sample_tokens.get(gt_index).copied().unwrap_or(".");
                if genotype == "." || genotype == "./." || genotype == ".|." {
                    continue;
                }
                if !genotype.contains('|') {
                    return Err(anyhow!(
                        "VCF corpus fixture phased VCF contains an unphased genotype `{genotype}`"
                    ));
                }
            }
        }
    }

    let sample_ids = sample_ids.ok_or_else(|| {
        anyhow!("VCF corpus fixture {} is missing a #CHROM header line", vcf_path.display())
    })?;
    if variant_count == 0 {
        return Err(anyhow!(
            "VCF corpus fixture {} contains no variant records",
            vcf_path.display()
        ));
    }

    if role == VcfVariantRole::Phased {
        phased_genotypes_only = true;
    }

    Ok(VcfDocumentSummary {
        sample_ids,
        variant_count,
        contigs: contigs.into_iter().collect(),
        phased_genotypes_only,
    })
}
