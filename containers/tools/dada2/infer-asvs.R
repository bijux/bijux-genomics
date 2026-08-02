#!/usr/bin/env Rscript

suppressPackageStartupMessages(library(dada2))

args <- commandArgs(trailingOnly = TRUE)
if (length(args) != 2L) {
  stop("usage: dada2 infer-asvs INPUT.fastq[.gz] OUTPUT.csv", call. = FALSE)
}

input <- normalizePath(args[[1]], mustWork = TRUE)
output <- args[[2]]
output_dir <- dirname(output)
dir.create(output_dir, recursive = TRUE, showWarnings = FALSE)
filtered <- file.path(output_dir, "filtered.fastq.gz")

filter_result <- filterAndTrim(
  input,
  filtered,
  maxN = 0,
  maxEE = 2,
  truncQ = 2,
  rm.phix = TRUE,
  compress = TRUE,
  multithread = TRUE
)
if (filter_result[[1, "reads.out"]] == 0L) {
  stop("DADA2 filtering retained no reads", call. = FALSE)
}

errors <- learnErrors(filtered, multithread = TRUE, verbose = TRUE)
dereplicated <- derepFastq(filtered, verbose = TRUE)
inferred <- dada(dereplicated, err = errors, multithread = TRUE)
sequence_table <- makeSequenceTable(inferred)
nonchimeric <- removeBimeraDenovo(
  sequence_table,
  method = "consensus",
  multithread = TRUE,
  verbose = TRUE
)
if (ncol(nonchimeric) == 0L) {
  stop("DADA2 inference produced no non-chimeric ASVs", call. = FALSE)
}

asv_table <- data.frame(
  sequence = colnames(nonchimeric),
  abundance = as.integer(nonchimeric[1, ]),
  row.names = NULL,
  check.names = FALSE
)
write.csv(asv_table, output, row.names = FALSE, quote = FALSE)
