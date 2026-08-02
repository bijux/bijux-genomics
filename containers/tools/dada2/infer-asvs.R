#!/usr/bin/env Rscript

suppressPackageStartupMessages(library(dada2))

parse_args <- function(args) {
  values <- list(input = NULL, output_dir = NULL, sample_id = NULL, threads = 1L)
  index <- 1L
  while (index <= length(args)) {
    flag <- args[[index]]
    if (!(flag %in% c("--input", "--output-dir", "--sample-id", "--threads"))) {
      stop(sprintf("unknown argument: %s", flag), call. = FALSE)
    }
    if (index == length(args)) {
      stop(sprintf("missing value for %s", flag), call. = FALSE)
    }
    value <- args[[index + 1L]]
    if (flag == "--input") values$input <- value
    if (flag == "--output-dir") values$output_dir <- value
    if (flag == "--sample-id") values$sample_id <- value
    if (flag == "--threads") values$threads <- suppressWarnings(as.integer(value))
    index <- index + 2L
  }
  if (is.null(values$input) || is.null(values$output_dir)) {
    stop("--input and --output-dir are required", call. = FALSE)
  }
  if (is.na(values$threads) || values$threads < 1L) {
    stop("--threads must be a positive integer", call. = FALSE)
  }
  if (is.null(values$sample_id)) {
    values$sample_id <- sub("[.](fastq|fq)([.]gz)?$", "", basename(values$input), ignore.case = TRUE)
  }
  if (!grepl("^[A-Za-z0-9][A-Za-z0-9._-]*$", values$sample_id)) {
    stop("--sample-id must contain only letters, numbers, dots, underscores, or hyphens", call. = FALSE)
  }
  values
}

options <- parse_args(commandArgs(trailingOnly = TRUE))
input <- normalizePath(options$input, mustWork = TRUE)
output_dir <- options$output_dir
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
  multithread = options$threads
)
if (filter_result[[1, "reads.out"]] == 0L) {
  stop("DADA2 filtering retained no reads", call. = FALSE)
}

errors <- learnErrors(filtered, multithread = options$threads, verbose = TRUE)
dereplicated <- derepFastq(filtered, verbose = TRUE)
inferred <- dada(dereplicated, err = errors, multithread = options$threads)
sequence_table <- makeSequenceTable(inferred)
nonchimeric <- removeBimeraDenovo(
  sequence_table,
  method = "consensus",
  multithread = options$threads,
  verbose = TRUE
)
if (ncol(nonchimeric) == 0L) {
  stop("DADA2 inference produced no non-chimeric ASVs", call. = FALSE)
}

sequences <- colnames(nonchimeric)
abundances <- as.integer(nonchimeric[1, ])
asv_ids <- sprintf("ASV%d", seq_along(sequences))
asv_table <- data.frame(
  sample_id = options$sample_id,
  asv_id = asv_ids,
  sequence = sequences,
  abundance = abundances,
  row.names = NULL,
  check.names = FALSE
)
write.table(
  asv_table,
  file.path(output_dir, "asv-table.tsv"),
  sep = "\t",
  row.names = FALSE,
  quote = FALSE
)

write_fasta <- function(path, headers, values) {
  connection <- file(path, open = "wt")
  on.exit(close(connection), add = TRUE)
  for (index in seq_along(values)) {
    writeLines(c(sprintf(">%s", headers[[index]]), values[[index]]), connection)
  }
}

write_fasta(
  file.path(output_dir, "asv-sequences.fasta"),
  sprintf("%s;sample=%s;abundance=%d", asv_ids, options$sample_id, abundances),
  sequences
)
write_fasta(file.path(output_dir, "taxonomy-ready.fasta"), asv_ids, sequences)

fastq <- gzfile(file.path(output_dir, "taxonomy-ready.fastq.gz"), open = "wt")
for (index in seq_along(sequences)) {
  quality <- paste(rep("I", nchar(sequences[[index]])), collapse = "")
  writeLines(c(sprintf("@%s", asv_ids[[index]]), sequences[[index]], "+", quality), fastq)
}
close(fastq)

report <- c(
  "{",
  '  "schema_version": "bijux.dada2.infer_asvs.v1",',
  sprintf('  "dada2_version": "%s",', as.character(packageVersion("dada2"))),
  sprintf('  "sample_id": "%s",', options$sample_id),
  sprintf('  "threads": %d,', options$threads),
  sprintf('  "reads_in": %d,', filter_result[[1, "reads.in"]]),
  sprintf('  "reads_filtered": %d,', filter_result[[1, "reads.out"]]),
  sprintf('  "asv_count": %d,', length(sequences)),
  sprintf('  "nonchimeric_reads": %d', sum(abundances)),
  "}"
)
writeLines(report, file.path(output_dir, "report.json"))
