BEGIN {
  OFS = "\t"
  FS = "\t"
}

NR > 1 {
  value[$1] = $2
}

END {
  complete = value["normalization_complete"] + 0
  file_count = value["native_file_count"] + 0
  native_bytes = value["native_output_bytes"] + 0
  if (complete != 1 || file_count < 1 || native_bytes < 1) {
    exit 2
  }
  if (mode == "summary") {
    print "metric", "value"
    print "normalization_complete", complete
    print "native_file_count", file_count
    print "native_output_bytes", native_bytes
  } else if (mode == "bundle") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.damage.bundle.v1\",\"stage_id\":\"bam.damage\",\"tool_id\":\"%s\",\"sample_id\":\"%s\",\"normalization_complete\":%d,\"native_file_count\":%d,\"native_output_bytes\":%d,\"authenticity_certified\":false}\n", tool, sample_id, complete, file_count, native_bytes
  } else if (mode == "probe") {
    printf "{\"schema_version\":\"bijux.benchmark.downstream_probe.v1\",\"stage_id\":\"benchmark.bam.damage.consume\",\"input_contract\":\"bijux.benchmark.bam.damage.bundle.v1\",\"complete\":true,\"authenticity_certified\":false}\n"
  } else if (mode == "metrics") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.damage.metrics.v1\",\"normalization_complete\":%d,\"native_file_count\":%d,\"native_output_bytes\":%d,\"bam_input_bytes\":%s,\"bam_index_input_bytes\":%s,\"reference_input_bytes\":%s,\"input_bytes\":%s,\"output_bytes\":%s}\n", complete, file_count, native_bytes, bam_input_bytes, bam_index_input_bytes, reference_input_bytes, input_bytes, output_bytes
  } else {
    exit 2
  }
}
