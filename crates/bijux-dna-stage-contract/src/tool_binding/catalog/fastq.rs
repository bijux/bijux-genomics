//! Product-supported FASTQ benchmark portfolio bindings.

use bijux_dna_core::contract::ToolConstraints;

use super::{
    input, output, resources, retained_binding, BindingInputV1, BindingOutputV1, OutputDiscoveryV1,
    ToolBindingV1, DIRECTORY, FASTA, FASTQ_SINGLE, JSON, TABLE,
};

pub(super) fn canonical_fastq_portfolio_bindings() -> Vec<ToolBindingV1> {
    vec![
        bowtie2_depletion("fastq.deplete_host", "host"),
        bowtie2_depletion("fastq.deplete_reference_contaminants", "contaminant"),
        deplete_rrna(),
        internal_observation(
            "fastq.detect_duplicates_premerge",
            "duplicate_signal_report",
            "reader=cat; case '${input:reads_r1}' in *.gz) reader='gzip -cd';; esac; $reader '${input:reads_r1}' | awk 'NR%4==2 { reads++; seen[$0]++ } END { duplicate=0; for (read in seen) if (seen[read] > 1) duplicate += seen[read]-1; fraction=reads > 0 ? duplicate/reads : 0; printf \"{\\\"schema_version\\\":\\\"bijux.fastq.detect_duplicates_premerge.report.v1\\\",\\\"stage\\\":\\\"fastq.detect_duplicates_premerge\\\",\\\"stage_id\\\":\\\"fastq.detect_duplicates_premerge\\\",\\\"tool_id\\\":\\\"bijux_dna\\\",\\\"paired_mode\\\":\\\"single_end\\\",\\\"duplicate_detection_policy\\\":\\\"report_only\\\",\\\"measurement_scope\\\":\\\"premerge_sequence_signature\\\",\\\"modifies_reads\\\":false,\\\"advisory_only\\\":true,\\\"reads_in\\\":%d,\\\"duplicate_signal_reads\\\":%d,\\\"duplicate_signal_fraction\\\":%.12g,\\\"compared_read_pairs\\\":null}\\n\", reads, duplicate, fraction }' > '${output:duplicate_signal_report}'",
        ),
        internal_observation(
            "fastq.estimate_library_complexity_prealign",
            "library_complexity_report",
            "reader=cat; case '${input:reads_r1}' in *.gz) reader='gzip -cd';; esac; $reader '${input:reads_r1}' | awk 'NR%4==2 { reads++; sequence=substr($0,1,31); seen[sequence]=1 } END { unique=0; for (sequence in seen) unique++; unique_fraction=reads > 0 ? unique/reads : 0; duplicate_fraction=1-unique_fraction; reason=reads > 0 ? \"null\" : \"\\\"insufficient_reads_for_prealign_complexity_estimation\\\"\"; printf \"{\\\"schema_version\\\":\\\"bijux.fastq.estimate_library_complexity_prealign.report.v1\\\",\\\"stage\\\":\\\"fastq.estimate_library_complexity_prealign\\\",\\\"stage_id\\\":\\\"fastq.estimate_library_complexity_prealign\\\",\\\"tool_id\\\":\\\"bijux_dna\\\",\\\"paired_mode\\\":\\\"single_end\\\",\\\"complexity_policy\\\":\\\"prealign_kmer\\\",\\\"estimate_method\\\":\\\"kmer_redundancy\\\",\\\"modifies_reads\\\":false,\\\"advisory_only\\\":true,\\\"reads_in\\\":%d,\\\"estimated_unique_fraction\\\":%.12g,\\\"estimated_duplicate_fraction\\\":%.12g,\\\"insufficient_data_reason\\\":%s,\\\"kmer_size\\\":31}\\n\", reads, unique_fraction, duplicate_fraction, reason }' > '${output:library_complexity_report}'",
        ),
        filter_reads("fastp"),
        filter_reads("prinseq"),
        filter_reads("bbduk"),
        normalize_abundance(),
        normalize_primers(),
        profile_overrepresented("fastqc"),
        profile_overrepresented("fastq_scan"),
        profile_overrepresented("seqkit"),
        remove_chimeras(),
        remove_duplicates(),
        report_qc(),
        trim_polyg("fastp"),
        trim_polyg("bbduk"),
        trim_reads("fastp"),
        trim_reads("cutadapt"),
        trim_reads("bbduk"),
        trim_reads("adapterremoval"),
        trim_reads("leehom"),
        trim_reads("prinseq"),
        trim_reads("seqkit"),
        seqprep_trim_reads(),
        trim_terminal_damage("adapterremoval"),
        trim_terminal_damage("cutadapt"),
        trim_terminal_damage("seqkit"),
        validate_reads("fastqc"),
        validate_reads("fastq_scan"),
    ]
}

fn bowtie2_depletion(stage_id: &str, label: &str) -> ToolBindingV1 {
    let retained = format!("{label}_depleted_reads_r1");
    let removed = format!("removed_{label}_reads_r1");
    let metrics = format!("{label}_mapping_metrics");
    let inputs = [input("reads_r1", FASTQ_SINGLE), input("reference_index", DIRECTORY)];
    let outputs =
        [output(&retained, FASTQ_SINGLE), output(&removed, FASTQ_SINGLE), output(&metrics, TABLE)];
    let script = format!(
        "'${{executable}}' -x '${{input:reference_index}}' -U '${{input:reads_r1}}' --un-gz '${{output:{retained}}}' --al-gz '${{output:{removed}}}' --met-file '${{output:{metrics}}}' -S /dev/null --threads 8"
    );
    fastq_binding_owned(
        stage_id,
        "bowtie2",
        &inputs,
        &outputs,
        &["/bin/sh".to_string(), "-c".to_string(), script],
        resources(8, 16, 8),
    )
}

fn deplete_rrna() -> ToolBindingV1 {
    fastq_binding(
        "fastq.deplete_rrna",
        "sortmerna",
        &[input("reads_r1", FASTQ_SINGLE), input("rrna_database", DIRECTORY)],
        &[
            prefix_output("rrna_filtered_reads", FASTQ_SINGLE),
            prefix_output("removed_rrna_reads", FASTQ_SINGLE),
            prefix_output("rrna_report", TABLE),
        ],
        &[
            "${executable}",
            "--ref",
            "${input:rrna_database}",
            "--reads",
            "${input:reads_r1}",
            "--other",
            "${output:rrna_filtered_reads}",
            "--aligned",
            "${output:removed_rrna_reads}",
            "--log",
            "--workdir",
            "${output:rrna_report}",
            "--fastx",
            "--zip-out",
            "--threads",
            "8",
        ],
        resources(8, 16, 16),
    )
}

fn internal_observation(stage_id: &str, output_name: &str, observation: &str) -> ToolBindingV1 {
    let outputs = [output(output_name, JSON)];
    let command = format!("'${{executable}}' --version >/dev/null && {observation}");
    fastq_binding_owned(
        stage_id,
        "bijux_dna",
        &[input("reads_r1", FASTQ_SINGLE)],
        &outputs,
        &["/bin/sh".to_string(), "-c".to_string(), command],
        resources(1, 2, 2),
    )
}

fn filter_reads(tool_id: &str) -> ToolBindingV1 {
    let command = match tool_id {
        "fastp" => vec![
            "${executable}",
            "--in1",
            "${input:reads_r1}",
            "--out1",
            "${output:filtered_reads_r1}",
            "--json",
            "${output:filter_report}",
            "--thread",
            "8",
            "--length_required",
            "1",
        ],
        "prinseq" => vec![
            "/bin/sh",
            "-c",
            "'${executable}' -fastq '${input:reads_r1}' -out_good '${output:filtered_reads_r1}.prefix' -out_bad null -min_len 1 && mv '${output:filtered_reads_r1}.prefix.fastq' '${output:filtered_reads_r1}' && printf 'prinseq_completed\\n' > '${output:filter_report}'",
        ],
        "bbduk" => vec![
            "${executable}",
            "in=${input:reads_r1}",
            "out=${output:filtered_reads_r1}",
            "stats=${output:filter_report}",
            "threads=8",
            "minlen=1",
        ],
        _ => unreachable!("closed FASTQ filter tool set"),
    };
    fastq_binding(
        "fastq.filter_reads",
        tool_id,
        &[input("reads_r1", FASTQ_SINGLE)],
        &[output("filtered_reads_r1", FASTQ_SINGLE), output("filter_report", TABLE)],
        &command,
        resources(8, 8, 4),
    )
}

fn normalize_abundance() -> ToolBindingV1 {
    fastq_binding(
        "fastq.normalize_abundance",
        "seqkit",
        &[input("abundance_table", TABLE)],
        &[output("normalized_abundance", TABLE)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' version >/dev/null && awk 'BEGIN { FS=OFS=\"\\t\" } NR==1 { print $1,$2,\"normalized_abundance\"; next } { rows[NR]=$0; total[$1]+=$3 } END { for (i=2; i<=NR; i++) { split(rows[i], column, FS); value=total[column[1]] > 0 ? column[3]/total[column[1]] : 0; print column[1],column[2],value } }' '${input:abundance_table}' > '${output:normalized_abundance}'",
        ],
        resources(1, 4, 2),
    )
}

fn normalize_primers() -> ToolBindingV1 {
    fastq_binding(
        "fastq.normalize_primers",
        "cutadapt",
        &[input("reads_r1", FASTQ_SINGLE), input("primer_reference", FASTA)],
        &[output("normalized_reads_r1", FASTQ_SINGLE), output("primer_report", JSON)],
        &[
            "${executable}",
            "--cores",
            "8",
            "--adapter",
            "file:${input:primer_reference}",
            "--minimum-length",
            "1",
            "--json",
            "${output:primer_report}",
            "--output",
            "${output:normalized_reads_r1}",
            "${input:reads_r1}",
        ],
        resources(8, 8, 4),
    )
}

fn profile_overrepresented(tool_id: &str) -> ToolBindingV1 {
    let outputs = if tool_id == "fastqc" {
        vec![output("overrepresented_evidence", DIRECTORY)]
    } else {
        vec![output("overrepresented_evidence", TABLE)]
    };
    let command = match tool_id {
        "fastqc" => vec![
            "/bin/sh",
            "-c",
            "mkdir -p '${output:overrepresented_evidence}' && '${executable}' --outdir '${output:overrepresented_evidence}' --threads 4 '${input:reads_r1}'",
        ],
        "fastq_scan" => vec![
            "/bin/sh",
            "-c",
            "'${executable}' '${input:reads_r1}' > '${output:overrepresented_evidence}'",
        ],
        "seqkit" => vec![
            "/bin/sh",
            "-c",
            "'${executable}' fx2tab -j 4 -n -s '${input:reads_r1}' > '${output:overrepresented_evidence}'",
        ],
        _ => unreachable!("closed overrepresented sequence tool set"),
    };
    fastq_binding(
        "fastq.profile_overrepresented_sequences",
        tool_id,
        &[input("reads_r1", FASTQ_SINGLE)],
        &outputs,
        &command,
        resources(4, 8, 4),
    )
}

fn remove_chimeras() -> ToolBindingV1 {
    fastq_binding(
        "fastq.remove_chimeras",
        "vsearch",
        &[input("representative_sequences", FASTA)],
        &[
            output("nonchimeric_sequences", FASTA),
            output("chimeric_sequences", FASTA),
            output("chimera_report", TABLE),
        ],
        &[
            "${executable}",
            "--uchime_denovo",
            "${input:representative_sequences}",
            "--nonchimeras",
            "${output:nonchimeric_sequences}",
            "--chimeras",
            "${output:chimeric_sequences}",
            "--uchimeout",
            "${output:chimera_report}",
            "--threads",
            "8",
        ],
        resources(8, 8, 4),
    )
}

fn remove_duplicates() -> ToolBindingV1 {
    fastq_binding(
        "fastq.remove_duplicates",
        "clumpify",
        &[input("reads_r1", FASTQ_SINGLE)],
        &[output("deduplicated_reads_r1", FASTQ_SINGLE), output("deduplication_report", TABLE)],
        &[
            "/bin/sh",
            "-c",
            "'${executable}' in='${input:reads_r1}' out='${output:deduplicated_reads_r1}' dedupe=t 2> '${output:deduplication_report}'",
        ],
        resources(8, 16, 16),
    )
}

fn report_qc() -> ToolBindingV1 {
    fastq_binding(
        "fastq.report_qc",
        "multiqc",
        &[input("qc_inputs", DIRECTORY)],
        &[output("multiqc_data", DIRECTORY)],
        &[
            "/bin/sh",
            "-c",
            "mkdir -p '${output:multiqc_data}' && '${executable}' --outdir '${output:multiqc_data}' --filename multiqc_report.html '${input:qc_inputs}'",
        ],
        resources(2, 8, 4),
    )
}

fn trim_polyg(tool_id: &str) -> ToolBindingV1 {
    let command = match tool_id {
        "fastp" => vec![
            "${executable}",
            "--in1",
            "${input:reads_r1}",
            "--out1",
            "${output:trimmed_reads_r1}",
            "--json",
            "${output:trim_report}",
            "--trim_poly_g",
            "--poly_g_min_len",
            "10",
            "--thread",
            "8",
        ],
        "bbduk" => vec![
            "${executable}",
            "in=${input:reads_r1}",
            "out=${output:trimmed_reads_r1}",
            "stats=${output:trim_report}",
            "trimpolygright=10",
            "threads=8",
        ],
        _ => unreachable!("closed poly-G tool set"),
    };
    fastq_binding(
        "fastq.trim_polyg_tails",
        tool_id,
        &[input("reads_r1", FASTQ_SINGLE)],
        &[output("trimmed_reads_r1", FASTQ_SINGLE), output("trim_report", TABLE)],
        &command,
        resources(8, 8, 4),
    )
}

fn trim_reads(tool_id: &str) -> ToolBindingV1 {
    let (inputs, command) = match tool_id {
        "fastp" => (
            vec![input("reads_r1", FASTQ_SINGLE)],
            vec![
                "${executable}",
                "--in1",
                "${input:reads_r1}",
                "--out1",
                "${output:trimmed_reads_r1}",
                "--thread",
                "8",
            ],
        ),
        "cutadapt" => (
            vec![input("reads_r1", FASTQ_SINGLE), input("adapter_reference", FASTA)],
            vec![
                "${executable}",
                "--cores",
                "8",
                "--adapter",
                "file:${input:adapter_reference}",
                "--minimum-length",
                "1",
                "--output",
                "${output:trimmed_reads_r1}",
                "${input:reads_r1}",
            ],
        ),
        "bbduk" => (
            vec![input("reads_r1", FASTQ_SINGLE), input("adapter_reference", FASTA)],
            vec![
                "${executable}",
                "in=${input:reads_r1}",
                "out=${output:trimmed_reads_r1}",
                "ref=${input:adapter_reference}",
                "ktrim=r",
                "k=23",
                "mink=11",
                "threads=8",
            ],
        ),
        "adapterremoval" => (
            vec![input("reads_r1", FASTQ_SINGLE)],
            vec![
                "${executable}",
                "--file1",
                "${input:reads_r1}",
                "--output1",
                "${output:trimmed_reads_r1}",
                "--threads",
                "8",
            ],
        ),
        "leehom" => (
            vec![input("reads_r1", FASTQ_SINGLE)],
            vec![
                "/bin/sh",
                "-c",
                "work_dir=$(mktemp -d); trap 'rm -rf \"$work_dir\"' EXIT; cd \"$work_dir\" && '${executable}' -fq1 '${input:reads_r1}' -t 8 -f AGATCGGAAGAGC -fqo leehom && mv leehom.fq.gz '${output:trimmed_reads_r1}'",
            ],
        ),
        "prinseq" => (
            vec![input("reads_r1", FASTQ_SINGLE)],
            vec![
                "/bin/sh",
                "-c",
                "'${executable}' -fastq '${input:reads_r1}' -out_good '${output:trimmed_reads_r1}.prefix' -out_bad null -trim_tail_right 5 && mv '${output:trimmed_reads_r1}.prefix.fastq' '${output:trimmed_reads_r1}'",
            ],
        ),
        "seqkit" => (
            vec![input("reads_r1", FASTQ_SINGLE)],
            vec![
                "${executable}",
                "seq",
                "--threads",
                "8",
                "--min-len",
                "1",
                "--out-file",
                "${output:trimmed_reads_r1}",
                "${input:reads_r1}",
            ],
        ),
        _ => unreachable!("closed trim tool set"),
    };
    fastq_binding(
        "fastq.trim_reads",
        tool_id,
        &inputs,
        &[output("trimmed_reads_r1", FASTQ_SINGLE)],
        &command,
        resources(8, 8, 4),
    )
}

fn seqprep_trim_reads() -> ToolBindingV1 {
    fastq_binding(
        "fastq.trim_reads",
        "seqprep",
        &[input("reads_r1", FASTQ_SINGLE), input("reads_r2", FASTQ_SINGLE)],
        &[output("trimmed_reads_r1", FASTQ_SINGLE), output("trimmed_reads_r2", FASTQ_SINGLE)],
        &[
            "${executable}",
            "-f",
            "${input:reads_r1}",
            "-r",
            "${input:reads_r2}",
            "-1",
            "${output:trimmed_reads_r1}",
            "-2",
            "${output:trimmed_reads_r2}",
        ],
        resources(4, 8, 4),
    )
}

fn trim_terminal_damage(tool_id: &str) -> ToolBindingV1 {
    let command = match tool_id {
        "adapterremoval" => vec![
            "${executable}",
            "--file1",
            "${input:reads_r1}",
            "--output1",
            "${output:damage_trimmed_reads_r1}",
            "--trim5p",
            "2",
            "--trim3p",
            "2",
            "--threads",
            "8",
        ],
        "cutadapt" => vec![
            "${executable}",
            "--cores",
            "8",
            "--cut",
            "2",
            "--cut",
            "-2",
            "--output",
            "${output:damage_trimmed_reads_r1}",
            "${input:reads_r1}",
        ],
        "seqkit" => vec![
            "${executable}",
            "subseq",
            "--threads",
            "8",
            "--range",
            "3:-3",
            "--out-file",
            "${output:damage_trimmed_reads_r1}",
            "${input:reads_r1}",
        ],
        _ => unreachable!("closed terminal-damage tool set"),
    };
    fastq_binding(
        "fastq.trim_terminal_damage",
        tool_id,
        &[input("reads_r1", FASTQ_SINGLE)],
        &[output("damage_trimmed_reads_r1", FASTQ_SINGLE)],
        &command,
        resources(8, 8, 4),
    )
}

fn validate_reads(tool_id: &str) -> ToolBindingV1 {
    let outputs = if tool_id == "fastqc" {
        vec![output("validation_evidence", DIRECTORY)]
    } else {
        vec![output("validation_evidence", TABLE)]
    };
    let command = match tool_id {
        "fastqc" => vec![
            "/bin/sh",
            "-c",
            "mkdir -p '${output:validation_evidence}' && '${executable}' --quiet --threads 4 --outdir '${output:validation_evidence}' '${input:reads_r1}'",
        ],
        "fastq_scan" => vec![
            "/bin/sh",
            "-c",
            "'${executable}' '${input:reads_r1}' > '${output:validation_evidence}'",
        ],
        _ => unreachable!("closed validation tool set"),
    };
    fastq_binding(
        "fastq.validate_reads",
        tool_id,
        &[input("reads_r1", FASTQ_SINGLE)],
        &outputs,
        &command,
        resources(4, 8, 4),
    )
}

fn prefix_output(name: &str, artifact_contract_id: &str) -> BindingOutputV1 {
    BindingOutputV1 {
        name: name.to_string(),
        artifact_contract_id: artifact_contract_id.to_string(),
        discovery: OutputDiscoveryV1::Glob { pattern: format!("${{output:{name}}}*") },
    }
}

fn fastq_binding(
    stage_id: &str,
    tool_id: &str,
    inputs: &[BindingInputV1],
    outputs: &[BindingOutputV1],
    command_template: &[&str],
    resources: ToolConstraints,
) -> ToolBindingV1 {
    let normalized_output_contract_id = format!("bijux.benchmark.{stage_id}.bundle.v1");
    let normalizer_id = format!(
        "bijux_dna_planner_fastq::tool_adapters::fastq::{}",
        stage_id.trim_start_matches("fastq.")
    );
    let downstream_probe_id = format!("benchmark.{stage_id}.compare");
    retained_binding(
        stage_id,
        tool_id,
        &["fastq_stage_input", "tool_native_output", "normalized_stage_result"],
        inputs,
        &[],
        outputs,
        command_template,
        &normalized_output_contract_id,
        &normalizer_id,
        &downstream_probe_id,
        resources,
        &[
            "raw_tool_output_is_preserved_before_normalization",
            "paired_end_campaigns_require_explicit_pair_aware_binding_inputs",
        ],
    )
}

fn fastq_binding_owned(
    stage_id: &str,
    tool_id: &str,
    inputs: &[BindingInputV1],
    outputs: &[BindingOutputV1],
    command_template: &[String],
    resources: ToolConstraints,
) -> ToolBindingV1 {
    let command_refs = command_template.iter().map(String::as_str).collect::<Vec<_>>();
    fastq_binding(stage_id, tool_id, inputs, outputs, &command_refs, resources)
}
