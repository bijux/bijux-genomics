BEGIN {
  OFS = "\t"
  FS = "\t"
}

mode == "summary" && $1 == "ACCUMULATION_LEVEL" {
  for (field = 1; field <= NF; field++) column[$field] = field
  parsing_metrics = 1
  next
}

mode == "summary" && parsing_metrics && $(column["ACCUMULATION_LEVEL"]) == "ALL_READS" {
  total_clusters = $(column["TOTAL_CLUSTERS"]) + 0
  aligned_reads = $(column["ALIGNED_READS"]) + 0
  at_dropout = $(column["AT_DROPOUT"]) + 0
  gc_dropout = $(column["GC_DROPOUT"]) + 0
  windows = $(column["WINDOWS"]) + 0
  read_starts = $(column["READ_STARTS"]) + 0
  found = 1
  next
}

mode != "summary" && NR > 1 {
  value[$1] = $2
}

END {
  if (mode == "summary") {
    if (!found || total_clusters <= 0 || aligned_reads <= 0 || aligned_reads > total_clusters || at_dropout < 0 || at_dropout > 100 || gc_dropout < 0 || gc_dropout > 100 || windows <= 0 || read_starts <= 0 || read_starts > aligned_reads) exit 2
    print "metric", "value"
    print "gc_bias_score", sprintf("%.12f", gc_dropout / 100)
    print "at_dropout", sprintf("%.12f", at_dropout)
    print "gc_dropout", sprintf("%.12f", gc_dropout)
    print "total_clusters", sprintf("%.0f", total_clusters)
    print "aligned_reads", sprintf("%.0f", aligned_reads)
    print "windows", sprintf("%.0f", windows)
    print "read_starts", sprintf("%.0f", read_starts)
  } else if (mode == "bundle") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.gc_bias.bundle.v1\",\"stage_id\":\"bam.gc_bias\",\"tool_id\":\"picard\",\"sample_id\":\"%s\",\"gc_bias_score\":%s,\"at_dropout\":%s,\"gc_dropout\":%s,\"total_clusters\":%s,\"aligned_reads\":%s,\"windows\":%s,\"read_starts\":%s}\n", sample_id, value["gc_bias_score"], value["at_dropout"], value["gc_dropout"], value["total_clusters"], value["aligned_reads"], value["windows"], value["read_starts"]
  } else if (mode == "probe") {
    complete = (value["total_clusters"] > 0 && value["aligned_reads"] > 0 && value["aligned_reads"] <= value["total_clusters"] && value["read_starts"] <= value["aligned_reads"]) ? "true" : "false"
    printf "{\"schema_version\":\"bijux.benchmark.downstream_probe.v1\",\"stage_id\":\"benchmark.bam.gc_bias.consume\",\"input_contract\":\"bijux.benchmark.bam.gc_bias.bundle.v1\",\"complete\":%s}\n", complete
  } else if (mode == "metrics") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.gc_bias.metrics.v1\",\"gc_bias_score\":%s,\"at_dropout\":%s,\"gc_dropout\":%s,\"total_clusters\":%s,\"aligned_reads\":%s,\"windows\":%s,\"read_starts\":%s,\"bam_input_bytes\":%s,\"bam_index_input_bytes\":%s,\"reference_input_bytes\":%s,\"input_bytes\":%s,\"output_bytes\":%s}\n", value["gc_bias_score"], value["at_dropout"], value["gc_dropout"], value["total_clusters"], value["aligned_reads"], value["windows"], value["read_starts"], bam_input_bytes, bam_index_input_bytes, reference_input_bytes, input_bytes, output_bytes
  } else {
    exit 2
  }
}
