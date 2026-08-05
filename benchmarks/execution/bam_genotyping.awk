BEGIN {
  FS = "\t"
  OFS = "\t"
}

mode == "summary" {
  total_records++
  depth = $6 + 0
  genotype = $5
  if ($6 != "." && depth > 0) {
    covered_sites++
    depth_sum += depth
  }
  if (genotype != "." && genotype != "./." && genotype != ".|.") {
    called_genotypes++
    split(genotype, alleles, /[|\/]/)
    for (allele_index in alleles) {
      if (alleles[allele_index] != "." && alleles[allele_index] + 0 > 0) {
        alternate_genotypes++
        break
      }
    }
  }
  next
}

mode != "summary" && FNR == 1 {
  next
}

mode != "summary" {
  values[$1] = $2
  next
}

END {
  if (mode == "summary") {
    if (total_records == 0 || covered_sites == 0 || called_genotypes == 0) {
      exit 2
    }
    print "metric", "value"
    print "total_records", total_records
    print "covered_sites", covered_sites
    print "called_genotypes", called_genotypes
    print "alternate_genotypes", alternate_genotypes
    printf "mean_depth_at_covered_sites\t%.12f\n", depth_sum / covered_sites
  } else if (mode == "bundle") {
    printf "{\"schema_version\":\"bijux.benchmark.bam.genotyping.bundle.v1\",\"stage_id\":\"bam.genotyping\",\"tool_id\":\"%s\",\"sample_id\":\"%s\",\"total_records\":%d,\"covered_sites\":%d,\"called_genotypes\":%d,\"alternate_genotypes\":%d,\"mean_depth_at_covered_sites\":%.12f}\n", tool, sample_id, values["total_records"], values["covered_sites"], values["called_genotypes"], values["alternate_genotypes"], values["mean_depth_at_covered_sites"]
  } else if (mode == "metrics") {
    printf "{\"total_records\":%.0f,\"covered_sites\":%.0f,\"called_genotypes\":%.0f,\"alternate_genotypes\":%.0f,\"mean_depth_at_covered_sites\":%.12f,\"bam_input_bytes\":%.0f,\"reference_input_bytes\":%.0f,\"database_input_bytes\":%.0f,\"input_bytes\":%.0f,\"output_bytes\":%.0f}\n", values["total_records"], values["covered_sites"], values["called_genotypes"], values["alternate_genotypes"], values["mean_depth_at_covered_sites"], bam_input_bytes, reference_input_bytes, database_input_bytes, input_bytes, output_bytes
  } else if (mode == "probe") {
    complete = values["total_records"] > 0 && values["covered_sites"] > 0 && values["called_genotypes"] > 0
    printf "{\"schema_version\":\"bijux.benchmark.downstream_probe.v1\",\"stage_id\":\"benchmark.bam.genotyping.consume\",\"input_contract\":\"bijux.benchmark.bam.genotyping.bundle.v1\",\"complete\":%s}\n", complete ? "true" : "false"
    if (!complete) {
      exit 3
    }
  } else {
    exit 4
  }
}
