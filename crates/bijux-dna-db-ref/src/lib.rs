use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ContigNormalizationPolicy {
    StrictOnly,
    DeterministicRemap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedSpeciesContext {
    pub context: SpeciesContext,
    pub supported_features: SupportedFeatures,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SupportedFeatures {
    pub sex_chr: bool,
    pub imputation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceBundle {
    pub bundle_id: String,
    pub species_id: String,
    pub build_id: String,
    pub fasta: String,
    pub fai: String,
    pub dict: String,
    pub contig_set_digest: String,
    pub mask_bed: Option<String>,
    pub regions_bed: Option<String>,
    pub source_lock_sha256: String,
    pub bundle_lock_sha256: String,
    pub normalization_policy: ContigNormalizationPolicy,
    pub remap_table: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceProvenance {
    pub species_id: String,
    pub build_id: String,
    pub bundle_id: String,
    pub contig_set_digest: String,
    pub source_lock_sha256: String,
    pub bundle_lock_sha256: String,
}

#[derive(Debug, Deserialize)]
struct BundlesConfig {
    #[serde(default)]
    bundle: Vec<BundleEntry>,
}

#[derive(Debug, Deserialize)]
struct BundleEntry {
    bundle_id: String,
    species_id: String,
    build_id: String,
    fasta: String,
    fai: String,
    dict: String,
    contig_set_digest: String,
    #[serde(default)]
    mask_bed: Option<String>,
    #[serde(default)]
    regions_bed: Option<String>,
    source_lock_sha256: String,
    bundle_lock_sha256: String,
    normalization_policy: String,
    #[serde(default)]
    remap: BTreeMap<String, String>,
    sex_system: String,
    par_policy: String,
    #[serde(default)]
    default_coverage_regime: Option<String>,
    #[serde(default)]
    supported_features: SupportedFeatureEntry,
    contigs: Vec<ContigEntry>,
}

#[derive(Debug, Default, Deserialize)]
struct SupportedFeatureEntry {
    #[serde(default)]
    sex_chr: bool,
    #[serde(default)]
    imputation: bool,
}

#[derive(Debug, Deserialize)]
struct ContigEntry {
    name: String,
    length_bp: u64,
}

#[derive(Debug, Deserialize)]
struct AliasesConfig {
    #[serde(default)]
    aliases: BTreeMap<String, String>,
    #[serde(default)]
    default_builds: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct CoverageRegimesConfig {
    #[serde(default)]
    species_profile: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PanelCatalogEntry {
    pub id: String,
    pub species_id: String,
    pub build_id: String,
    #[serde(default)]
    pub status: String,
    pub version: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub lock_ref: String,
    #[serde(default)]
    pub citation: Option<String>,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
    pub compatibility: CatalogCompatibility,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MapCatalogEntry {
    pub id: String,
    pub species_id: String,
    pub build_id: String,
    #[serde(default)]
    pub status: String,
    pub version: String,
    #[serde(default)]
    pub lock_ref: String,
    #[serde(default)]
    pub citation: Option<String>,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
    pub compatibility: MapCompatibility,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CatalogFileEntry {
    pub name: String,
    pub path: String,
    pub format: String,
    pub url: String,
    pub checksum_sha256: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CatalogCompatibility {
    #[serde(default)]
    pub tool_tags: Vec<String>,
    pub requires_phased: bool,
    pub supports_gl_input: bool,
    pub supports_minimac_m3vcf: bool,
    pub glimpse_reference_format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MapCompatibility {
    #[serde(default)]
    pub tool_tags: Vec<String>,
    pub coordinate_system: String,
}

#[derive(Debug, Deserialize)]
struct PanelsConfig {
    #[serde(default)]
    panel: Vec<PanelCatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct MapsConfig {
    #[serde(default)]
    map: Vec<MapCatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct PanelLocksConfig {
    #[serde(default)]
    locks: BTreeMap<String, PanelLockEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PanelLockEntry {
    pub species_id: String,
    pub build_id: String,
    pub panel_id: String,
    pub version: String,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
}

#[derive(Debug, Deserialize)]
struct MapLocksConfig {
    #[serde(default)]
    locks: BTreeMap<String, MapLockEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MapLockEntry {
    pub species_id: String,
    pub build_id: String,
    pub map_id: String,
    pub version: String,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn load_toml<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str::<T>(&raw).with_context(|| format!("parse {}", path.display()))
}

pub trait RefService: Send + Sync {
    /// # Errors
    /// Returns an error if species/build resolution configuration cannot be loaded.
    fn resolve_coverage_profile(&self, species: &str, build: &str) -> Result<Option<String>>;
    /// # Errors
    /// Returns an error if the species/build reference bundle cannot be resolved.
    fn resolve_reference_bundle(&self, species: &str, build: &str) -> Result<ReferenceBundle>;
    /// # Errors
    /// Returns an error if panel catalogs cannot be loaded or no matching panel is found.
    fn resolve_panel(
        &self,
        species: &str,
        build: &str,
        panel_id: Option<&str>,
    ) -> Result<PanelCatalogEntry>;
    /// # Errors
    /// Returns an error if map catalogs cannot be loaded or no matching map is found.
    fn resolve_map(
        &self,
        species: &str,
        build: &str,
        map_id: Option<&str>,
    ) -> Result<MapCatalogEntry>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RuntimeRefService;

impl RefService for RuntimeRefService {
    fn resolve_coverage_profile(&self, species: &str, build: &str) -> Result<Option<String>> {
        resolve_coverage_profile(species, build)
    }

    fn resolve_reference_bundle(&self, species: &str, build: &str) -> Result<ReferenceBundle> {
        resolve_reference_bundle(species, build)
    }

    fn resolve_panel(
        &self,
        species: &str,
        build: &str,
        panel_id: Option<&str>,
    ) -> Result<PanelCatalogEntry> {
        resolve_panel(species, build, panel_id)
    }

    fn resolve_map(
        &self,
        species: &str,
        build: &str,
        map_id: Option<&str>,
    ) -> Result<MapCatalogEntry> {
        resolve_map(species, build, map_id)
    }
}

#[must_use]
pub fn ref_service() -> &'static dyn RefService {
    static SERVICE: OnceLock<RuntimeRefService> = OnceLock::new();
    SERVICE.get_or_init(RuntimeRefService::default)
}

/// # Errors
/// Returns an error if alias config cannot be read or the alias is unknown.
pub fn resolve_species_alias(
    alias: &str,
    requested_build: Option<&str>,
) -> Result<(String, String)> {
    let path = workspace_root().join("configs/runtime/species_aliases.toml");
    let cfg: AliasesConfig = load_toml(&path)?;
    let canonical_species = cfg
        .aliases
        .get(&alias.to_ascii_lowercase())
        .cloned()
        .ok_or_else(|| anyhow!("unknown species alias: {alias}"))?;
    let build = requested_build
        .map(ToString::to_string)
        .or_else(|| cfg.default_builds.get(&canonical_species).cloned())
        .ok_or_else(|| {
            anyhow!("no build provided and no default build for species {canonical_species}")
        })?;
    Ok((canonical_species, build))
}

/// # Errors
/// Returns an error if coverage profile config cannot be read.
pub fn resolve_coverage_profile(species: &str, build: &str) -> Result<Option<String>> {
    let path = workspace_root().join("configs/runtime/coverage_regimes.toml");
    let cfg: CoverageRegimesConfig = load_toml(&path)?;
    let key = format!("{species}:{build}");
    Ok(cfg.species_profile.get(&key).cloned())
}

/// # Errors
/// Returns an error when the species/build bundle cannot be resolved.
pub fn resolve_species_context(species: &str, build: &str) -> Result<ResolvedSpeciesContext> {
    let bundle = resolve_bundle_entry(species, build)?;
    let default_coverage_regime = bundle
        .default_coverage_regime
        .as_deref()
        .map(parse_coverage_regime)
        .transpose()?;
    let context = SpeciesContext {
        species_id: bundle.species_id.clone(),
        build_id: bundle.build_id.clone(),
        contig_set_digest: bundle.contig_set_digest.clone(),
        contigs: bundle
            .contigs
            .iter()
            .map(|c| ContigSpec {
                name: c.name.clone(),
                length_bp: c.length_bp,
            })
            .collect(),
        sex_system: bundle.sex_system.clone(),
        par_policy: bundle.par_policy.clone(),
        default_coverage_regime,
    };
    Ok(ResolvedSpeciesContext {
        context,
        supported_features: SupportedFeatures {
            sex_chr: bundle.supported_features.sex_chr,
            imputation: bundle.supported_features.imputation,
        },
    })
}

/// # Errors
/// Returns an error when the reference bundle cannot be resolved or violates contract.
pub fn resolve_reference_bundle(species: &str, build: &str) -> Result<ReferenceBundle> {
    let bundle = resolve_bundle_entry(species, build)?;
    validate_sha256(&bundle.source_lock_sha256, "source_lock_sha256")?;
    validate_sha256(&bundle.bundle_lock_sha256, "bundle_lock_sha256")?;
    let normalization_policy = match bundle.normalization_policy.as_str() {
        "strict_only" => ContigNormalizationPolicy::StrictOnly,
        "deterministic_remap" => ContigNormalizationPolicy::DeterministicRemap,
        other => bail!("unknown normalization policy: {other}"),
    };
    if normalization_policy == ContigNormalizationPolicy::DeterministicRemap
        && bundle.remap.is_empty()
    {
        bail!("deterministic_remap requires non-empty remap table");
    }
    Ok(ReferenceBundle {
        bundle_id: bundle.bundle_id.clone(),
        species_id: bundle.species_id.clone(),
        build_id: bundle.build_id.clone(),
        fasta: bundle.fasta.clone(),
        fai: bundle.fai.clone(),
        dict: bundle.dict.clone(),
        contig_set_digest: bundle.contig_set_digest.clone(),
        mask_bed: bundle.mask_bed.clone(),
        regions_bed: bundle.regions_bed.clone(),
        source_lock_sha256: bundle.source_lock_sha256.clone(),
        bundle_lock_sha256: bundle.bundle_lock_sha256.clone(),
        normalization_policy,
        remap_table: bundle.remap.clone(),
    })
}

/// # Errors
/// Returns an error if policy forbids remapping and contig names differ.
pub fn normalize_contig_name(bundle: &ReferenceBundle, contig: &str) -> Result<String> {
    match bundle.normalization_policy {
        ContigNormalizationPolicy::StrictOnly => Ok(contig.to_string()),
        ContigNormalizationPolicy::DeterministicRemap => bundle
            .remap_table
            .get(contig)
            .cloned()
            .or_else(|| {
                if bundle.remap_table.values().any(|v| v == contig) {
                    Some(contig.to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("contig {contig} not in deterministic remap table")),
    }
}

#[must_use]
pub fn reference_provenance(
    species: &str,
    build: &str,
    bundle: &ReferenceBundle,
) -> ReferenceProvenance {
    ReferenceProvenance {
        species_id: species.to_string(),
        build_id: build.to_string(),
        bundle_id: bundle.bundle_id.clone(),
        contig_set_digest: bundle.contig_set_digest.clone(),
        source_lock_sha256: bundle.source_lock_sha256.clone(),
        bundle_lock_sha256: bundle.bundle_lock_sha256.clone(),
    }
}

/// # Errors
/// Returns an error if panel resolution fails.
pub fn resolve_panel(
    species: &str,
    build: &str,
    panel_id: Option<&str>,
) -> Result<PanelCatalogEntry> {
    let path = workspace_root().join("configs/vcf/panels/panels.toml");
    let cfg: PanelsConfig = load_toml(&path)?;
    let mut candidates = cfg
        .panel
        .into_iter()
        .filter(|p| p.species_id == species && p.build_id == build)
        .collect::<Vec<_>>();
    if let Some(id) = panel_id {
        candidates.retain(|p| p.id == id);
    }
    let panel = candidates
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no panel found for {species}:{build}"))?;
    if panel.license.trim().is_empty() {
        bail!("panel {} missing required license metadata", panel.id);
    }
    if panel.lock_ref.trim().is_empty() {
        bail!("panel {} missing required lock_ref metadata", panel.id);
    }
    let _ = resolve_panel_lock(&panel)?;
    Ok(panel)
}

/// # Errors
/// Returns an error if map resolution fails.
pub fn resolve_map(species: &str, build: &str, map_id: Option<&str>) -> Result<MapCatalogEntry> {
    let path = workspace_root().join("configs/vcf/maps/maps.toml");
    let cfg: MapsConfig = load_toml(&path)?;
    let mut candidates = cfg
        .map
        .into_iter()
        .filter(|m| m.species_id == species && m.build_id == build)
        .collect::<Vec<_>>();
    if let Some(id) = map_id {
        candidates.retain(|m| m.id == id);
    }
    let map = candidates
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no map found for {species}:{build}"))?;
    if map.lock_ref.trim().is_empty() {
        bail!("map {} missing required lock_ref metadata", map.id);
    }
    let _ = resolve_map_lock(&map)?;
    Ok(map)
}

fn parse_lock_ref(lock_ref: &str) -> Result<(&str, &str)> {
    let (path, anchor) = lock_ref
        .split_once('#')
        .ok_or_else(|| anyhow!("invalid lock_ref `{lock_ref}`: missing #anchor"))?;
    let key = anchor
        .strip_prefix("locks.")
        .ok_or_else(|| anyhow!("invalid lock_ref `{lock_ref}`: anchor must start with `locks.`"))?;
    if path.trim().is_empty() || key.trim().is_empty() {
        bail!("invalid lock_ref `{lock_ref}`: empty path or key");
    }
    Ok((path, key))
}

/// # Errors
/// Returns an error if panel lock metadata is missing or malformed.
pub fn resolve_panel_lock(panel: &PanelCatalogEntry) -> Result<PanelLockEntry> {
    let (lock_path, key) = parse_lock_ref(&panel.lock_ref)?;
    let path = workspace_root().join("configs/vcf/panels").join(lock_path);
    let cfg: PanelLocksConfig = load_toml(&path)?;
    let entry = cfg
        .locks
        .get(key)
        .ok_or_else(|| anyhow!("panel lock entry `{key}` not found in {}", path.display()))?
        .clone();
    if entry.panel_id != panel.id || entry.species_id != panel.species_id || entry.build_id != panel.build_id {
        bail!("panel lock entry does not match panel identity {}", panel.id);
    }
    if entry.files.is_empty() {
        bail!("panel lock entry {} has no files", panel.id);
    }
    for file in &entry.files {
        validate_sha256(&file.checksum_sha256, "panel lock checksum_sha256")?;
    }
    Ok(entry)
}

/// # Errors
/// Returns an error if map lock metadata is missing or malformed.
pub fn resolve_map_lock(map: &MapCatalogEntry) -> Result<MapLockEntry> {
    let (lock_path, key) = parse_lock_ref(&map.lock_ref)?;
    let path = workspace_root().join("configs/vcf/maps").join(lock_path);
    let cfg: MapLocksConfig = load_toml(&path)?;
    let entry = cfg
        .locks
        .get(key)
        .ok_or_else(|| anyhow!("map lock entry `{key}` not found in {}", path.display()))?
        .clone();
    if entry.map_id != map.id || entry.species_id != map.species_id || entry.build_id != map.build_id {
        bail!("map lock entry does not match map identity {}", map.id);
    }
    if entry.files.is_empty() {
        bail!("map lock entry {} has no files", map.id);
    }
    for file in &entry.files {
        validate_sha256(&file.checksum_sha256, "map lock checksum_sha256")?;
    }
    Ok(entry)
}

/// # Errors
/// Returns an error if tool compatibility requirements are not satisfied.
pub fn validate_imputation_tool_compatibility(
    tool_id: &str,
    panel: &PanelCatalogEntry,
    map: &MapCatalogEntry,
) -> Result<()> {
    if !panel.compatibility.tool_tags.iter().any(|x| x == tool_id) {
        bail!("panel {} not compatible with tool {}", panel.id, tool_id);
    }
    if !map.compatibility.tool_tags.iter().any(|x| x == tool_id) {
        bail!("map {} not compatible with tool {}", map.id, tool_id);
    }
    if tool_id == "minimac4" && !panel.compatibility.supports_minimac_m3vcf {
        bail!("minimac4 requires m3vcf-compatible panel representation");
    }
    if tool_id == "minimac4" && !panel.files.iter().any(|f| f.name == "panel_m3vcf") {
        bail!("minimac4 requires `panel_m3vcf` in panel files");
    }
    if tool_id == "glimpse"
        && panel
            .compatibility
            .glimpse_reference_format
            .trim()
            .is_empty()
    {
        bail!("GLIMPSE requires declared reference format");
    }
    if tool_id == "glimpse"
        && !matches!(
            panel.compatibility.glimpse_reference_format.as_str(),
            "bcf+sites" | "bcf" | "sites"
        )
    {
        bail!("GLIMPSE requires supported reference format (bcf+sites|bcf|sites)");
    }
    if matches!(tool_id, "impute5" | "minimac4") && map.compatibility.coordinate_system != "bp" {
        bail!("{tool_id} requires bp coordinate-system genetic map");
    }
    if tool_id == "impute5" && !panel.compatibility.requires_phased {
        bail!("impute5 requires phased panel compatibility");
    }
    if tool_id == "beagle" && !panel.compatibility.supports_gl_input {
        bail!("beagle requires panel compatibility with GL input");
    }
    Ok(())
}

fn resolve_bundle_entry(species: &str, build: &str) -> Result<BundleEntry> {
    let path = workspace_root().join("configs/runtime/reference_bundles.toml");
    let cfg: BundlesConfig = load_toml(&path)?;
    cfg.bundle
        .into_iter()
        .find(|entry| entry.species_id == species && entry.build_id == build)
        .ok_or_else(|| anyhow!("no reference bundle found for {species}:{build}"))
}

fn validate_sha256(value: &str, name: &str) -> Result<()> {
    let ascii_hex = value.chars().all(|c| c.is_ascii_hexdigit());
    if value.len() != 64 || !ascii_hex {
        bail!("{name} must be 64-char lowercase hex");
    }
    Ok(())
}

fn parse_coverage_regime(raw: &str) -> Result<bijux_dna_domain_vcf::taxonomy::CoverageRegime> {
    match raw {
        "lowcov_gl" => Ok(bijux_dna_domain_vcf::taxonomy::CoverageRegime::LowCovGl),
        "pseudohaploid" => Ok(bijux_dna_domain_vcf::taxonomy::CoverageRegime::Pseudohaploid),
        "diploid" => Ok(bijux_dna_domain_vcf::taxonomy::CoverageRegime::Diploid),
        _ => bail!("unknown coverage regime value: {raw}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn species_context_and_bundle_resolve() {
        let resolved = resolve_species_context("Homo sapiens", "GRCh38")
            .unwrap_or_else(|err| panic!("resolve species context: {err}"));
        assert_eq!(resolved.context.build_id, "GRCh38");
        assert!(resolved.supported_features.imputation);
        let bundle = resolve_reference_bundle("Homo sapiens", "GRCh38")
            .unwrap_or_else(|err| panic!("resolve reference bundle: {err}"));
        assert_eq!(bundle.bundle_id, "hsapiens_grch38_primary");
    }

    #[test]
    fn deterministic_remap_table_is_enforced() {
        let bundle = resolve_reference_bundle("Canis lupus", "CanFam4")
            .unwrap_or_else(|err| panic!("resolve reference bundle: {err}"));
        let mapped = normalize_contig_name(&bundle, "chr1")
            .unwrap_or_else(|err| panic!("normalize contig: {err}"));
        assert_eq!(mapped, "1");
    }

    #[test]
    fn panel_and_map_resolution_work() {
        let panel = resolve_panel("Homo sapiens", "GRCh38", None)
            .unwrap_or_else(|err| panic!("resolve panel: {err}"));
        let map = resolve_map("Homo sapiens", "GRCh38", None)
            .unwrap_or_else(|err| panic!("resolve map: {err}"));
        let panel_lock = resolve_panel_lock(&panel)
            .unwrap_or_else(|err| panic!("resolve panel lock: {err}"));
        let map_lock =
            resolve_map_lock(&map).unwrap_or_else(|err| panic!("resolve map lock: {err}"));
        assert!(!panel_lock.files.is_empty());
        assert!(!map_lock.files.is_empty());
        validate_imputation_tool_compatibility("glimpse", &panel, &map)
            .unwrap_or_else(|err| panic!("compatibility: {err}"));
    }

    #[test]
    fn minimac_requires_m3vcf_support() {
        let panel = resolve_panel("Homo sapiens", "GRCh38", Some("hsapiens_grch38_full"))
            .unwrap_or_else(|err| panic!("resolve panel: {err}"));
        let map = resolve_map("Homo sapiens", "GRCh38", Some("hsapiens_grch38_chr_map"))
            .unwrap_or_else(|err| panic!("resolve map: {err}"));
        let err = validate_imputation_tool_compatibility("minimac4", &panel, &map)
            .expect_err("full panel must refuse minimac4");
        assert!(err.to_string().contains("minimac4"));
    }

    #[test]
    fn invalid_lock_ref_is_rejected() {
        let panel = PanelCatalogEntry {
            id: "panel".to_string(),
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            status: "production".to_string(),
            version: "1.0.0".to_string(),
            license: "CC-BY-4.0".to_string(),
            lock_ref: "not_a_lock_ref".to_string(),
            citation: None,
            files: vec![],
            compatibility: CatalogCompatibility {
                tool_tags: vec!["glimpse".to_string()],
                requires_phased: true,
                supports_gl_input: true,
                supports_minimac_m3vcf: false,
                glimpse_reference_format: "bcf+sites".to_string(),
            },
        };
        let err = resolve_panel_lock(&panel).expect_err("invalid lock_ref must fail");
        assert!(err.to_string().contains("invalid lock_ref"));
    }
}
