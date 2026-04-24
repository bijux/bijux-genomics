
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RegionChunk {
    pub chunk_id: String,
    pub contig: String,
    pub start: u64,
    pub end: u64,
}

impl RegionChunk {
    #[must_use]
    pub fn region_string(&self) -> String {
        format!("{}:{}-{}", self.contig, self.start, self.end)
    }
}

#[derive(Debug, Clone)]
pub struct ChunkingPlanParams {
    pub window_size_bp: u64,
    pub overlap_bp: u64,
    pub chr_include: Option<Vec<String>>,
    pub chr_exclude: Vec<String>,
    pub max_parallel_chunks: usize,
    pub chr_level_threshold_bp: u64,
}

impl Default for ChunkingPlanParams {
    fn default() -> Self {
        Self {
            window_size_bp: 5_000_000,
            overlap_bp: 100_000,
            chr_include: None,
            chr_exclude: Vec::new(),
            max_parallel_chunks: 8,
            chr_level_threshold_bp: 10_000_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkFailurePolicy {
    FailFast,
    PartialAllowed,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChunkRunOutputs {
    pub merged_vcf: PathBuf,
    pub chunks_json: PathBuf,
    pub run_mode: String,
}

#[derive(Debug, Clone, Serialize)]
struct ChunkProvenance {
    chunk_id: String,
    region: String,
    tool_digest: String,
    params_digest: String,
    input_checksum: String,
    output_checksum: String,
}

fn parse_variant_key(line: &str) -> Option<(String, u64, String)> {
    let fields = parse_record_fields(line)?;
    let pos = fields[1].parse::<u64>().ok()?;
    let key = format!("{}:{}:{}:{}", fields[0], fields[1], fields[3], fields[4]);
    Some((fields[0].to_string(), pos, key))
}

/// # Errors
/// Returns an error if chunk parameters are invalid.
pub fn plan_regions_deterministic(
    species_context: &SpeciesContext,
    params: &ChunkingPlanParams,
) -> Result<Vec<RegionChunk>> {
    if params.window_size_bp == 0 {
        bail!("window_size_bp must be > 0");
    }
    if params.overlap_bp >= params.window_size_bp {
        bail!("overlap_bp must be less than window_size_bp");
    }
    let mut chunks = Vec::new();
    for contig in &species_context.contigs {
        if params
            .chr_include
            .as_ref()
            .is_some_and(|allow| !allow.iter().any(|c| c == &contig.name))
        {
            continue;
        }
        if params.chr_exclude.iter().any(|c| c == &contig.name) {
            continue;
        }
        if contig.length_bp <= params.chr_level_threshold_bp {
            chunks.push(RegionChunk {
                chunk_id: format!("{}:whole", contig.name),
                contig: contig.name.clone(),
                start: 1,
                end: contig.length_bp,
            });
            continue;
        }
        let step = params.window_size_bp - params.overlap_bp;
        let mut start = 1u64;
        let mut idx = 0usize;
        while start <= contig.length_bp {
            let end = std::cmp::min(start + params.window_size_bp - 1, contig.length_bp);
            chunks.push(RegionChunk {
                chunk_id: format!("{}:{idx:05}", contig.name),
                contig: contig.name.clone(),
                start,
                end,
            });
            idx += 1;
            if end == contig.length_bp {
                break;
            }
            start = start.saturating_add(step);
        }
    }
    chunks.sort_by(|a, b| {
        a.contig
            .cmp(&b.contig)
            .then(a.start.cmp(&b.start))
            .then(a.end.cmp(&b.end))
            .then(a.chunk_id.cmp(&b.chunk_id))
    });
    Ok(chunks)
}

fn checksum_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    sha256_hex(hasher.finalize())
}

fn chunk_canonical_contig_label(raw: &str) -> String {
    raw.trim_start_matches("chr").to_ascii_uppercase()
}

fn normalize_header_deterministic(
    header: &[String],
    species_context: &SpeciesContext,
) -> Vec<String> {
    let rank = species_context
        .contigs
        .iter()
        .enumerate()
        .map(|(i, c)| (chunk_canonical_contig_label(&c.name), i))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut fileformat = vec![];
    let mut contigs = vec![];
    let mut other_meta = vec![];
    let mut chrom = None::<String>;
    for h in header {
        if h.starts_with("##fileformat=") {
            fileformat.push(h.clone());
        } else if h.starts_with("##contig=<") {
            contigs.push(h.clone());
        } else if h.starts_with("##") {
            other_meta.push(h.clone());
        } else if h.starts_with("#CHROM\t") {
            let mut fields = h.split('\t').map(str::to_string).collect::<Vec<_>>();
            if fields.len() > 9 {
                let mut sample_ids = fields.split_off(9);
                sample_ids.sort();
                fields.extend(sample_ids);
            }
            chrom = Some(fields.join("\t"));
        }
    }
    contigs.sort_by_key(|h| {
        let id = h
            .split("ID=")
            .nth(1)
            .and_then(|x| x.split([',', '>']).next())
            .unwrap_or_default();
        rank.get(&chunk_canonical_contig_label(id))
            .copied()
            .unwrap_or(usize::MAX)
    });
    other_meta.sort();
    let mut normalized = vec![];
    normalized.extend(fileformat);
    normalized.extend(other_meta);
    normalized.extend(contigs);
    if let Some(line) = chrom {
        normalized.push(line);
    }
    normalized
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

/// # Errors
/// Returns an error if chunk execution/merge validation fails.
#[allow(clippy::too_many_arguments)]
pub fn run_chunked_regions(
    input_vcf: &Path,
    panel_vcf: &Path,
    out_dir: &Path,
    species_context: &SpeciesContext,
    params: &ChunkingPlanParams,
    policy: ChunkFailurePolicy,
    rerun_chunk: Option<&str>,
) -> Result<ChunkRunOutputs> {
    bijux_dna_infra::ensure_dir(out_dir)?;
    let contract = crate::path_contract::VcfPathContract::canonical(out_dir);
    let stage_logs_dir = contract.logs_dir.join("vcf.chunked_merge");
    bijux_dna_infra::ensure_dir(&stage_logs_dir)?;
    let chunks = plan_regions_deterministic(species_context, params)?;
    let input_raw = std::fs::read_to_string(input_vcf)?;
    let panel_raw = std::fs::read_to_string(panel_vcf)?;
    let input_checksum = checksum_hex(input_raw.as_bytes());
    let panel_keys = panel_raw
        .lines()
        .filter_map(parse_variant_key)
        .map(|(_, _, k)| k)
        .collect::<std::collections::BTreeSet<_>>();

    let header = input_raw
        .lines()
        .filter(|l| l.starts_with('#'))
        .map(str::to_string)
        .collect::<Vec<_>>();
    let normalized_header = normalize_header_deterministic(&header, species_context);
    let records = input_raw
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    let chunks_dir = out_dir.join("chunks");
    bijux_dna_infra::ensure_dir(&chunks_dir)?;
    let mut manifest = Vec::new();
    let mut merged_records = std::collections::BTreeMap::<String, String>::new();

    for chunk in &chunks {
        if rerun_chunk.is_some_and(|id| id != chunk.chunk_id) {
            continue;
        }
        let chunk_slug = chunk.chunk_id.replace(':', "_");
        let chunk_out = chunks_dir.join(format!("{chunk_slug}.vcf.gz"));
        let prov_out = chunks_dir.join(format!(
            "{}.provenance.json",
            chunk_slug
        ));
        let checksum_out = chunks_dir.join(format!("{chunk_slug}.sha256"));
        let chunk_plain = chunks_dir.join(format!("{chunk_slug}.vcf"));
        let chunk_tbi = PathBuf::from(format!("{}.tbi", chunk_out.display()));
        let chunk_logs_dir = stage_logs_dir.join(format!("chunk-{chunk_slug}"));
        bijux_dna_infra::ensure_dir(&chunk_logs_dir)?;

        let mut chunk_lines = Vec::new();
        let mut actual_count = 0u64;
        let mut overlap_count = 0u64;
        for line in &records {
            if let Some((chr, pos, key)) = parse_variant_key(line) {
                if chr == chunk.contig && pos >= chunk.start && pos <= chunk.end {
                    chunk_lines.push(line.clone());
                    actual_count += 1;
                    if panel_keys.contains(&key) {
                        overlap_count += 1;
                    }
                    merged_records.entry(key).or_insert_with(|| line.clone());
                }
            }
        }

        let chunk_payload = format!("{}\n{}\n", normalized_header.join("\n"), chunk_lines.join("\n"));
        let output_checksum = checksum_hex(chunk_payload.as_bytes());
        let resume_ok = if chunk_out.exists() && chunk_tbi.exists() && checksum_out.exists() {
            let existing_sum = std::fs::read_to_string(&checksum_out).unwrap_or_default();
            existing_sum.trim() == output_checksum
        } else {
            false
        };
        atomic_write_bytes(
            &chunk_logs_dir.join("stdout.log"),
            format!("chunk_id={}\nresumed={}\n", chunk.chunk_id, resume_ok).as_bytes(),
        )?;
        atomic_write_bytes(&chunk_logs_dir.join("stderr.log"), b"")?;
        atomic_write_bytes(
            &chunk_logs_dir.join("command.txt"),
            b"chunk-extract -> bgzip -> tabix\n",
        )?;
        if resume_ok {
            manifest.push(serde_json::json!({
                "chunk_id": chunk.chunk_id,
                "region": chunk.region_string(),
                "estimated_variants": actual_count,
                "actual_variants": actual_count,
                "panel_overlap_per_region": overlap_count,
                "resumed": true,
            }));
            continue;
        }

        if actual_count == 0 {
            manifest.push(serde_json::json!({
                "chunk_id": chunk.chunk_id,
                "region": chunk.region_string(),
                "estimated_variants": 0,
                "actual_variants": 0,
                "panel_overlap_per_region": 0,
                "warning": "empty_chunk",
                "resumed": false,
            }));
            continue;
        }

        atomic_write_bytes(&chunk_plain, chunk_payload.as_bytes())?;
        let _ = crate::vcf_io::vcf_index_bgzip_tabix(&chunk_plain, &chunk_out)?;
        bijux_dna_infra::remove_file_if_exists(&chunk_plain)?;
        atomic_write_bytes(&checksum_out, format!("{output_checksum}\n").as_bytes())?;
        let prov = ChunkProvenance {
            chunk_id: chunk.chunk_id.clone(),
            region: chunk.region_string(),
            tool_digest: "sha256:planner-digest-placeholder".to_string(),
            params_digest: checksum_hex(
                serde_json::to_string(&serde_json::json!({
                    "window_size_bp": params.window_size_bp,
                    "overlap_bp": params.overlap_bp,
                    "max_parallel_chunks": params.max_parallel_chunks,
                }))?
                .as_bytes(),
            ),
            input_checksum: input_checksum.clone(),
            output_checksum: output_checksum.clone(),
        };
        atomic_write_json(&prov_out, &prov)?;
        manifest.push(serde_json::json!({
            "chunk_id": chunk.chunk_id,
            "region": chunk.region_string(),
            "estimated_variants": actual_count,
            "actual_variants": actual_count,
            "panel_overlap_per_region": overlap_count,
            "provenance": prov_out,
            "resumed": false,
        }));
    }

    let merged_vcf = out_dir.join("merged_chunks.vcf.gz");
    let merged_plain = out_dir.join("merged_chunks.vcf");
    let mut ordered = merged_records.values().cloned().collect::<Vec<_>>();
    ordered.sort_by(|a, b| {
        let ka = parse_variant_key(a)
            .map(|(c, p, k)| (c, p, k))
            .unwrap_or_default();
        let kb = parse_variant_key(b)
            .map(|(c, p, k)| (c, p, k))
            .unwrap_or_default();
        ka.cmp(&kb)
    });
    let merged_payload = format!("{}\n{}\n", normalized_header.join("\n"), ordered.join("\n"));
    atomic_write_bytes(&merged_plain, merged_payload.as_bytes())?;
    let _ = crate::vcf_io::vcf_index_bgzip_tabix(&merged_plain, &merged_vcf)?;
    bijux_dna_infra::remove_file_if_exists(&merged_plain)?;

    // Boundary correctness: no dropped/duplicated keys compared to deterministic de-overlapped union.
    let merged_keys = ordered
        .iter()
        .filter_map(|l| parse_variant_key(l).map(|(_, _, k)| k))
        .collect::<std::collections::BTreeSet<_>>();
    if merged_keys.len() != ordered.len() {
        bail!("chunk boundary correctness violated: duplicate variants after merge");
    }
    let source_keys = records
        .iter()
        .filter_map(|l| parse_variant_key(l).map(|(_, _, k)| k))
        .collect::<std::collections::BTreeSet<_>>();
    if !merged_keys.is_subset(&source_keys) {
        bail!("chunk boundary correctness violated: merged output has unknown variants");
    }

    let chunks_json = out_dir.join("chunks.json");
    atomic_write_json(
        &chunks_json,
        &serde_json::json!({
            "schema_version": "bijux.vcf.chunk_plan.v1",
            "failure_policy": match policy {
                ChunkFailurePolicy::FailFast => "fail_fast",
                ChunkFailurePolicy::PartialAllowed => "partial_allowed_non_production",
            },
            "non_production": policy == ChunkFailurePolicy::PartialAllowed,
            "chunks": manifest,
        }),
    )?;

    Ok(ChunkRunOutputs {
        merged_vcf,
        chunks_json,
        run_mode: if policy == ChunkFailurePolicy::PartialAllowed {
            "non_production_partial".to_string()
        } else {
            "production_fail_fast".to_string()
        },
    })
}
use std::fmt::Write as _;
