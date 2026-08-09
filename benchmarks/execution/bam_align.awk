function flag_has(flag, mask) {
  return int(flag / mask) % 2
}

function read_end(flag) {
  if (flag_has(flag, 64)) return "1"
  if (flag_has(flag, 128)) return "2"
  return "0"
}

BEGIN {
  OFS = "\t"
  FS = "\t"

  if (mode == "summary") {
    if (truth_path == "") exit 2
    if ((getline truth_header < truth_path) <= 0) exit 2
    split(truth_header, truth_columns, "\t")
    if (truth_columns[1] != "read_id" || truth_columns[2] != "read_end" || truth_columns[3] != "reference_name" || truth_columns[4] != "position") exit 2
    while ((getline truth_line < truth_path) > 0) {
      split(truth_line, truth_row, "\t")
      truth_key = truth_row[1] SUBSEP truth_row[2]
      if (truth_row[1] == "" || truth_row[2] !~ /^[012]$/ || truth_seen[truth_key]) exit 2
      truth_seen[truth_key] = 1
      truth_reference[truth_key] = truth_row[3]
      truth_position[truth_key] = truth_row[4] + 0
      truth_count++
      if (truth_row[3] != "*" && truth_row[4] + 0 > 0) expected_mapped_reads++
    }
    close(truth_path)
  }
}

mode == "summary" && $0 !~ /^@/ {
  flag = $2 + 0
  if (flag_has(flag, 256) || flag_has(flag, 2048)) next
  key = $1 SUBSEP read_end(flag)
  if (!truth_seen[key] || observed[key]) exit 2
  observed[key] = 1
  total_reads++
  if (!flag_has(flag, 4)) {
    mapped_reads++
    if (truth_reference[key] == $3 && truth_position[key] == $4 + 0) exact_placement_reads++
  }
  next
}

mode != "summary" && NR > 1 {
  value[$1] = $2
}

END {
  if (mode == "summary") {
    if (total_reads <= 0 || total_reads != truth_count || mapped_reads <= 0 || expected_mapped_reads <= 0 || exact_placement_reads > mapped_reads || exact_placement_reads > expected_mapped_reads) exit 2
    print "metric", "value"
    print "total_reads", sprintf("%.0f", total_reads)
    print "mapped_reads", sprintf("%.0f", mapped_reads)
    print "alignment_rate", sprintf("%.12f", mapped_reads / total_reads)
    print "expected_mapped_reads", sprintf("%.0f", expected_mapped_reads)
    print "exact_placement_reads", sprintf("%.0f", exact_placement_reads)
    print "placement_precision", sprintf("%.12f", exact_placement_reads / mapped_reads)
    print "placement_recall", sprintf("%.12f", exact_placement_reads / expected_mapped_reads)
  } else if (mode == "bundle") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.align.bundle.v1\",\"stage_id\":\"bam.align\",\"tool_id\":\"%s\",\"sample_id\":\"%s\",\"total_reads\":%s,\"mapped_reads\":%s,\"alignment_rate\":%s,\"expected_mapped_reads\":%s,\"exact_placement_reads\":%s,\"placement_precision\":%s,\"placement_recall\":%s}\n", tool, sample_id, value["total_reads"], value["mapped_reads"], value["alignment_rate"], value["expected_mapped_reads"], value["exact_placement_reads"], value["placement_precision"], value["placement_recall"]
  } else if (mode == "probe") {
    complete = (value["total_reads"] > 0 && value["mapped_reads"] > 0 && value["expected_mapped_reads"] > 0 && value["exact_placement_reads"] >= 0) ? "true" : "false"
    printf "{\"schema_version\":\"bijux.benchmark.downstream_probe.v1\",\"stage_id\":\"benchmark.bam.align.compare\",\"input_contract\":\"bijux.benchmark.bam.align.bundle.v1\",\"complete\":%s}\n", complete
  } else if (mode == "metrics") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.align.metrics.v1\",\"total_reads\":%s,\"mapped_reads\":%s,\"alignment_rate\":%s,\"expected_mapped_reads\":%s,\"exact_placement_reads\":%s,\"placement_precision\":%s,\"placement_recall\":%s,\"fastq_input_bytes\":%s,\"reference_input_bytes\":%s,\"reference_index_input_bytes\":%s,\"truth_input_bytes\":%s,\"input_bytes\":%s,\"output_bytes\":%s}\n", value["total_reads"], value["mapped_reads"], value["alignment_rate"], value["expected_mapped_reads"], value["exact_placement_reads"], value["placement_precision"], value["placement_recall"], fastq_input_bytes, reference_input_bytes, reference_index_input_bytes, truth_input_bytes, input_bytes, output_bytes
  } else {
    exit 2
  }
}
