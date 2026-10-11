BEGIN {
  OFS = "\t"
  FS = "\t"
}

NR > 1 {
  value[$1] = $2
}

END {
  total = value["total_primary_reads"]
  mapped = value["mapped_primary_reads"]
  nonduplicate = value["mapped_nonduplicate_primary_reads"]
  fraction = mapped / total
  if (total <= 0 || mapped <= 0 || mapped > total || nonduplicate <= 0 || nonduplicate > mapped) {
    exit 2
  }
  if (mode == "summary") {
    print "metric", "value"
    print "total_primary_reads", sprintf("%.0f", total)
    print "mapped_primary_reads", sprintf("%.0f", mapped)
    print "mapped_fraction", sprintf("%.12f", fraction)
    print "mapped_nonduplicate_primary_reads", sprintf("%.0f", nonduplicate)
  } else if (mode == "bundle") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.endogenous_content.bundle.v1\",\"stage_id\":\"bam.endogenous_content\",\"tool_id\":\"%s\",\"sample_id\":\"%s\",\"total_primary_reads\":%s,\"mapped_primary_reads\":%s,\"mapped_fraction\":%.12f,\"mapped_nonduplicate_primary_reads\":%s}\n", tool, sample_id, total, mapped, fraction, nonduplicate
  } else if (mode == "probe") {
    printf "{\"schema_version\":\"bijux.benchmark.downstream_probe.v1\",\"stage_id\":\"benchmark.bam.endogenous_content.consume\",\"input_contract\":\"bijux.benchmark.bam.endogenous_content.bundle.v1\",\"complete\":true}\n"
  } else if (mode == "metrics") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.endogenous_content.metrics.v1\",\"total_primary_reads\":%s,\"mapped_primary_reads\":%s,\"mapped_fraction\":%.12f,\"mapped_nonduplicate_primary_reads\":%s,\"bam_input_bytes\":%s,\"bam_index_input_bytes\":%s,\"input_bytes\":%s,\"output_bytes\":%s}\n", total, mapped, fraction, nonduplicate, bam_input_bytes, bam_index_input_bytes, input_bytes, output_bytes
  } else {
    exit 2
  }
}
