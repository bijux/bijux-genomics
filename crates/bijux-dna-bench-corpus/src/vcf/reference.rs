//! Owner: bijux-dna-bench-corpus
//! VCF corpus reference ownership.
use super::{
    anyhow, fs, BTreeMap, BTreeSet, FastaIndexRow, Path, ReferenceContig, ReferenceDictRow, Result,
};
use anyhow::Context;

pub(super) fn load_validated_reference_contigs(
    reference_fasta: &Path,
    reference_fasta_index: &Path,
    reference_dict: &Path,
) -> Result<Vec<ReferenceContig>> {
    if !reference_fasta.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture reference FASTA is missing: {}",
            reference_fasta.display()
        ));
    }
    if !reference_fasta_index.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture reference FASTA index is missing: {}",
            reference_fasta_index.display()
        ));
    }
    if !reference_dict.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture reference dictionary is missing: {}",
            reference_dict.display()
        ));
    }

    let reference_contigs = parse_reference_fasta(reference_fasta)?;
    let index_rows = parse_reference_fasta_index(reference_fasta_index)?;
    let dict_rows = parse_reference_dict(reference_dict)?;
    if reference_contigs.len() != index_rows.len() {
        return Err(anyhow!(
            "VCF corpus fixture reference FASTA index count {} does not match FASTA contig count {}",
            index_rows.len(),
            reference_contigs.len()
        ));
    }
    if reference_contigs.len() != dict_rows.len() {
        return Err(anyhow!(
            "VCF corpus fixture reference dictionary count {} does not match FASTA contig count {}",
            dict_rows.len(),
            reference_contigs.len()
        ));
    }

    for (reference_contig, index_row) in reference_contigs.iter().zip(index_rows.iter()) {
        if reference_contig.name != index_row.name {
            return Err(anyhow!(
                "VCF corpus fixture FASTA index contig `{}` does not match FASTA contig `{}`",
                index_row.name,
                reference_contig.name
            ));
        }
        if reference_contig.length != index_row.length {
            return Err(anyhow!(
                "VCF corpus fixture FASTA index length for `{}` is {}, expected {}",
                index_row.name,
                index_row.length,
                reference_contig.length
            ));
        }
    }
    for (reference_contig, dict_row) in reference_contigs.iter().zip(dict_rows.iter()) {
        if reference_contig.name != dict_row.name {
            return Err(anyhow!(
                "VCF corpus fixture reference dictionary contig `{}` does not match FASTA contig `{}`",
                dict_row.name,
                reference_contig.name
            ));
        }
        if reference_contig.length != dict_row.length {
            return Err(anyhow!(
                "VCF corpus fixture reference dictionary length for `{}` is {}, expected {}",
                dict_row.name,
                dict_row.length,
                reference_contig.length
            ));
        }
    }

    Ok(reference_contigs)
}

pub(super) fn parse_reference_fasta(reference_fasta: &Path) -> Result<Vec<ReferenceContig>> {
    let raw = fs::read_to_string(reference_fasta)
        .with_context(|| format!("read {}", reference_fasta.display()))?;
    let mut contigs = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_length = 0usize;
    let mut seen = BTreeSet::new();

    for line in raw.lines() {
        if let Some(header) = line.strip_prefix('>') {
            if let Some(name) = current_name.take() {
                if current_length == 0 {
                    return Err(anyhow!(
                        "VCF corpus fixture reference contig `{name}` has no sequence bases"
                    ));
                }
                contigs.push(ReferenceContig { name, length: current_length });
            }
            let name = header.trim();
            if name.is_empty() {
                return Err(anyhow!(
                    "VCF corpus fixture reference FASTA contains an empty contig name"
                ));
            }
            if !seen.insert(name.to_string()) {
                return Err(anyhow!("VCF corpus fixture reference FASTA repeats contig `{name}`"));
            }
            current_name = Some(name.to_string());
            current_length = 0;
            continue;
        }

        let sequence = line.trim();
        if sequence.is_empty() {
            continue;
        }
        if current_name.is_none() {
            return Err(anyhow!(
                "VCF corpus fixture reference FASTA contains sequence data before the first header"
            ));
        }
        if !sequence
            .chars()
            .all(|base| matches!(base, 'A' | 'C' | 'G' | 'T' | 'N' | 'a' | 'c' | 'g' | 't' | 'n'))
        {
            return Err(anyhow!(
                "VCF corpus fixture reference FASTA contains non-IUPAC base content"
            ));
        }
        current_length += sequence.len();
    }

    if let Some(name) = current_name.take() {
        if current_length == 0 {
            return Err(anyhow!(
                "VCF corpus fixture reference contig `{name}` has no sequence bases"
            ));
        }
        contigs.push(ReferenceContig { name, length: current_length });
    }

    if contigs.is_empty() {
        return Err(anyhow!("VCF corpus fixture reference FASTA must declare at least one contig"));
    }
    Ok(contigs)
}

pub(super) fn parse_reference_fasta_index(
    reference_fasta_index: &Path,
) -> Result<Vec<FastaIndexRow>> {
    let raw = fs::read_to_string(reference_fasta_index)
        .with_context(|| format!("read {}", reference_fasta_index.display()))?;
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for (line_number, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 5 {
            return Err(anyhow!(
                "VCF corpus fixture FASTA index line {} must contain at least 5 tab-delimited fields",
                line_number + 1
            ));
        }
        let name = fields[0].trim();
        if name.is_empty() {
            return Err(anyhow!(
                "VCF corpus fixture FASTA index line {} contains an empty contig name",
                line_number + 1
            ));
        }
        if !seen.insert(name.to_string()) {
            return Err(anyhow!("VCF corpus fixture FASTA index repeats contig `{name}`"));
        }
        let length = fields[1].parse::<usize>().with_context(|| {
            format!(
                "VCF corpus fixture FASTA index line {} has invalid contig length `{}`",
                line_number + 1,
                fields[1]
            )
        })?;
        rows.push(FastaIndexRow { name: name.to_string(), length });
    }
    if rows.is_empty() {
        return Err(anyhow!("VCF corpus fixture FASTA index must declare at least one contig row"));
    }
    Ok(rows)
}

pub(super) fn parse_reference_dict(reference_dict: &Path) -> Result<Vec<ReferenceDictRow>> {
    let raw = fs::read_to_string(reference_dict)
        .with_context(|| format!("read {}", reference_dict.display()))?;
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for (line_number, line) in raw.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with("@HD") {
            continue;
        }
        if !line.starts_with("@SQ") {
            return Err(anyhow!(
                "VCF corpus fixture reference dictionary line {} must start with `@SQ` or `@HD`",
                line_number + 1
            ));
        }
        let name =
            line.split('\t').find_map(|field| field.strip_prefix("SN:")).ok_or_else(|| {
                anyhow!(
                    "VCF corpus fixture reference dictionary line {} is missing `SN:`",
                    line_number + 1
                )
            })?;
        let length = line
            .split('\t')
            .find_map(|field| field.strip_prefix("LN:"))
            .ok_or_else(|| {
                anyhow!(
                    "VCF corpus fixture reference dictionary line {} is missing `LN:`",
                    line_number + 1
                )
            })?
            .parse::<usize>()
            .with_context(|| {
                format!(
                    "parse reference dictionary length on line {} in {}",
                    line_number + 1,
                    reference_dict.display()
                )
            })?;
        if !seen.insert(name.to_string()) {
            return Err(anyhow!("VCF corpus fixture reference dictionary repeats contig `{name}`"));
        }
        rows.push(ReferenceDictRow { name: name.to_string(), length });
    }
    if rows.is_empty() {
        return Err(anyhow!(
            "VCF corpus fixture reference dictionary must declare at least one `@SQ` row"
        ));
    }
    Ok(rows)
}

pub(super) fn validate_target_sites_bed(
    target_sites_bed: &Path,
    reference_contigs: &[ReferenceContig],
) -> Result<usize> {
    if !target_sites_bed.is_file() {
        return Err(anyhow!(
            "VCF corpus fixture target-sites BED is missing: {}",
            target_sites_bed.display()
        ));
    }
    let reference_lengths = reference_contigs
        .iter()
        .map(|contig| (contig.name.as_str(), contig.length))
        .collect::<BTreeMap<_, _>>();
    let raw = fs::read_to_string(target_sites_bed)
        .with_context(|| format!("read {}", target_sites_bed.display()))?;
    let mut interval_count = 0usize;
    for (line_number, line) in raw.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 3 {
            return Err(anyhow!(
                "VCF corpus fixture BED line {} must contain at least 3 tab-delimited fields",
                line_number + 1
            ));
        }
        let contig = fields[0];
        let Some(reference_length) = reference_lengths.get(contig).copied() else {
            return Err(anyhow!(
                "VCF corpus fixture BED line {} references unknown contig `{contig}`",
                line_number + 1
            ));
        };
        let start = fields[1].parse::<usize>().with_context(|| {
            format!(
                "VCF corpus fixture BED line {} has invalid start `{}`",
                line_number + 1,
                fields[1]
            )
        })?;
        let end = fields[2].parse::<usize>().with_context(|| {
            format!(
                "VCF corpus fixture BED line {} has invalid end `{}`",
                line_number + 1,
                fields[2]
            )
        })?;
        if start >= end {
            return Err(anyhow!(
                "VCF corpus fixture BED line {} must satisfy start < end",
                line_number + 1
            ));
        }
        if end > reference_length {
            return Err(anyhow!(
                "VCF corpus fixture BED line {} ends beyond reference contig `{contig}` length {}",
                line_number + 1,
                reference_length
            ));
        }
        interval_count += 1;
    }
    if interval_count == 0 {
        return Err(anyhow!("VCF corpus fixture BED must declare at least one target interval"));
    }
    Ok(interval_count)
}
