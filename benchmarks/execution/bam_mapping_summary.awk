BEGIN {
  OFS = "\t"
  if (mode == "summary") {
    if (source == "picard") {
      FS = "\t"
    } else {
      FS = "[[:space:]]+"
    }
  } else {
    FS = "\t"
  }
}

mode == "summary" && source == "picard" && $1 == "CATEGORY" {
  for (field = 1; field <= NF; field++) {
    column[$field] = field
  }
  next
}

mode == "summary" && source == "picard" && ($1 == "PAIR" || $1 == "UNPAIRED") {
  primary_reads += $(column["TOTAL_READS"])
  mapped_primary_reads += $(column["PF_READS_ALIGNED"])
  if ($1 == "PAIR") {
    paired_primary_reads += $(column["TOTAL_READS"])
  }
  next
}

mode == "summary" && source == "samtools" {
  if ($4 == "primary" && NF == 4) {
    primary_reads = $1 + $3
  } else if ($4 == "primary" && $5 == "mapped") {
    mapped_primary_reads = $1 + $3
  } else if ($4 == "paired" && $5 == "in" && $6 == "sequencing") {
    paired_primary_reads = $1 + $3
  }
  next
}

mode != "summary" && NR > 1 {
  value[$1] = $2
}

END {
  if (mode == "summary") {
    if (primary_reads <= 0 || mapped_primary_reads <= 0 || paired_primary_reads <= 0 || mapped_primary_reads > primary_reads || paired_primary_reads > primary_reads) {
      exit 2
    }
    print "metric", "value"
    print "primary_reads", sprintf("%.0f", primary_reads)
    print "mapped_primary_reads", sprintf("%.0f", mapped_primary_reads)
    print "mapped_primary_fraction", sprintf("%.12f", mapped_primary_reads / primary_reads)
    print "paired_primary_reads", sprintf("%.0f", paired_primary_reads)
  } else if (mode == "bundle") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.mapping_summary.bundle.v1\",\"stage_id\":\"bam.mapping_summary\",\"tool_id\":\"%s\",\"sample_id\":\"%s\",\"primary_reads\":%s,\"mapped_primary_reads\":%s,\"mapped_primary_fraction\":%s,\"paired_primary_reads\":%s}\n", tool, sample_id, value["primary_reads"], value["mapped_primary_reads"], value["mapped_primary_fraction"], value["paired_primary_reads"]
  } else if (mode == "probe") {
    complete = (value["primary_reads"] > 0 && value["mapped_primary_reads"] > 0 && value["paired_primary_reads"] > 0) ? "true" : "false"
    printf "{\"schema_version\":\"bijux.benchmark.downstream_probe.v1\",\"stage_id\":\"benchmark.bam.mapping_summary.compare\",\"input_contract\":\"bijux.benchmark.bam.mapping_summary.bundle.v1\",\"complete\":%s}\n", complete
  } else if (mode == "metrics") {
    printf "{\"primary_reads\":%s,\"mapped_primary_reads\":%s,\"mapped_primary_fraction\":%s,\"paired_primary_reads\":%s,\"bam_input_bytes\":%s,\"bam_index_input_bytes\":%s,\"input_bytes\":%s,\"output_bytes\":%s}\n", value["primary_reads"], value["mapped_primary_reads"], value["mapped_primary_fraction"], value["paired_primary_reads"], bam_input_bytes, bam_index_input_bytes, input_bytes, output_bytes
  } else {
    exit 2
  }
}
