//! Owner: bijux-dna-bench-workflows
//! Ancient DNA fixtures reads.
use super::super::{
    anyhow, bail, fs, read_fastq_records, write_fastq_records, BTreeMap, BufRead, BufReader,
    Context, FastqRecord, Path, PipelineInputFixtures, ReadGroupSpec, Result,
    SyntheticReadPlacement, ADNA_REFERENCE_CONTIG,
};
use std::fmt::Write as _;

pub(crate) fn materialize_adna_pipeline_input_fastqs(
    output_root: &Path,
    reference_fasta: &Path,
) -> Result<PipelineInputFixtures> {
    let stage_root = output_root.join("artifacts/fastq.source");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let (reference_name, reference_sequence) =
        read_named_reference_contig(reference_fasta, ADNA_REFERENCE_CONTIG)?;

    let mut placements = BTreeMap::<String, SyntheticReadPlacement>::new();
    let mut r1 = Vec::<FastqRecord>::new();
    let mut r2 = Vec::<FastqRecord>::new();

    let pair_specs = [
        ("adna_pair_a_r1", 1_u64, true, true, true),
        ("adna_pair_dup_r1", 1_u64, true, true, true),
        ("adna_pair_b_r1", 5_u64, true, true, true),
    ];
    for (header, position, add_internal_variant, add_5p_damage, add_3p_damage) in pair_specs {
        let sequence = synthetic_adna_read_sequence(
            &reference_sequence,
            usize::try_from(position).unwrap_or(1),
            24,
            add_internal_variant,
            add_5p_damage,
            add_3p_damage,
        )?;
        r1.push(FastqRecord {
            header: format!("@{header}"),
            sequence,
            plus: "+".to_string(),
            quality: "I".repeat(24),
        });
        placements.insert(
            header.to_string(),
            SyntheticReadPlacement { reference_name: reference_name.clone(), position },
        );
    }

    let mate_specs = [
        ("adna_pair_a_r2", 33_u64, false, true, true),
        ("adna_pair_dup_r2", 33_u64, false, true, true),
        ("adna_pair_b_r2", 37_u64, false, true, true),
    ];
    for (header, position, add_internal_variant, add_5p_damage, add_3p_damage) in mate_specs {
        let sequence = synthetic_adna_read_sequence(
            &reference_sequence,
            usize::try_from(position).unwrap_or(1),
            24,
            add_internal_variant,
            add_5p_damage,
            add_3p_damage,
        )?;
        r2.push(FastqRecord {
            header: format!("@{header}"),
            sequence,
            plus: "+".to_string(),
            quality: "I".repeat(24),
        });
        placements.insert(
            header.to_string(),
            SyntheticReadPlacement { reference_name: reference_name.clone(), position },
        );
    }

    let raw_r1 = stage_root.join("raw_R1.fastq");
    let raw_r2 = stage_root.join("raw_R2.fastq");
    write_fastq_records(&raw_r1, &r1)?;
    write_fastq_records(&raw_r2, &r2)?;

    Ok(PipelineInputFixtures { raw_r1, raw_r2, placements, reference_name })
}

pub(crate) fn synthetic_adna_read_sequence(
    reference_sequence: &str,
    start_position: usize,
    length: usize,
    add_internal_variant: bool,
    add_5p_damage: bool,
    add_3p_damage: bool,
) -> Result<String> {
    if start_position == 0 {
        bail!("synthetic aDNA read positions are 1-based");
    }
    let start_index = start_position - 1;
    let end_index = start_index.saturating_add(length);
    let reference_window = reference_sequence
        .get(start_index..end_index)
        .ok_or_else(|| anyhow!("synthetic aDNA read window is out of range"))?;
    let mut bases = reference_window.as_bytes().to_vec();
    if add_5p_damage && bases.len() > 2 && bases[1] == b'C' {
        bases[1] = b'T';
    }
    if add_3p_damage && bases.len() > 2 && bases[bases.len() - 2] == b'G' {
        let last_index = bases.len() - 2;
        bases[last_index] = b'A';
    }
    if add_internal_variant && bases.len() > 12 {
        let target = 11_usize;
        bases[target] = match bases[target] {
            b'C' => b'A',
            b'G' => b'T',
            _ => b'G',
        };
    }
    String::from_utf8(bases)
        .map_err(|error| anyhow!("synthetic aDNA sequence is not UTF-8: {error}"))
}

pub(crate) fn read_named_reference_contig(
    reference_fasta: &Path,
    wanted: &str,
) -> Result<(String, String)> {
    let file = fs::File::open(reference_fasta)
        .with_context(|| format!("open {}", reference_fasta.display()))?;
    let reader = BufReader::new(file);
    let mut current_name = None::<String>;
    let mut current_sequence = String::new();
    for line in reader.lines() {
        let line = line.with_context(|| format!("read {}", reference_fasta.display()))?;
        if let Some(header) = line.strip_prefix('>') {
            if let Some(name) = current_name.take() {
                if name == wanted {
                    return Ok((name, current_sequence));
                }
                current_sequence = String::new();
            }
            current_name = Some(header.split_whitespace().next().unwrap_or_default().to_string());
            continue;
        }
        current_sequence.push_str(line.trim());
    }
    if let Some(name) = current_name {
        if name == wanted {
            return Ok((name, current_sequence));
        }
    }
    bail!("reference {} does not contain required contig `{wanted}`", reference_fasta.display())
}

pub(crate) fn write_mapped_sam_from_fastqs(
    reference_fasta: &Path,
    reference_name: &str,
    input_r1: &Path,
    input_r2: &Path,
    read_group: &ReadGroupSpec,
    placements: &BTreeMap<String, SyntheticReadPlacement>,
    output_sam: &Path,
) -> Result<()> {
    let (_, reference_sequence) = read_named_reference_contig(reference_fasta, reference_name)?;
    let mut payload = String::new();
    payload.push_str("@HD\tVN:1.6\tSO:coordinate\n");
    writeln!(payload, "@SQ\tSN:{reference_name}\tLN:{}", reference_sequence.len())?;
    write!(
        payload,
        "@RG\tID:{}\tSM:{}\tPL:{}\tLB:{}",
        read_group.id, read_group.sample, read_group.platform, read_group.library
    )?;
    if let Some(unit) = &read_group.platform_unit {
        write!(payload, "\tPU:{unit}")?;
    }
    payload.push('\n');

    let mut records = read_fastq_records(input_r1)?;
    records.extend(read_fastq_records(input_r2)?);
    for record in records {
        let qname = record.header.trim_start_matches('@');
        let placement = placements
            .get(qname)
            .ok_or_else(|| anyhow!("missing synthetic placement for `{qname}`"))?;
        writeln!(
            payload,
            "{qname}\t0\t{}\t{}\t60\t{}M\t*\t0\t0\t{}\t{}\tRG:Z:{}",
            placement.reference_name,
            placement.position,
            record.sequence.len(),
            record.sequence,
            record.quality,
            read_group.id
        )?;
    }
    bijux_dna_infra::write_bytes(output_sam, payload)?;
    Ok(())
}
