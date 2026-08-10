BEGIN {
  OFS = "\t"
  FS = "[\t ]+"
}

mode == "summary" && FILENAME ~ /reference-span[.]tsv$/ {
  if ($1 == "reference_bases") {
    reference_bases = $2
  }
  next
}

mode == "summary" && source == "bedtools" {
  span = $3 - $2
  depth_sum += span * $4
  if ($4 > 0) {
    covered_bases += span
  }
  next
}

mode == "summary" && source == "samtools" {
  covered_bases++
  depth_sum += $3
  next
}

mode == "summary" && source == "mosdepth" && FILENAME ~ /summary[.]txt$/ && $1 == "total" {
  depth_sum = $3
  next
}

mode == "summary" && source == "mosdepth" && FILENAME ~ /quantized[.]bed$/ && $4 == "1:inf" {
  covered_bases += $3 - $2
  next
}

mode != "summary" && NR > 1 {
  value[$1] = $2
}

END {
  if (mode == "summary") {
    if (reference_bases > 0) {
      mean_depth = depth_sum / reference_bases
    }
    if (reference_bases <= 0 || covered_bases <= 0 || covered_bases > reference_bases || mean_depth <= 0) {
      exit 2
    }
    print "metric", "value"
    print "reference_bases", sprintf("%.0f", reference_bases)
    print "covered_bases", sprintf("%.0f", covered_bases)
    print "coverage_fraction", sprintf("%.12f", covered_bases / reference_bases)
    print "mean_depth", sprintf("%.12f", mean_depth)
  } else if (mode == "bundle") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.coverage.bundle.v1\",\"stage_id\":\"bam.coverage\",\"tool_id\":\"%s\",\"sample_id\":\"%s\",\"reference_bases\":%s,\"covered_bases\":%s,\"coverage_fraction\":%s,\"mean_depth\":%s}\n", tool, sample_id, value["reference_bases"], value["covered_bases"], value["coverage_fraction"], value["mean_depth"]
  } else if (mode == "probe") {
    complete = (value["reference_bases"] > 0 && value["covered_bases"] > 0 && value["mean_depth"] > 0) ? "true" : "false"
    printf "{\"schema_version\":\"bijux.benchmark.downstream_probe.v1\",\"stage_id\":\"benchmark.bam.coverage.compare\",\"input_contract\":\"bijux.benchmark.bam.coverage.bundle.v1\",\"complete\":%s}\n", complete
  } else if (mode == "metrics") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.coverage.metrics.v1\",\"reference_bases\":%s,\"covered_bases\":%s,\"coverage_fraction\":%s,\"mean_depth\":%s,\"bam_input_bytes\":%s,\"bam_index_input_bytes\":%s,\"input_bytes\":%s,\"output_bytes\":%s}\n", value["reference_bases"], value["covered_bases"], value["coverage_fraction"], value["mean_depth"], bam_input_bytes, bam_index_input_bytes, input_bytes, output_bytes
  } else {
    exit 2
  }
}
