use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use reqwest::blocking::Client;
use serde::Serialize;
use zip::ZipArchive;

use crate::commands::benchmark::local_pipeline_dag::{
    load_validated_local_pipeline_dag_report, LocalPipelineReferenceAssets,
    LocalPipelineReferenceContext, LocalPipelineSnpEffAssets, LocalPipelineVariantAssets,
};

const PIPELINE_ASSET_MATERIALIZATION_SCHEMA_VERSION: &str =
    "bijux.hpc.pipeline_asset_materialization.v1";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PipelineAssetMaterializationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) pipeline_id: String,
    pub(crate) pipeline_config_path: String,
    pub(crate) operations_root: String,
    pub(crate) requested_surface: String,
    pub(crate) completed_surfaces: Vec<String>,
    pub(crate) reference_materialization: Option<ReferenceAssetMaterializationReport>,
    pub(crate) variant_materialization: Option<VariantAssetMaterializationReport>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ReferenceAssetMaterializationReport {
    pub(crate) surface: String,
    pub(crate) root_path: String,
    pub(crate) genome_archive_path: String,
    pub(crate) genome_fasta_path: String,
    pub(crate) fai_path: String,
    pub(crate) assembly_report_path: String,
    pub(crate) bwa_index_paths: Vec<String>,
    pub(crate) artifacts: Vec<MaterializedArtifact>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VariantAssetMaterializationReport {
    pub(crate) surface: String,
    pub(crate) root_path: String,
    pub(crate) source_vcf_path: String,
    pub(crate) old_header_path: String,
    pub(crate) updated_header_path: String,
    pub(crate) body_contigs_path: String,
    pub(crate) header_contigs_path: String,
    pub(crate) contig_map_path: String,
    pub(crate) missing_contigs_path: String,
    pub(crate) validation_ok_path: String,
    pub(crate) refseq_vcf_path: String,
    pub(crate) annotated_vcf_path: String,
    pub(crate) snpeff_runtime_root: String,
    pub(crate) snpeff_data_dir: String,
    pub(crate) snpeff_jar_path: String,
    pub(crate) snpeff_config_path: String,
    pub(crate) snpeff_db_ready_path: String,
    pub(crate) snpeff_database_zip_path: String,
    pub(crate) snpeff_core_zip_path: String,
    pub(crate) artifacts: Vec<MaterializedArtifact>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct MaterializedArtifact {
    pub(crate) role: String,
    pub(crate) path: String,
    pub(crate) action: String,
}

#[derive(Debug, Clone, Copy)]
enum RequestedSurface {
    All,
    Reference,
    Variant,
}

impl RequestedSurface {
    fn parse(raw: &str) -> Result<Self> {
        match raw.trim() {
            "all" => Ok(Self::All),
            "ref-prep" => Ok(Self::Reference),
            "snp-prep" => Ok(Self::Variant),
            other => Err(anyhow!(
                "unsupported pipeline asset surface `{other}`; expected one of: all, ref-prep, snp-prep"
            )),
        }
    }

    fn needs_reference(self) -> bool {
        matches!(self, Self::All | Self::Reference)
    }

    fn needs_variant(self) -> bool {
        matches!(self, Self::All | Self::Variant)
    }
}

pub(crate) fn pipeline_asset_materialization(
    repo_root: &Path,
    args: &crate::commands::cli::PipelineAssetMaterializationArgs,
) -> Result<PipelineAssetMaterializationReport> {
    render_pipeline_asset_materialization(
        repo_root,
        &resolve_candidate(repo_root, &args.config),
        args.operations_root.as_deref(),
        &args.surface,
    )
}

pub(crate) fn render_pipeline_asset_materialization(
    repo_root: &Path,
    config_path: &Path,
    operations_root: Option<&Path>,
    requested_surface: &str,
) -> Result<PipelineAssetMaterializationReport> {
    let pipeline = load_validated_local_pipeline_dag_report(repo_root, config_path)?;
    let requested_surface = RequestedSurface::parse(requested_surface)?;
    let operations_root =
        operations_root.map(|path| resolve_candidate(repo_root, path)).unwrap_or_else(|| {
            repo_root.join("artifacts/pipeline-operations").join(&pipeline.pipeline_id)
        });
    bijux_dna_infra::ensure_dir(&operations_root)?;

    let http = Client::builder()
        .build()
        .context("build HTTP client for pipeline asset materialization")?;

    let reference_materialization = if requested_surface.needs_reference() {
        let assets = pipeline.reference_assets.as_ref().ok_or_else(|| {
            anyhow!("pipeline asset materialization requested `ref-prep` but reference_assets is missing")
        })?;
        Some(materialize_reference_assets(&operations_root, assets, &http)?)
    } else {
        None
    };

    let variant_materialization = if requested_surface.needs_variant() {
        let assets = pipeline.variant_assets.as_ref().ok_or_else(|| {
            anyhow!(
                "pipeline asset materialization requested `snp-prep` but variant_assets is missing"
            )
        })?;
        let reference_assets = pipeline.reference_assets.as_ref().ok_or_else(|| {
            anyhow!("pipeline asset materialization requested `snp-prep` but reference_assets is missing")
        })?;
        Some(materialize_variant_assets(
            &operations_root,
            pipeline.reference_context.as_ref(),
            reference_assets,
            assets,
            &http,
        )?)
    } else {
        None
    };

    let mut completed_surfaces = Vec::new();
    if reference_materialization.is_some() {
        completed_surfaces.push("ref-prep".to_string());
    }
    if variant_materialization.is_some() {
        completed_surfaces.push("snp-prep".to_string());
    }

    Ok(PipelineAssetMaterializationReport {
        schema_version: PIPELINE_ASSET_MATERIALIZATION_SCHEMA_VERSION,
        pipeline_id: pipeline.pipeline_id,
        pipeline_config_path: config_path.display().to_string(),
        operations_root: operations_root.display().to_string(),
        requested_surface: requested_surface_label(requested_surface).to_string(),
        completed_surfaces,
        reference_materialization,
        variant_materialization,
    })
}

fn materialize_reference_assets(
    operations_root: &Path,
    assets: &LocalPipelineReferenceAssets,
    http: &Client,
) -> Result<ReferenceAssetMaterializationReport> {
    let root = operations_root.join("reference").join(&assets.directory_name);
    bijux_dna_infra::ensure_dir(&root)?;

    let genome_archive_path = root.join(&assets.genome_filename);
    let genome_fasta_path = decompressed_reference_fasta_path(&root, &assets.genome_filename);
    let fai_path = PathBuf::from(format!("{}.fai", genome_fasta_path.display()));
    let assembly_report_path = root.join(&assets.assembly_report_filename);
    let bwa_index_paths = bwa_index_paths(&genome_fasta_path);

    let mut artifacts = Vec::new();
    artifacts.push(download_file(
        http,
        &assets.genome_url,
        &genome_archive_path,
        "reference_fasta_gz",
    )?);
    artifacts.push(download_file(
        http,
        &assets.assembly_report_url,
        &assembly_report_path,
        "assembly_report",
    )?);
    artifacts.push(decompress_gzip_file(
        &genome_archive_path,
        &genome_fasta_path,
        "reference_fasta",
    )?);
    artifacts.push(build_fai_file(&genome_fasta_path, &fai_path)?);
    let bwa_action = ensure_bwa_index(&genome_fasta_path, &bwa_index_paths)?;
    for path in &bwa_index_paths {
        artifacts.push(MaterializedArtifact {
            role: bwa_role(path),
            path: path.display().to_string(),
            action: bwa_action.clone(),
        });
    }

    Ok(ReferenceAssetMaterializationReport {
        surface: "ref-prep".to_string(),
        root_path: root.display().to_string(),
        genome_archive_path: genome_archive_path.display().to_string(),
        genome_fasta_path: genome_fasta_path.display().to_string(),
        fai_path: fai_path.display().to_string(),
        assembly_report_path: assembly_report_path.display().to_string(),
        bwa_index_paths: bwa_index_paths.iter().map(|path| path.display().to_string()).collect(),
        artifacts,
    })
}

fn materialize_variant_assets(
    operations_root: &Path,
    reference_context: Option<&LocalPipelineReferenceContext>,
    reference_assets: &LocalPipelineReferenceAssets,
    assets: &LocalPipelineVariantAssets,
    http: &Client,
) -> Result<VariantAssetMaterializationReport> {
    let root = operations_root.join("variants").join(&assets.directory_name);
    bijux_dna_infra::ensure_dir(&root)?;

    let reference_root = operations_root.join("reference").join(&reference_assets.directory_name);
    let assembly_report_path = reference_root.join(&reference_assets.assembly_report_filename);
    let source_vcf_path = root.join(&assets.source_vcf_filename);
    let old_header_path = root.join("old_header.txt");
    let updated_header_path = root.join("updated_header.txt");
    let body_contigs_path = root.join("body_contig_ids.txt");
    let header_contigs_path = root.join("header_contig_ids.txt");
    let contig_map_path = root.join("contig_map.tsv");
    let missing_contigs_path = root.join("assembly_report_missing_contigs.txt");
    let validation_ok_path = root.join("contig_validation.ok");
    let refseq_vcf_path = root.join(&assets.refseq_vcf_filename);
    let annotated_vcf_path = root.join(&assets.annotated_vcf_filename);
    let snpeff_database_zip_path = root.join(&assets.snpeff.database_filename);
    let snpeff_core_zip_path = root.join(&assets.snpeff.core_filename);
    let snpeff_runtime_root = root.join("snpeff_runtime");
    let snpeff_data_dir = root.join("snpeff_data");
    let snpeff_jar_path = snpeff_runtime_root.join("snpEff.jar");
    let snpeff_config_path = snpeff_runtime_root.join("snpEff.local.config");
    let snpeff_db_ready_path =
        snpeff_runtime_root.join(format!("{}.db.ready", assets.snpeff.genome_id));

    let mut artifacts = Vec::new();
    artifacts.push(download_file(
        http,
        &reference_assets.assembly_report_url,
        &assembly_report_path,
        "assembly_report",
    )?);
    artifacts.push(download_file(http, &assets.source_vcf_url, &source_vcf_path, "source_vcf_gz")?);
    artifacts.push(download_file(
        http,
        &assets.snpeff.database_url,
        &snpeff_database_zip_path,
        "snpeff_database_zip",
    )?);
    artifacts.push(download_file(
        http,
        &assets.snpeff.core_url,
        &snpeff_core_zip_path,
        "snpeff_core_zip",
    )?);
    artifacts.push(extract_vcf_header(&source_vcf_path, &old_header_path)?);
    artifacts.extend(prepare_namespace_assets(
        &source_vcf_path,
        &assembly_report_path,
        &old_header_path,
        &updated_header_path,
        &body_contigs_path,
        &header_contigs_path,
        &contig_map_path,
        &missing_contigs_path,
        &validation_ok_path,
    )?);
    artifacts.push(rename_vcf_to_refseq(
        &source_vcf_path,
        &updated_header_path,
        &contig_map_path,
        &refseq_vcf_path,
    )?);
    artifacts.push(install_snpeff_database(
        &snpeff_database_zip_path,
        &snpeff_data_dir,
        &assets.snpeff,
    )?);
    artifacts.push(install_snpeff_core(&snpeff_core_zip_path, &snpeff_jar_path, &assets.snpeff)?);
    artifacts.push(write_snpeff_config(
        reference_context.map(|context| context.species_id.as_str()),
        &snpeff_data_dir,
        &snpeff_config_path,
        &assets.snpeff,
    )?);
    artifacts.push(write_ready_marker(&snpeff_db_ready_path, "ready\n", "snpeff_db_ready")?);
    artifacts.push(annotate_refseq_vcf(
        &refseq_vcf_path,
        &snpeff_jar_path,
        &snpeff_config_path,
        &annotated_vcf_path,
        &assets.snpeff.genome_id,
    )?);

    Ok(VariantAssetMaterializationReport {
        surface: "snp-prep".to_string(),
        root_path: root.display().to_string(),
        source_vcf_path: source_vcf_path.display().to_string(),
        old_header_path: old_header_path.display().to_string(),
        updated_header_path: updated_header_path.display().to_string(),
        body_contigs_path: body_contigs_path.display().to_string(),
        header_contigs_path: header_contigs_path.display().to_string(),
        contig_map_path: contig_map_path.display().to_string(),
        missing_contigs_path: missing_contigs_path.display().to_string(),
        validation_ok_path: validation_ok_path.display().to_string(),
        refseq_vcf_path: refseq_vcf_path.display().to_string(),
        annotated_vcf_path: annotated_vcf_path.display().to_string(),
        snpeff_runtime_root: snpeff_runtime_root.display().to_string(),
        snpeff_data_dir: snpeff_data_dir.display().to_string(),
        snpeff_jar_path: snpeff_jar_path.display().to_string(),
        snpeff_config_path: snpeff_config_path.display().to_string(),
        snpeff_db_ready_path: snpeff_db_ready_path.display().to_string(),
        snpeff_database_zip_path: snpeff_database_zip_path.display().to_string(),
        snpeff_core_zip_path: snpeff_core_zip_path.display().to_string(),
        artifacts,
    })
}

fn requested_surface_label(surface: RequestedSurface) -> &'static str {
    match surface {
        RequestedSurface::All => "all",
        RequestedSurface::Reference => "ref-prep",
        RequestedSurface::Variant => "snp-prep",
    }
}

fn resolve_candidate(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn decompressed_reference_fasta_path(root: &Path, genome_filename: &str) -> PathBuf {
    if let Some(stem) = genome_filename.strip_suffix(".gz") {
        root.join(stem)
    } else {
        root.join(format!("{genome_filename}.expanded"))
    }
}

fn download_file(
    http: &Client,
    url: &str,
    output_path: &Path,
    role: &str,
) -> Result<MaterializedArtifact> {
    if file_is_nonempty(output_path) {
        return Ok(MaterializedArtifact {
            role: role.to_string(),
            path: output_path.display().to_string(),
            action: "reuse".to_string(),
        });
    }
    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let mut response = http
        .get(url)
        .send()
        .with_context(|| format!("download {url}"))?
        .error_for_status()
        .with_context(|| format!("download {url}"))?;
    bijux_dna_infra::atomic_write_with(output_path, |file| {
        response.copy_to(file).map(|_| ()).map_err(std::io::Error::other)
    })
    .with_context(|| format!("write {}", output_path.display()))?;
    Ok(MaterializedArtifact {
        role: role.to_string(),
        path: output_path.display().to_string(),
        action: "download".to_string(),
    })
}

fn decompress_gzip_file(
    input_path: &Path,
    output_path: &Path,
    role: &str,
) -> Result<MaterializedArtifact> {
    if file_is_nonempty(output_path) {
        return Ok(MaterializedArtifact {
            role: role.to_string(),
            path: output_path.display().to_string(),
            action: "reuse".to_string(),
        });
    }
    let input = File::open(input_path).with_context(|| format!("open {}", input_path.display()))?;
    let mut decoder = GzDecoder::new(input);
    bijux_dna_infra::atomic_write_with(output_path, |file| {
        std::io::copy(&mut decoder, file).map(|_| ())
    })
    .with_context(|| format!("decompress {} to {}", input_path.display(), output_path.display()))?;
    Ok(MaterializedArtifact {
        role: role.to_string(),
        path: output_path.display().to_string(),
        action: "decompress".to_string(),
    })
}

fn build_fai_file(fasta_path: &Path, fai_path: &Path) -> Result<MaterializedArtifact> {
    if file_is_nonempty(fai_path) {
        return Ok(MaterializedArtifact {
            role: "reference_fai".to_string(),
            path: fai_path.display().to_string(),
            action: "reuse".to_string(),
        });
    }
    build_fai(fasta_path, fai_path)?;
    Ok(MaterializedArtifact {
        role: "reference_fai".to_string(),
        path: fai_path.display().to_string(),
        action: "build".to_string(),
    })
}

fn build_fai(fasta_path: &Path, fai_path: &Path) -> Result<()> {
    let mut input = BufReader::new(
        File::open(fasta_path).with_context(|| format!("open {}", fasta_path.display()))?,
    );
    let mut line = Vec::new();
    let mut sequence_name = None::<String>;
    let mut sequence_length = 0_u64;
    let mut sequence_offset = 0_u64;
    let mut line_bases = 0_u64;
    let mut line_width = 0_u64;
    let mut entries = String::new();

    loop {
        line.clear();
        let bytes_read = input
            .read_until(b'\n', &mut line)
            .with_context(|| format!("read {}", fasta_path.display()))?;
        if bytes_read == 0 {
            break;
        }
        if line.first() == Some(&b'>') {
            if let Some(name) = sequence_name.as_ref() {
                writeln!(
                    entries,
                    "{name}\t{sequence_length}\t{sequence_offset}\t{line_bases}\t{line_width}"
                )
                .map_err(std::io::Error::other)
                .context("render fai entry")?;
            }
            let header = String::from_utf8_lossy(&line);
            let next_name = header
                .trim_end_matches(['\r', '\n'])
                .trim_start_matches('>')
                .split_whitespace()
                .next()
                .ok_or_else(|| {
                    anyhow!("fasta header is missing sequence name in {}", fasta_path.display())
                })?;
            sequence_name = Some(next_name.to_string());
            sequence_length = 0;
            line_bases = 0;
            line_width = 0;
            sequence_offset = input
                .stream_position()
                .with_context(|| format!("read offset for {}", fasta_path.display()))?;
            continue;
        }
        let stripped = line
            .iter()
            .copied()
            .take_while(|byte| *byte != b'\r' && *byte != b'\n')
            .collect::<Vec<_>>();
        if stripped.is_empty() {
            continue;
        }
        if sequence_name.is_none() {
            bail!("encountered FASTA sequence data before any header in {}", fasta_path.display());
        }
        let stripped_len =
            u64::try_from(stripped.len()).map_err(|_| anyhow!("reference line length overflow"))?;
        let line_len =
            u64::try_from(line.len()).map_err(|_| anyhow!("reference line width overflow"))?;
        sequence_length += stripped_len;
        if line_bases == 0 {
            line_bases = stripped_len;
            line_width = line_len;
        }
    }

    if let Some(name) = sequence_name.as_ref() {
        writeln!(
            entries,
            "{name}\t{sequence_length}\t{sequence_offset}\t{line_bases}\t{line_width}"
        )
        .map_err(std::io::Error::other)
        .context("render terminal fai entry")?;
    }

    bijux_dna_infra::write_string(fai_path, &entries)
        .with_context(|| format!("write {}", fai_path.display()))?;
    Ok(())
}

fn bwa_index_paths(fasta_path: &Path) -> Vec<PathBuf> {
    ["amb", "ann", "bwt", "pac", "sa"]
        .iter()
        .map(|ext| PathBuf::from(format!("{}.{}", fasta_path.display(), ext)))
        .collect()
}

fn ensure_bwa_index(fasta_path: &Path, index_paths: &[PathBuf]) -> Result<String> {
    if index_paths.iter().all(|path| file_is_nonempty(path)) {
        return Ok("reuse".to_string());
    }
    let args = vec!["index".to_string(), fasta_path.display().to_string()];
    bijux_dna_stages_vcf::engine::run_boundary_checked_command("bwa", &args, None)
        .with_context(|| format!("build bwa index for {}", fasta_path.display()))?;
    Ok("build".to_string())
}

fn bwa_role(path: &Path) -> String {
    let ext = path.extension().and_then(|value| value.to_str()).unwrap_or("idx");
    format!("bwa_index_{ext}")
}

fn extract_vcf_header(vcf_path: &Path, output_path: &Path) -> Result<MaterializedArtifact> {
    if file_is_nonempty(output_path) {
        return Ok(MaterializedArtifact {
            role: "vcf_header".to_string(),
            path: output_path.display().to_string(),
            action: "reuse".to_string(),
        });
    }
    let mut header = String::new();
    for line in iter_gzip_lines(vcf_path)? {
        let line = line?;
        if !line.starts_with('#') {
            break;
        }
        header.push_str(&line);
        header.push('\n');
    }
    bijux_dna_infra::write_string(output_path, &header)
        .with_context(|| format!("write {}", output_path.display()))?;
    Ok(MaterializedArtifact {
        role: "vcf_header".to_string(),
        path: output_path.display().to_string(),
        action: "extract".to_string(),
    })
}

fn prepare_namespace_assets(
    vcf_path: &Path,
    assembly_report_path: &Path,
    old_header_path: &Path,
    updated_header_path: &Path,
    body_contigs_path: &Path,
    header_contigs_path: &Path,
    contig_map_path: &Path,
    missing_contigs_path: &Path,
    validation_ok_path: &Path,
) -> Result<Vec<MaterializedArtifact>> {
    if [
        updated_header_path,
        body_contigs_path,
        header_contigs_path,
        contig_map_path,
        missing_contigs_path,
        validation_ok_path,
    ]
    .iter()
    .all(|path| file_is_nonempty(path))
    {
        return Ok(vec![
            reused_artifact("updated_header", updated_header_path),
            reused_artifact("body_contigs", body_contigs_path),
            reused_artifact("header_contigs", header_contigs_path),
            reused_artifact("contig_map", contig_map_path),
            reused_artifact("missing_contigs", missing_contigs_path),
            reused_artifact("validation_ok", validation_ok_path),
        ]);
    }

    let body_contigs = collect_body_contigs(vcf_path)?;
    let mappings = parse_assembly_report(assembly_report_path)?;
    let header = VcfHeader::from_file(old_header_path)?;

    let mut resolved = Vec::new();
    let mut missing = Vec::new();
    for contig in &body_contigs {
        if let Some(row) = mappings.get(contig) {
            resolved.push((contig.clone(), row.clone()));
        } else {
            missing.push(contig.clone());
        }
    }

    let mapped_targets =
        resolved.iter().map(|(_source, row)| row.target.clone()).collect::<Vec<_>>();
    header.write_refseq_header(updated_header_path, &mapped_targets)?;
    bijux_dna_infra::write_string(body_contigs_path, &join_lines(&body_contigs))
        .with_context(|| format!("write {}", body_contigs_path.display()))?;
    bijux_dna_infra::write_string(header_contigs_path, &join_lines(&mapped_targets))
        .with_context(|| format!("write {}", header_contigs_path.display()))?;
    bijux_dna_infra::write_string(missing_contigs_path, &join_lines(&missing))
        .with_context(|| format!("write {}", missing_contigs_path.display()))?;

    let mut mapping_tsv = String::from(
        "source_contig\ttarget_contig\tsequence_name\tassigned_molecule\tgenbank_accession\trefseq_accession\tucsc_style_name\n",
    );
    for (source_contig, row) in &resolved {
        writeln!(
            mapping_tsv,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            source_contig,
            row.target,
            row.sequence_name,
            row.assigned_molecule,
            row.genbank_accession,
            row.refseq_accession,
            row.ucsc_style_name
        )
        .map_err(std::io::Error::other)
        .context("render contig map")?;
    }
    bijux_dna_infra::write_string(contig_map_path, &mapping_tsv)
        .with_context(|| format!("write {}", contig_map_path.display()))?;

    if !missing.is_empty() {
        bail!("assembly report does not cover all VCF body contigs: {}", missing.join(", "));
    }
    bijux_dna_infra::write_string(validation_ok_path, "ok\n")
        .with_context(|| format!("write {}", validation_ok_path.display()))?;

    Ok(vec![
        built_artifact("updated_header", updated_header_path),
        built_artifact("body_contigs", body_contigs_path),
        built_artifact("header_contigs", header_contigs_path),
        built_artifact("contig_map", contig_map_path),
        built_artifact("missing_contigs", missing_contigs_path),
        built_artifact("validation_ok", validation_ok_path),
    ])
}

fn collect_body_contigs(vcf_path: &Path) -> Result<Vec<String>> {
    let mut seen = BTreeMap::<String, ()>::new();
    let mut ordered = Vec::new();
    for line in iter_gzip_lines(vcf_path)? {
        let line = line?;
        if line.starts_with('#') {
            continue;
        }
        let contig = line
            .split('\t')
            .next()
            .ok_or_else(|| anyhow!("vcf record is missing CHROM in {}", vcf_path.display()))?;
        if seen.insert(contig.to_string(), ()).is_none() {
            ordered.push(contig.to_string());
        }
    }
    Ok(ordered)
}

fn iter_gzip_lines(path: &Path) -> Result<impl Iterator<Item = Result<String, std::io::Error>>> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);
    Ok(reader.lines())
}

#[derive(Debug, Clone)]
struct AssemblyReportRow {
    target: String,
    sequence_name: String,
    assigned_molecule: String,
    genbank_accession: String,
    refseq_accession: String,
    ucsc_style_name: String,
}

fn parse_assembly_report(report_path: &Path) -> Result<BTreeMap<String, AssemblyReportRow>> {
    let mut mappings = BTreeMap::new();
    let reader = BufReader::new(
        File::open(report_path).with_context(|| format!("open {}", report_path.display()))?,
    );
    for line in reader.lines() {
        let line = line.with_context(|| format!("read {}", report_path.display()))?;
        if line.starts_with('#') {
            continue;
        }
        let parts = line.split('\t').collect::<Vec<_>>();
        if parts.len() < 10 {
            continue;
        }
        let sequence_name = parts[0].to_string();
        let assigned_molecule = parts[2].to_string();
        let genbank_accession = parts[4].to_string();
        let refseq_accession = parts[6].to_string();
        let ucsc_style_name = parts[9].to_string();
        let target = if refseq_accession != "=" {
            refseq_accession.clone()
        } else {
            genbank_accession.clone()
        };
        let row = AssemblyReportRow {
            target,
            sequence_name: sequence_name.clone(),
            assigned_molecule: assigned_molecule.clone(),
            genbank_accession: genbank_accession.clone(),
            refseq_accession: refseq_accession.clone(),
            ucsc_style_name: ucsc_style_name.clone(),
        };
        for alias in
            [sequence_name, assigned_molecule, genbank_accession, refseq_accession, ucsc_style_name]
        {
            if !alias.is_empty() && alias != "=" {
                mappings.insert(alias, row.clone());
            }
        }
    }
    Ok(mappings)
}

#[derive(Debug, Clone)]
struct VcfHeader {
    meta_lines: Vec<String>,
    column_header: String,
}

impl VcfHeader {
    fn from_file(path: &Path) -> Result<Self> {
        let raw =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let mut meta_lines = Vec::new();
        let mut column_header = None::<String>;
        for line in raw.lines() {
            if line.starts_with("##") {
                if !line.starts_with("##contig=") {
                    meta_lines.push(format!("{line}\n"));
                }
                continue;
            }
            if line.starts_with("#CHROM") {
                column_header = Some(format!("{line}\n"));
            }
        }
        let column_header =
            column_header.ok_or_else(|| anyhow!("missing #CHROM line in {}", path.display()))?;
        Ok(Self { meta_lines, column_header })
    }

    fn write_refseq_header(&self, path: &Path, refseq_contigs: &[String]) -> Result<()> {
        let mut payload = String::new();
        for line in &self.meta_lines {
            payload.push_str(line);
        }
        for contig in refseq_contigs {
            writeln!(payload, "##contig=<ID={contig}>")
                .map_err(std::io::Error::other)
                .context("render refseq contig header")?;
        }
        payload.push_str(&self.column_header);
        bijux_dna_infra::write_string(path, &payload)
            .with_context(|| format!("write {}", path.display()))
    }
}

fn join_lines(values: &[String]) -> String {
    if values.is_empty() {
        String::new()
    } else {
        format!("{}\n", values.join("\n"))
    }
}

fn rename_vcf_to_refseq(
    vcf_path: &Path,
    updated_header_path: &Path,
    contig_map_path: &Path,
    output_path: &Path,
) -> Result<MaterializedArtifact> {
    if file_is_nonempty(output_path) {
        return Ok(reused_artifact("refseq_normalized_vcf_gz", output_path));
    }
    let header_text = std::fs::read_to_string(updated_header_path)
        .with_context(|| format!("read {}", updated_header_path.display()))?;
    let contig_map = load_contig_map(contig_map_path)?;
    bijux_dna_infra::atomic_write_with(output_path, |file| {
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(header_text.as_bytes())?;
        let reader = BufReader::new(GzDecoder::new(File::open(vcf_path)?));
        for line in reader.lines() {
            let line = line?;
            if line.starts_with('#') {
                continue;
            }
            let mut fields = line.split('\t').map(str::to_string).collect::<Vec<_>>();
            if let Some(target) =
                fields.first().and_then(|value| contig_map.get(value.as_str())).cloned()
            {
                fields[0] = target;
            }
            encoder.write_all(fields.join("\t").as_bytes())?;
            encoder.write_all(b"\n")?;
        }
        encoder.try_finish()
    })
    .with_context(|| format!("write {}", output_path.display()))?;
    Ok(built_artifact("refseq_normalized_vcf_gz", output_path))
}

fn load_contig_map(path: &Path) -> Result<BTreeMap<String, String>> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut mappings = BTreeMap::new();
    for line in raw.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 2 {
            continue;
        }
        mappings.insert(fields[0].to_string(), fields[1].to_string());
    }
    Ok(mappings)
}

fn install_snpeff_database(
    zip_path: &Path,
    data_dir: &Path,
    assets: &LocalPipelineSnpEffAssets,
) -> Result<MaterializedArtifact> {
    let genome_dir = data_dir.join(&assets.genome_id);
    let predictor = genome_dir.join("snpEffectPredictor.bin");
    if file_is_nonempty(&predictor) {
        return Ok(reused_artifact("snpeff_database", &predictor));
    }
    bijux_dna_infra::ensure_dir(data_dir)?;
    let parent = genome_dir
        .parent()
        .ok_or_else(|| anyhow!("resolve parent directory for {}", genome_dir.display()))?;
    let staging_dir = parent.join(format!(".{}.staging", assets.genome_id));
    if staging_dir.exists() {
        bijux_dna_infra::remove_dir_all(&staging_dir)?;
    }
    bijux_dna_infra::ensure_dir(&staging_dir)?;

    let file = File::open(zip_path).with_context(|| format!("open {}", zip_path.display()))?;
    let mut archive =
        ZipArchive::new(file).with_context(|| format!("open {}", zip_path.display()))?;
    let prefixes = [format!("data/{}/", assets.genome_id), format!("{}/", assets.genome_id)];
    let mut copied = 0_usize;
    for index in 0..archive.len() {
        let mut member = archive
            .by_index(index)
            .with_context(|| format!("read {index} from {}", zip_path.display()))?;
        let normalized = member.name().trim_start_matches("./");
        if normalized.split('/').any(|part| part == "..") {
            bail!("unsafe archive member path in {}: {}", zip_path.display(), normalized);
        }
        let Some(active_prefix) =
            prefixes.iter().find(|prefix| normalized.starts_with(prefix.as_str()))
        else {
            continue;
        };
        let relative = normalized.trim_start_matches(active_prefix);
        if relative.is_empty() {
            continue;
        }
        let destination = staging_dir.join(relative);
        if member.is_dir() {
            bijux_dna_infra::ensure_dir(&destination)?;
            continue;
        }
        if let Some(parent) = destination.parent() {
            bijux_dna_infra::ensure_dir(parent)?;
        }
        bijux_dna_infra::atomic_write_with(&destination, |file| {
            std::io::copy(&mut member, file).map(|_| ())
        })
        .with_context(|| format!("extract {}", destination.display()))?;
        copied += 1;
    }
    if copied == 0 {
        bail!("could not find genome directory {} in {}", assets.genome_id, zip_path.display());
    }
    if genome_dir.exists() {
        bijux_dna_infra::remove_dir_all(&genome_dir)?;
    }
    bijux_dna_infra::rename(&staging_dir, &genome_dir)?;
    if !file_is_nonempty(&predictor) {
        bail!("snpEffectPredictor.bin missing after install for {}", assets.genome_id);
    }
    Ok(built_artifact("snpeff_database", &predictor))
}

fn install_snpeff_core(
    zip_path: &Path,
    jar_output: &Path,
    _assets: &LocalPipelineSnpEffAssets,
) -> Result<MaterializedArtifact> {
    if file_is_nonempty(jar_output) {
        return Ok(reused_artifact("snpeff_jar", jar_output));
    }
    let file = File::open(zip_path).with_context(|| format!("open {}", zip_path.display()))?;
    let mut archive =
        ZipArchive::new(file).with_context(|| format!("open {}", zip_path.display()))?;
    let mut jar_bytes = None::<Vec<u8>>;
    for index in 0..archive.len() {
        let mut member = archive
            .by_index(index)
            .with_context(|| format!("read {index} from {}", zip_path.display()))?;
        if Path::new(member.name()).file_name().and_then(|value| value.to_str())
            == Some("snpEff.jar")
        {
            let mut bytes = Vec::new();
            member
                .read_to_end(&mut bytes)
                .with_context(|| format!("read snpEff.jar from {}", zip_path.display()))?;
            jar_bytes = Some(bytes);
            break;
        }
    }
    let jar_bytes =
        jar_bytes.ok_or_else(|| anyhow!("snpEff.jar not found in {}", zip_path.display()))?;
    bijux_dna_infra::write_bytes(jar_output, jar_bytes)
        .with_context(|| format!("write {}", jar_output.display()))?;
    Ok(built_artifact("snpeff_jar", jar_output))
}

fn write_snpeff_config(
    species_id: Option<&str>,
    data_dir: &Path,
    output_path: &Path,
    assets: &LocalPipelineSnpEffAssets,
) -> Result<MaterializedArtifact> {
    let species_label = species_id.unwrap_or("species");
    let payload = format!(
        "data.dir = {}\n{}.genome : {} {}\n",
        data_dir.display(),
        assets.genome_id,
        species_label,
        assets.genome_id
    );
    write_string_if_changed(output_path, &payload, "snpeff_config")
}

fn write_ready_marker(path: &Path, payload: &str, role: &str) -> Result<MaterializedArtifact> {
    write_string_if_changed(path, payload, role)
}

fn write_string_if_changed(path: &Path, payload: &str, role: &str) -> Result<MaterializedArtifact> {
    if path.exists() {
        let existing =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        if existing == payload {
            return Ok(reused_artifact(role, path));
        }
    }
    bijux_dna_infra::write_string(path, payload)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(built_artifact(role, path))
}

fn annotate_refseq_vcf(
    refseq_vcf_path: &Path,
    snpeff_jar_path: &Path,
    snpeff_config_path: &Path,
    annotated_vcf_path: &Path,
    genome_id: &str,
) -> Result<MaterializedArtifact> {
    if file_is_nonempty(annotated_vcf_path) {
        return Ok(reused_artifact("annotated_vcf_gz", annotated_vcf_path));
    }
    let tmp_path = PathBuf::from(format!("{}.tmp", annotated_vcf_path.display()));
    let script = format!(
        "set -euo pipefail; mkdir -p {out_dir}; java -jar {jar} ann -noStats -c {config} {genome} {vcf} | gzip -c > {tmp}; test -s {tmp}; mv {tmp} {out}",
        out_dir = shell_quote_path(
            annotated_vcf_path.parent().ok_or_else(|| anyhow!(
                "annotated vcf path has no parent: {}",
                annotated_vcf_path.display()
            ))?
        ),
        jar = shell_quote_path(snpeff_jar_path),
        config = shell_quote_path(snpeff_config_path),
        genome = shell_quote(genome_id),
        vcf = shell_quote_path(refseq_vcf_path),
        tmp = shell_quote_path(&tmp_path),
        out = shell_quote_path(annotated_vcf_path),
    );
    let args = vec!["-c".to_string(), script];
    bijux_dna_stages_vcf::engine::run_boundary_checked_command("sh", &args, None)
        .with_context(|| format!("annotate {}", refseq_vcf_path.display()))?;
    Ok(built_artifact("annotated_vcf_gz", annotated_vcf_path))
}

fn shell_quote(raw: &str) -> String {
    format!("'{}'", raw.replace('\'', "'\"'\"'"))
}

fn shell_quote_path(path: &Path) -> String {
    shell_quote(&path.display().to_string())
}

fn file_is_nonempty(path: &Path) -> bool {
    path.metadata().map(|metadata| metadata.is_file() && metadata.len() > 0).unwrap_or(false)
}

fn reused_artifact(role: &str, path: &Path) -> MaterializedArtifact {
    MaterializedArtifact {
        role: role.to_string(),
        path: path.display().to_string(),
        action: "reuse".to_string(),
    }
}

fn built_artifact(role: &str, path: &Path) -> MaterializedArtifact {
    MaterializedArtifact {
        role: role.to_string(),
        path: path.display().to_string(),
        action: "build".to_string(),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::io::{Read, Write};
    use std::path::Path;

    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;

    use super::{
        build_fai, extract_vcf_header, install_snpeff_core, install_snpeff_database,
        load_contig_map, prepare_namespace_assets, rename_vcf_to_refseq, write_snpeff_config,
        LocalPipelineSnpEffAssets,
    };

    #[test]
    fn build_fai_tracks_reference_offsets() {
        let dir = tempdir().expect("tempdir");
        let fasta_path = dir.path().join("reference.fa");
        let fai_path = dir.path().join("reference.fa.fai");
        write_bytes(&fasta_path, b">chr1\nACGT\nAC\n>chr2\nTTAA\n");

        build_fai(&fasta_path, &fai_path).expect("build fai");

        let observed = std::fs::read_to_string(&fai_path).expect("read fai");
        assert_eq!(observed, "chr1\t6\t6\t4\t5\nchr2\t4\t20\t4\t5\n");
    }

    #[test]
    fn namespace_prep_and_refseq_rename_remap_contigs() {
        let dir = tempdir().expect("tempdir");
        let source_vcf = dir.path().join("equcab.vcf.gz");
        let old_header = dir.path().join("old_header.txt");
        let updated_header = dir.path().join("updated_header.txt");
        let body_contigs = dir.path().join("body_contigs.txt");
        let header_contigs = dir.path().join("header_contigs.txt");
        let contig_map = dir.path().join("contig_map.tsv");
        let missing_contigs = dir.path().join("missing.txt");
        let validation_ok = dir.path().join("validation.ok");
        let refseq_vcf = dir.path().join("equcab_refseq.vcf.gz");
        let assembly_report = dir.path().join("assembly_report.txt");

        write_gzip(
            &source_vcf,
            concat!(
                "##fileformat=VCFv4.2\n",
                "##contig=<ID=1>\n",
                "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\n",
                "1\t10\t.\tA\tG\t.\tPASS\t.\n",
                "2\t20\t.\tC\tT\t.\tPASS\t.\n"
            ),
        );
        write_bytes(
            &assembly_report,
            concat!(
                "# comment\n",
                "1\tassembled-molecule\t1\tChromosome\tCM000001.1\t=\tNC_009144.3\tPrimary Assembly\t.\tchr1\n",
                "2\tassembled-molecule\t2\tChromosome\tCM000002.1\t=\tNC_009145.3\tPrimary Assembly\t.\tchr2\n"
            )
            .as_bytes(),
        );

        extract_vcf_header(&source_vcf, &old_header).expect("extract header");
        prepare_namespace_assets(
            &source_vcf,
            &assembly_report,
            &old_header,
            &updated_header,
            &body_contigs,
            &header_contigs,
            &contig_map,
            &missing_contigs,
            &validation_ok,
        )
        .expect("prepare namespace");
        rename_vcf_to_refseq(&source_vcf, &updated_header, &contig_map, &refseq_vcf)
            .expect("rename vcf");

        let header = std::fs::read_to_string(&updated_header).expect("read updated header");
        assert!(header.contains("##contig=<ID=NC_009144.3>"));
        assert!(header.contains("##contig=<ID=NC_009145.3>"));

        let map = load_contig_map(&contig_map).expect("load contig map");
        assert_eq!(map.get("1"), Some(&"NC_009144.3".to_string()));
        assert_eq!(map.get("2"), Some(&"NC_009145.3".to_string()));

        let renamed = read_gzip(&refseq_vcf);
        assert!(renamed.contains("NC_009144.3\t10"));
        assert!(renamed.contains("NC_009145.3\t20"));
    }

    #[test]
    fn snpeff_runtime_installers_extract_expected_assets() {
        let dir = tempdir().expect("tempdir");
        let core_zip = dir.path().join("core.zip");
        let db_zip = dir.path().join("db.zip");
        let jar_path = dir.path().join("runtime/snpEff.jar");
        let data_dir = dir.path().join("data");
        let config_path = dir.path().join("runtime/snpEff.local.config");
        let assets = LocalPipelineSnpEffAssets {
            genome_id: "EquCab3.0.99".to_string(),
            database_url: "https://example.test/db.zip".to_string(),
            database_filename: "db.zip".to_string(),
            core_url: "https://example.test/core.zip".to_string(),
            core_filename: "core.zip".to_string(),
        };

        write_zip(&core_zip, &[("snpEff/snpEff.jar", b"jar-bits".as_slice())]);
        write_zip(
            &db_zip,
            &[("data/EquCab3.0.99/snpEffectPredictor.bin", b"predictor".as_slice())],
        );

        install_snpeff_core(&core_zip, &jar_path, &assets).expect("install snpeff core");
        install_snpeff_database(&db_zip, &data_dir, &assets).expect("install snpeff database");
        write_snpeff_config(Some("Equus caballus"), &data_dir, &config_path, &assets)
            .expect("write snpeff config");

        assert_eq!(std::fs::read(&jar_path).expect("read jar"), b"jar-bits");
        assert!(data_dir.join("EquCab3.0.99").join("snpEffectPredictor.bin").is_file());
        let config = std::fs::read_to_string(&config_path).expect("read config");
        assert!(config.contains("data.dir = "));
        assert!(config.contains("EquCab3.0.99.genome : Equus caballus EquCab3.0.99"));
    }

    fn write_gzip(path: &Path, payload: &str) {
        bijux_dna_infra::atomic_write_with(path, |file| {
            let mut encoder = GzEncoder::new(file, Compression::default());
            encoder.write_all(payload.as_bytes())?;
            let _ = encoder.finish()?;
            Ok(())
        })
        .expect("create gzip");
    }

    fn read_gzip(path: &Path) -> String {
        let file = File::open(path).expect("open gzip");
        let mut decoder = flate2::read::GzDecoder::new(file);
        let mut text = String::new();
        decoder.read_to_string(&mut text).expect("read gzip text");
        text
    }

    fn write_zip(path: &Path, entries: &[(&str, &[u8])]) {
        bijux_dna_infra::atomic_write_with(path, |file| {
            let mut writer = zip::ZipWriter::new(file);
            let options = SimpleFileOptions::default();
            for (name, payload) in entries {
                writer.start_file(name, options).expect("start zip member");
                writer.write_all(payload)?;
            }
            let _ = writer.finish().expect("finish zip");
            Ok(())
        })
        .expect("create zip");
    }

    fn write_bytes(path: &Path, payload: &[u8]) {
        bijux_dna_infra::atomic_write_with(path, |file| file.write_all(payload))
            .expect("write test payload");
    }
}
