use bijux_dna_core::prelude::{
    CommandSpecV1, ContainerImageRefV1, ToolConstraints, ToolExecutionSpecV1, ToolId,
};
use bijux_dna_planner_fastq::tool_adapters::fastq::correct_errors::{
    plan_correct_with_options, CorrectPlanOptions,
};

fn tool(tool_id: &str) -> ToolExecutionSpecV1 {
    ToolExecutionSpecV1 {
        tool_id: ToolId::new(tool_id.to_string()),
        tool_version: "fixture".to_string(),
        image: ContainerImageRefV1 { image: "bijux/test:latest".to_string(), digest: None },
        command: CommandSpecV1 { template: vec![tool_id.to_string(), "{{reads_r1}}".to_string()] },
        resources: ToolConstraints {
            runtime: "docker".to_string(),
            mem_gb: 1,
            tmp_gb: 1,
            threads: 2,
        },
    }
}

#[test]
#[allow(non_snake_case)]
fn slow__bayeshammer_reconstruction_preserves_paired_record_count() {
    let input_r1 = concat!(
        "@read1/1\n",
        "AAAAAA\n",
        "+\n",
        "IIIIII\n",
        "@read2/1\n",
        "TTTTTT\n",
        "+\n",
        "IIIIII\n",
    );
    let input_r2 = concat!(
        "@read1/2\n",
        "CCCCCC\n",
        "+\n",
        "IIIIII\n",
        "@read2/2\n",
        "GGGGGG\n",
        "+\n",
        "IIIIII\n",
    );
    let corrected_r1 = concat!("@read1/1\n", "AACCAA\n", "+\n", "IIIIII\n",);
    let corrected_r2 = concat!("@read1/2\n", "CCGGCC\n", "+\n", "IIIIII\n",);
    let unpaired = concat!("@read2/1\n", "TTTTAA\n", "+\n", "IIIIII\n",);

    let plan = plan_correct_with_options(
        &tool("bayeshammer"),
        std::path::Path::new("reads_R1.fastq"),
        Some(std::path::Path::new("reads_R2.fastq")),
        std::path::Path::new("out"),
        &CorrectPlanOptions::baseline(),
    )
    .expect("bayeshammer plan should build");

    let script = &plan.command.template[2];
    assert!(script.contains("R_unpaired"));
    assert!(script.contains("INPUT_R1='reads_R1.fastq'"));
    assert!(script.contains("INPUT_R2='reads_R2.fastq'"));

    let (reconstructed_r1, reconstructed_r2) = reconstruct_bayeshammer_pairs(
        parse_fastq_text(input_r1),
        parse_fastq_text(input_r2),
        parse_fastq_text(corrected_r1),
        parse_fastq_text(corrected_r2),
        parse_fastq_text(unpaired),
    );

    let decoded_r1 = render_fastq_text(&reconstructed_r1);
    let decoded_r2 = render_fastq_text(&reconstructed_r2);
    assert_eq!(reconstructed_r1.len(), 2);
    assert_eq!(reconstructed_r2.len(), 2);
    assert!(decoded_r1.contains("@read1/1\nAACCAA\n+\nIIIIII\n"));
    assert!(decoded_r1.contains("@read2/1\nTTTTAA\n+\nIIIIII\n"));
    assert!(decoded_r2.contains("@read1/2\nCCGGCC\n+\nIIIIII\n"));
    assert!(decoded_r2.contains("@read2/2\nGGGGGG\n+\nIIIIII\n"));
}

type FastqRecord = (String, String, String, String);

fn parse_fastq_text(text: &str) -> Vec<FastqRecord> {
    let mut records = Vec::new();
    let mut lines = text.lines();
    loop {
        let Some(header) = lines.next() else {
            break;
        };
        let Some(sequence) = lines.next() else {
            panic!("FASTQ sequence line");
        };
        let Some(plus) = lines.next() else {
            panic!("FASTQ plus line");
        };
        let Some(quality) = lines.next() else {
            panic!("FASTQ quality line");
        };
        records.push((
            header.to_string(),
            sequence.to_string(),
            plus.to_string(),
            quality.to_string(),
        ));
    }
    records
}

fn render_fastq_text(records: &[FastqRecord]) -> String {
    let mut rendered = String::new();
    for (header, sequence, plus, quality) in records {
        rendered.push_str(header);
        rendered.push('\n');
        rendered.push_str(sequence);
        rendered.push('\n');
        rendered.push_str(plus);
        rendered.push('\n');
        rendered.push_str(quality);
        rendered.push('\n');
    }
    rendered
}

fn read_key(record: &FastqRecord) -> String {
    let mut token = record
        .0
        .split_whitespace()
        .next()
        .unwrap_or_else(|| panic!("FASTQ header token"))
        .trim_start_matches('@')
        .to_string();
    if token.ends_with("/1") || token.ends_with("/2") {
        token.truncate(token.len() - 2);
    }
    token
}

fn sequence_distance(lhs: &str, rhs: &str) -> usize {
    let overlap = lhs.len().min(rhs.len());
    let mismatches =
        lhs.chars().zip(rhs.chars()).take(overlap).filter(|(left, right)| left != right).count();
    mismatches + lhs.len().max(rhs.len()) - overlap
}

fn reconstruct_bayeshammer_pairs(
    original_r1: Vec<FastqRecord>,
    original_r2: Vec<FastqRecord>,
    paired_r1: Vec<FastqRecord>,
    paired_r2: Vec<FastqRecord>,
    unpaired: Vec<FastqRecord>,
) -> (Vec<FastqRecord>, Vec<FastqRecord>) {
    let paired_r1_by_key = paired_r1
        .into_iter()
        .map(|record| (read_key(&record), record))
        .collect::<std::collections::BTreeMap<_, _>>();
    let paired_r2_by_key = paired_r2
        .into_iter()
        .map(|record| (read_key(&record), record))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut unpaired_by_key = std::collections::BTreeMap::<String, Vec<FastqRecord>>::new();
    for record in unpaired {
        unpaired_by_key.entry(read_key(&record)).or_default().push(record);
    }

    let mut reconstructed_r1 = Vec::new();
    let mut reconstructed_r2 = Vec::new();
    for (original_r1_record, original_r2_record) in original_r1.into_iter().zip(original_r2) {
        let key = read_key(&original_r1_record);
        let mut corrected_r1 = paired_r1_by_key.get(&key).cloned();
        let mut corrected_r2 = paired_r2_by_key.get(&key).cloned();
        let unpaired_records = unpaired_by_key.get(&key).cloned().unwrap_or_default();

        for unpaired_record in unpaired_records {
            let score_r1 = sequence_distance(&unpaired_record.1, &original_r1_record.1);
            let score_r2 = sequence_distance(&unpaired_record.1, &original_r2_record.1);
            if corrected_r1.is_none() && (corrected_r2.is_some() || score_r1 <= score_r2) {
                corrected_r1 = Some(unpaired_record);
                continue;
            }
            if corrected_r2.is_none() {
                corrected_r2 = Some(unpaired_record);
                continue;
            }
            if score_r1 <= score_r2 {
                corrected_r1 = Some(unpaired_record);
            } else {
                corrected_r2 = Some(unpaired_record);
            }
        }

        reconstructed_r1.push(corrected_r1.unwrap_or(original_r1_record));
        reconstructed_r2.push(corrected_r2.unwrap_or(original_r2_record));
    }

    (reconstructed_r1, reconstructed_r2)
}
