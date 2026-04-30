use std::collections::{BTreeMap, VecDeque};
use std::path::Path;

use anyhow::Result;

use crate::artifacts::{RepairPairsReportV1, REPAIR_PAIRS_REPORT_SCHEMA_VERSION};

use super::fastq_io::{parse_header_pairing, read_fastq_records, write_fastq_records, FastqRecord};

/// Repair paired FASTQ streams into retained, rescued, and singleton outputs.
///
/// # Errors
/// Returns an error when inputs cannot be parsed or outputs cannot be written.
pub fn repair_pairs(
    r1: &Path,
    r2: &Path,
    retained_r1: &Path,
    retained_r2: &Path,
    rescued_r1: &Path,
    rescued_r2: &Path,
    singleton_r1: &Path,
    singleton_r2: &Path,
    rejected_path: &Path,
) -> Result<RepairPairsReportV1> {
    let left = read_fastq_records(r1)?;
    let right = read_fastq_records(r2)?;

    let mut right_index: BTreeMap<String, VecDeque<usize>> = BTreeMap::new();
    for (idx, record) in right.iter().enumerate() {
        let (base, _) = parse_header_pairing(record.header.trim_start_matches('@'));
        right_index.entry(base).or_default().push_back(idx);
    }

    let mut used_right = vec![false; right.len()];

    let mut retained_left = Vec::new();
    let mut retained_right = Vec::new();
    let mut rescued_left = Vec::new();
    let mut rescued_right = Vec::new();
    let mut singleton_left = Vec::new();
    let mut singleton_right = Vec::new();
    let rejected = Vec::<FastqRecord>::new();

    for (left_idx, left_record) in left.iter().enumerate() {
        let (base, _) = parse_header_pairing(left_record.header.trim_start_matches('@'));
        let Some(queue) = right_index.get_mut(&base) else {
            singleton_left.push(left_record.clone());
            continue;
        };
        let Some(right_idx) = queue.pop_front() else {
            singleton_left.push(left_record.clone());
            continue;
        };
        used_right[right_idx] = true;
        let right_record = right[right_idx].clone();
        if left_idx == right_idx {
            retained_left.push(left_record.clone());
            retained_right.push(right_record);
        } else {
            rescued_left.push(left_record.clone());
            rescued_right.push(right_record);
        }
    }

    for (idx, right_record) in right.iter().enumerate() {
        if !used_right[idx] {
            singleton_right.push(right_record.clone());
        }
    }

    write_fastq_records(retained_r1, &retained_left)?;
    write_fastq_records(retained_r2, &retained_right)?;
    write_fastq_records(rescued_r1, &rescued_left)?;
    write_fastq_records(rescued_r2, &rescued_right)?;
    write_fastq_records(singleton_r1, &singleton_left)?;
    write_fastq_records(singleton_r2, &singleton_right)?;
    write_fastq_records(rejected_path, &rejected)?;

    Ok(RepairPairsReportV1 {
        schema_version: REPAIR_PAIRS_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.repair_pairs".to_string(),
        stage_id: "fastq.repair_pairs".to_string(),
        tool_id: "bijux".to_string(),
        reads_in_r1: left.len() as u64,
        reads_in_r2: right.len() as u64,
        retained_pairs: retained_left.len() as u64,
        rescued_pairs: rescued_left.len() as u64,
        singleton_r1: singleton_left.len() as u64,
        singleton_r2: singleton_right.len() as u64,
        rejected_records: 0,
        retained_r1: retained_r1.display().to_string(),
        retained_r2: retained_r2.display().to_string(),
        rescued_r1: rescued_r1.display().to_string(),
        rescued_r2: rescued_r2.display().to_string(),
        singleton_r1_path: singleton_r1.display().to_string(),
        singleton_r2_path: singleton_r2.display().to_string(),
        rejected_path: rejected_path.display().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::repair_pairs;

    fn write_fastq(path: &std::path::Path, records: &[(&str, &str, &str)]) -> anyhow::Result<()> {
        let mut payload = String::new();
        for (header, seq, qual) in records {
            payload.push_str(&format!("@{header}\n{seq}\n+\n{qual}\n"));
        }
        std::fs::write(path, payload)?;
        Ok(())
    }

    #[test]
    fn repair_pairs_splits_retained_rescued_and_singletons() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-repair-pairs")?;
        let r1 = temp.path().join("r1.fastq");
        let r2 = temp.path().join("r2.fastq");
        write_fastq(&r1, &[("A/1", "AAAA", "!!!!"), ("B/1", "CCCC", "####"), ("C/1", "GGGG", "$$$$")])?;
        write_fastq(&r2, &[("A/2", "TTTT", "!!!!"), ("C/2", "GGGG", "$$$$"), ("B/2", "CCCC", "####"), ("D/2", "AAAA", "++++")])?;

        let report = repair_pairs(
            &r1,
            &r2,
            &temp.path().join("retained_r1.fastq"),
            &temp.path().join("retained_r2.fastq"),
            &temp.path().join("rescued_r1.fastq"),
            &temp.path().join("rescued_r2.fastq"),
            &temp.path().join("singletons_r1.fastq"),
            &temp.path().join("singletons_r2.fastq"),
            &temp.path().join("rejected.fastq"),
        )?;

        assert_eq!(report.retained_pairs, 1);
        assert_eq!(report.rescued_pairs, 2);
        assert_eq!(report.singleton_r1, 0);
        assert_eq!(report.singleton_r2, 1);
        Ok(())
    }
}
