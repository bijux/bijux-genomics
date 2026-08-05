BEGIN {
  OFS = "\t"
  FS = "\t"
}

mode == "summary" && source == "picard" && $1 == "LIBRARY" {
  for (i = 1; i <= NF; i++) column[$i] = i
  if (getline <= 0) exit 2
  examined = $(column["UNPAIRED_READS_EXAMINED"]) + 2 * $(column["READ_PAIRS_EXAMINED"])
  duplicates = $(column["UNPAIRED_READ_DUPLICATES"]) + 2 * $(column["READ_PAIR_DUPLICATES"])
  next
}

mode == "summary" && source == "samtools" {
  split($1, item, ":")
  key = item[1]
  parsed_value = item[2]
  gsub(/^[[:space:]]+|[[:space:]]+$/, "", key)
  gsub(/^[[:space:]]+|[[:space:]]+$/, "", parsed_value)
  if (key == "EXAMINED") examined = parsed_value + 0
  if (key == "DUPLICATE TOTAL") duplicates = parsed_value + 0
  next
}

mode != "summary" && NR > 1 {
  metric_value[$1] = $2
}

END {
  if (mode == "summary") {
    if (examined <= 0 || duplicates < 0 || duplicates > examined) exit 2
    print "metric", "value"
    print "examined_reads", sprintf("%.0f", examined)
    print "duplicate_reads", sprintf("%.0f", duplicates)
    print "duplication_fraction", sprintf("%.12f", duplicates / examined)
  } else if (mode == "bundle") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.duplication_metrics.bundle.v1\",\"stage_id\":\"bam.duplication_metrics\",\"tool_id\":\"%s\",\"sample_id\":\"%s\",\"examined_reads\":%s,\"duplicate_reads\":%s,\"duplication_fraction\":%s}\n", tool, sample_id, metric_value["examined_reads"], metric_value["duplicate_reads"], metric_value["duplication_fraction"]
  } else if (mode == "probe") {
    complete = (metric_value["examined_reads"] > 0 && metric_value["duplicate_reads"] >= 0) ? "true" : "false"
    printf "{\"schema_version\":\"bijux.benchmark.downstream_probe.v1\",\"stage_id\":\"benchmark.bam.duplication_metrics.compare\",\"input_contract\":\"bijux.benchmark.bam.duplication_metrics.bundle.v1\",\"complete\":%s}\n", complete
  } else if (mode == "metrics") {
    printf "{\"examined_reads\":%s,\"duplicate_reads\":%s,\"duplication_fraction\":%s,\"bam_input_bytes\":%s,\"bam_index_input_bytes\":%s,\"input_bytes\":%s,\"output_bytes\":%s}\n", metric_value["examined_reads"], metric_value["duplicate_reads"], metric_value["duplication_fraction"], bam_input_bytes, bam_index_input_bytes, input_bytes, output_bytes
  } else {
    exit 2
  }
}
