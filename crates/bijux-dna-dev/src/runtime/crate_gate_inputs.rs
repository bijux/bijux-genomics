//! Bind the crate gate's two proofs to finalized native inventory without changing producer inputs.

use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Component, Path, PathBuf};

use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::runtime::workspace::Workspace;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
enum Proof {
    #[serde(rename = "typed_artifact_handoff")]
    TypedArtifactHandoff,
    #[serde(rename = "vcf_metric_registry")]
    VcfMetricRegistry,
}

impl Proof {
    fn package(self) -> &'static str {
        match self {
            Self::TypedArtifactHandoff => "bijux-dna-api",
            Self::VcfMetricRegistry => "bijux-dna-domain-vcf",
        }
    }

    fn binary(self) -> &'static str {
        match self {
            Self::TypedArtifactHandoff => "bijux_dna_api",
            Self::VcfMetricRegistry => "contracts",
        }
    }

    fn suite(self) -> &'static str {
        match self {
            Self::TypedArtifactHandoff => "bijux-dna-api",
            Self::VcfMetricRegistry => "bijux-dna-domain-vcf::contracts",
        }
    }

    fn kind(self) -> &'static str {
        match self {
            Self::TypedArtifactHandoff => "lib",
            Self::VcfMetricRegistry => "test",
        }
    }

    fn case(self) -> &'static str {
        match self {
            Self::TypedArtifactHandoff => {
                "runtime::validation::tests::typed_artifact_handoff_rejects_wrong_stage_inputs"
            }
            Self::VcfMetricRegistry => {
                "parsers::metric_registry::vcf_metric_registry_rejects_unregistered_stage_metrics"
            }
        }
    }
}

#[derive(Deserialize)]
struct Inventory {
    #[serde(rename = "test-count")]
    test_count: usize,
    #[serde(rename = "rust-suites", deserialize_with = "unique_map")]
    suites: BTreeMap<String, Suite>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Suite {
    package_name: String,
    binary_id: String,
    binary_name: String,
    kind: String,
    binary_path: PathBuf,
    cwd: PathBuf,
    status: String,
    #[serde(deserialize_with = "unique_map")]
    testcases: BTreeMap<String, Case>,
}

#[derive(Deserialize)]
struct Case {
    ignored: bool,
    #[serde(rename = "filter-match")]
    filter_match: FilterMatch,
}

#[derive(Deserialize)]
struct FilterMatch {
    status: String,
}

// Duplicate JSON keys must not silently replace a selected suite or proof case.
fn unique_map<'de, D, T>(deserializer: D) -> std::result::Result<BTreeMap<String, T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct UniqueMap<T>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for UniqueMap<T> {
        type Value = BTreeMap<String, T>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("an object with unique identity keys")
        }
        fn visit_map<M: serde::de::MapAccess<'de>>(
            self,
            mut access: M,
        ) -> std::result::Result<Self::Value, M::Error> {
            let mut values = BTreeMap::new();
            while let Some((key, value)) = access.next_entry::<String, T>()? {
                if values.insert(key.clone(), value).is_some() {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate inventory identity {key}"
                    )));
                }
            }
            Ok(values)
        }
    }
    deserializer.deserialize_map(UniqueMap(std::marker::PhantomData))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct PhysicalInput {
    path: PathBuf,
    sha256: String,
    bytes: u64,
    readonly: bool,
    #[cfg(unix)]
    mode: u32,
}

#[derive(Serialize)]
struct FinalizedInputs {
    inventory: PathBuf,
    inventory_sha256: String,
    executable: PathBuf,
    executable_sha256: String,
}

/// Revision authority supplied by the already-admitted executor source snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct CrateGateProducerIdentity {
    source_commit: String,
    source_patch_sha256: String,
}

impl CrateGateProducerIdentity {
    pub(crate) fn new(source_commit: &str, source_patch_sha256: &str) -> Result<Self> {
        ensure!(
            source_commit.len() == 40 && source_commit.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "crate proof producer commit must be 40 hexadecimal characters"
        );
        ensure!(
            source_patch_sha256.len() == 64
                && source_patch_sha256.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "crate proof producer patch digest must be SHA-256"
        );
        Ok(Self {
            source_commit: source_commit.to_string(),
            source_patch_sha256: source_patch_sha256.to_string(),
        })
    }
}

/// Captured producer authority can be shared by disjoint consumer preparations.
pub(crate) struct CrateGateProofInputs {
    producer_source: PathBuf,
    producer_identity: CrateGateProducerIdentity,
    target_root: PathBuf,
    original_inventory: PhysicalInput,
    raw_inventory: Value,
    proofs: BTreeMap<Proof, PhysicalInput>,
}

/// Consumer-specific, read-only wire files; producer input verification remains shared.
pub(crate) struct PublishedCrateGateProofInputs {
    manifest: PathBuf,
    files: Vec<PhysicalInput>,
    pub(crate) provenance: Value,
}

impl CrateGateProofInputs {
    pub(crate) fn capture(
        inventory_path: &Path,
        producer_source: &Path,
        target_root: &Path,
        producer_identity: CrateGateProducerIdentity,
    ) -> Result<Self> {
        admit_root(producer_source)?;
        admit_root(target_root)?;
        admit_workspace(producer_source)?;
        let original_inventory = fingerprint(inventory_path)?;
        let bytes = std::fs::read(inventory_path)?;
        ensure!(
            bijux_dna_infra::hash_bytes_sha256(&bytes) == original_inventory.sha256,
            "crate proof producer inventory changed during admission"
        );
        let inventory: Inventory = serde_json::from_slice(&bytes)
            .context("parse finalized complete crate proof inventory")?;
        let count = inventory.suites.values().try_fold(0_usize, |count, suite| {
            ensure!(
                suite.status == "listed",
                "complete proof inventory contains an unlisted suite"
            );
            ensure!(
                suite.testcases.values().all(|case| case.filter_match.status == "matches"),
                "complete proof inventory contains filtered cases"
            );
            count.checked_add(suite.testcases.len()).context("proof inventory case count overflow")
        })?;
        ensure!(count == inventory.test_count, "complete proof inventory case count differs");
        let mut proofs = BTreeMap::new();
        for proof in [Proof::TypedArtifactHandoff, Proof::VcfMetricRegistry] {
            let candidates = inventory
                .suites
                .iter()
                .filter(|(_, suite)| {
                    suite.package_name == proof.package() && suite.binary_name == proof.binary()
                })
                .collect::<Vec<_>>();
            ensure!(candidates.len() == 1, "crate proof requires exactly one owned suite");
            let (id, suite) = candidates[0];
            ensure!(
                id == proof.suite()
                    && suite.binary_id == proof.suite()
                    && suite.kind == proof.kind(),
                "crate proof suite identity/kind differs from its canonical owner"
            );
            ensure!(
                suite.cwd == producer_source.join("crates").join(proof.package()),
                "crate proof suite belongs to a different producer source"
            );
            let case =
                suite.testcases.get(proof.case()).context("required crate proof case is absent")?;
            ensure!(!case.ignored, "required crate proof case must not be ignored");
            ensure!(
                suite.binary_path.starts_with(target_root),
                "crate proof executable escapes compiled target"
            );
            let executable = fingerprint(&suite.binary_path)?;
            admit_native_executable(&executable.path)?;
            ensure!(
                !proofs.values().any(|previous: &PhysicalInput| previous.path == executable.path),
                "crate proofs cannot share an executable binding"
            );
            proofs.insert(proof, executable);
        }
        let captured = Self {
            producer_source: producer_source.to_path_buf(),
            producer_identity,
            target_root: target_root.to_path_buf(),
            original_inventory,
            raw_inventory: serde_json::from_slice(&bytes)?,
            proofs,
        };
        captured.verify_unchanged()?;
        Ok(captured)
    }

    pub(crate) fn publish_for_consumer(
        &self,
        consumer_source: &Path,
    ) -> Result<PublishedCrateGateProofInputs> {
        self.verify_unchanged()?;
        admit_root(consumer_source)?;
        admit_workspace(consumer_source)?;
        let workspace = Workspace { root: consumer_source.to_path_buf() };
        let output = workspace.admit_artifact_path(Path::new("artifacts/ci/crate-gate-proofs"))?;
        // The caller exclusively owns this isolated consumer. Refuse existing publication.
        let parent = output.parent().context("crate proof publication needs a parent")?;
        bijux_dna_infra::ensure_dir(parent)?;
        std::fs::create_dir(&output).context("reserve exclusive consumer proof artifacts")?;
        let mut inventory = self.raw_inventory.clone();
        for proof in self.proofs.keys() {
            inventory["rust-suites"][proof.suite()]["cwd"] =
                serde_json::to_value(consumer_source.join("crates").join(proof.package()))?;
        }
        let inventory_path = output.join("inventory.json");
        let consumer_inventory = publish_readonly(&inventory_path, &inventory)?;
        let bindings = self
            .proofs
            .iter()
            .map(|(proof, executable)| {
                (
                    *proof,
                    FinalizedInputs {
                        inventory: inventory_path.clone(),
                        inventory_sha256: consumer_inventory.sha256.clone(),
                        executable: executable.path.clone(),
                        executable_sha256: executable.sha256.clone(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let manifest = output.join("bindings.json");
        let binding_file = publish_readonly(&manifest, &bindings)?;
        let selected = self
            .proofs
            .iter()
            .map(|(proof, executable)| {
                (
                    *proof,
                    serde_json::json!({
                        "suite_id": proof.suite(), "case_id": proof.case(),
                        "producer_cwd": self.producer_source.join("crates").join(proof.package()),
                        "consumer_cwd": consumer_source.join("crates").join(proof.package()),
                        "compiled_executable": executable,
                    }),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let provenance = serde_json::json!({
            "schema_version": "bijux.ci.crate_gate_proof_inputs.v1",
            "producer_source_root": self.producer_source,
            "producer_source_identity": self.producer_identity,
            "consumer_source_root": consumer_source,
            "compiled_target_root": self.target_root,
            "original_producer_inventory": self.original_inventory,
            "consumer_inventory": consumer_inventory,
            "binding_manifest": binding_file,
            "proofs": selected,
            "remapping": "only_two_declared_suite_cwd_fields",
        });
        let provenance_file = publish_readonly(&output.join("provenance.json"), &provenance)?;
        Ok(PublishedCrateGateProofInputs {
            manifest,
            files: vec![consumer_inventory, binding_file, provenance_file],
            provenance,
        })
    }

    /// Verify once after all consumers, alongside the existing full compiled-input seal.
    pub(crate) fn verify_unchanged(&self) -> Result<()> {
        for input in std::iter::once(&self.original_inventory).chain(self.proofs.values()) {
            ensure!(
                fingerprint(&input.path)? == *input,
                "crate proof producer physical input changed"
            );
        }
        Ok(())
    }
}

impl PublishedCrateGateProofInputs {
    pub(crate) fn environment(&self) -> Result<(String, String)> {
        Ok((
            "BIJUX_CRATE_GATE_PROOF_INPUTS".to_string(),
            self.manifest
                .to_str()
                .context("crate proof binding manifest must be UTF-8")?
                .to_string(),
        ))
    }

    /// Call before consumer cleanup so immutable derived inputs remain accountable.
    pub(crate) fn verify_unchanged(&self) -> Result<()> {
        for input in &self.files {
            ensure!(
                fingerprint(&input.path)? == *input,
                "crate proof consumer physical input changed"
            );
        }
        Ok(())
    }
}

fn admit_root(path: &Path) -> Result<()> {
    ensure!(
        path.is_absolute()
            && path.is_dir()
            && !path
                .components()
                .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
            && path.canonicalize()? == path,
        "crate proof root must be an absolute canonical directory"
    );
    Ok(())
}

fn admit_workspace(root: &Path) -> Result<()> {
    for path in
        ["Cargo.toml", "crates/bijux-dna-api/Cargo.toml", "crates/bijux-dna-domain-vcf/Cargo.toml"]
    {
        ensure!(
            root.join(path).is_file(),
            "crate proof workspace lacks its declared package manifests"
        );
    }
    Ok(())
}

fn fingerprint(path: &Path) -> Result<PhysicalInput> {
    let metadata = path
        .symlink_metadata()
        .with_context(|| format!("inspect crate proof input {}", path.display()))?;
    ensure!(
        path.is_absolute()
            && metadata.is_file()
            && !metadata.file_type().is_symlink()
            && !path
                .components()
                .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
            && path.canonicalize()? == path,
        "crate proof input must be an absolute canonical regular file"
    );
    Ok(PhysicalInput {
        path: path.to_path_buf(),
        sha256: bijux_dna_infra::hash_file_sha256(path)?,
        bytes: metadata.len(),
        readonly: metadata.permissions().readonly(),
        #[cfg(unix)]
        mode: {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode()
        },
    })
}

fn publish_readonly(path: &Path, value: &impl Serialize) -> Result<PhysicalInput> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&bytes)?;
    drop(file);
    let mut permissions = path.metadata()?.permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions)?;
    let identity = fingerprint(path)?;
    ensure!(
        identity.sha256 == bijux_dna_infra::hash_bytes_sha256(&bytes),
        "consumer proof publication changed"
    );
    Ok(identity)
}

fn admit_native_executable(path: &Path) -> Result<()> {
    use std::io::Read;
    let mut magic = [0; 4];
    std::fs::File::open(path)?.read_exact(&mut magic)?;
    ensure!(
        magic == *b"\x7fELF"
            || magic == [0xcf, 0xfa, 0xed, 0xfe]
            || magic == [0xce, 0xfa, 0xed, 0xfe]
            || magic == [0xfe, 0xed, 0xfa, 0xcf]
            || magic == [0xfe, 0xed, 0xfa, 0xce]
            || magic[..2] == *b"MZ",
        "crate proof input must be a native compiled executable"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        ensure!(
            path.metadata()?.permissions().mode() & 0o111 != 0,
            "crate proof input is not executable"
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "crate_gate_inputs/contracts.rs"]
mod contracts;
