//! Owner: bijux-dna-bench-workflows
//! Core germline fixtures reads.
use super::super::{
    anyhow, bail, fs, read_fastq_records, write_fastq_records, BTreeMap, Context, FastqRecord,
    Path, PipelineInputFixtures, ReadGroupSpec, Result, SyntheticReadPlacement,
};
use std::fmt::Write as _;

pub(crate) fn materialize_pipeline_input_fastqs(
    output_root: &Path,
    reference_fasta: &Path,
) -> Result<PipelineInputFixtures> {
    let stage_root = output_root.join("artifacts/fastq.source");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;
    let raw_r1 = stage_root.join("raw_R1.fastq");
    let raw_r2 = stage_root.join("raw_R2.fastq");
    let (reference_name, reference_sequence) = read_first_reference_contig(reference_fasta)?;
    let window_start = 900usize;
    let read_length = 30usize;
    if reference_sequence.len() < window_start + read_length {
        bail!("reference {} is too short to synthesize pipeline FASTQs", reference_fasta.display());
    }

    let mut variant_sequence =
        reference_sequence[window_start..window_start + read_length].to_string();
    let variant_offset = read_length / 2;
    let reference_base = variant_sequence.as_bytes()[variant_offset];
    let alt_base = alternate_base(reference_base);
    variant_sequence
        .replace_range(variant_offset..=variant_offset, &char::from(alt_base).to_string());
    let quality = "F".repeat(read_length);

    let records_r1 = vec![
        FastqRecord {
            header: "@read1/1".to_string(),
            sequence: variant_sequence.clone(),
            plus: "+".to_string(),
            quality: quality.clone(),
        },
        FastqRecord {
            header: "@read2/1".to_string(),
            sequence: variant_sequence.clone(),
            plus: "+".to_string(),
            quality: quality.clone(),
        },
    ];
    let records_r2 = vec![
        FastqRecord {
            header: "@read1/2".to_string(),
            sequence: variant_sequence.clone(),
            plus: "+".to_string(),
            quality: quality.clone(),
        },
        FastqRecord {
            header: "@read2/2".to_string(),
            sequence: variant_sequence,
            plus: "+".to_string(),
            quality,
        },
    ];
    write_fastq_records(&raw_r1, &records_r1)?;
    write_fastq_records(&raw_r2, &records_r2)?;

    let position = u64::try_from(window_start + 1).context("pipeline FASTQ position overflow")?;
    let placements = BTreeMap::from([
        (
            "read1/1".to_string(),
            SyntheticReadPlacement { reference_name: reference_name.clone(), position },
        ),
        (
            "read1/2".to_string(),
            SyntheticReadPlacement { reference_name: reference_name.clone(), position },
        ),
        (
            "read2/1".to_string(),
            SyntheticReadPlacement { reference_name: reference_name.clone(), position },
        ),
        ("read2/2".to_string(), SyntheticReadPlacement { reference_name, position }),
    ]);

    Ok(PipelineInputFixtures { raw_r1, raw_r2, placements })
}

pub(crate) fn read_first_reference_contig(reference_fasta: &Path) -> Result<(String, String)> {
    let payload = fs::read_to_string(reference_fasta)
        .with_context(|| format!("read {}", reference_fasta.display()))?;
    let mut current_name = None::<String>;
    let mut current_sequence = String::new();
    for line in payload.lines() {
        if let Some(header) = line.strip_prefix('>') {
            if let Some(name) = current_name.take() {
                return Ok((name, current_sequence));
            }
            current_name = Some(header.split_whitespace().next().unwrap_or_default().to_string());
            current_sequence.clear();
        } else {
            current_sequence.push_str(line.trim());
        }
    }
    current_name
        .map(|name| (name, current_sequence))
        .ok_or_else(|| anyhow!("reference {} has no FASTA contigs", reference_fasta.display()))
}

pub(crate) fn alternate_base(reference_base: u8) -> u8 {
    match reference_base.to_ascii_uppercase() {
        b'A' => b'C',
        b'C' => b'G',
        b'G' => b'T',
        _ => b'A',
    }
}

pub(crate) fn write_mapped_sam_from_fastqs(
    reference_fasta: &Path,
    input_r1: &Path,
    input_r2: &Path,
    read_group: &ReadGroupSpec,
    placements: &BTreeMap<String, SyntheticReadPlacement>,
    output_sam: &Path,
) -> Result<()> {
    let (reference_name, reference_sequence) = read_first_reference_contig(reference_fasta)?;
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
