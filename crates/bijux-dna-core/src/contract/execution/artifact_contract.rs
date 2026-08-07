use std::collections::{BTreeMap, BTreeSet};

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::foundation::{BijuxError, Result};

pub const ARTIFACT_CONTRACT_SCHEMA_V1: &str = "bijux.artifact_contract.v1";
pub const ARTIFACT_CONTRACT_CATALOG_SCHEMA_V1: &str = "bijux.artifact_contract_catalog.v1";

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactClassV1 {
    Fastq,
    Alignment,
    Variant,
    Reference,
    Auxiliary,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactFormatV1 {
    Fastq,
    Bam,
    Cram,
    Vcf,
    Bcf,
    Fasta,
    Tabular,
    Json,
    Html,
    Text,
    Directory,
    Database,
    Index,
    File,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactCompressionV1 {
    None,
    Gzip,
    Bgzf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactCardinalityV1 {
    One,
    Many,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactPairingV1 {
    NotApplicable,
    SingleEnd,
    PairedEnd,
    Interleaved,
    Merged,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactSortOrderV1 {
    NotApplicable,
    Unsorted,
    QueryName,
    Coordinate,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactPresenceV1 {
    Forbidden,
    Optional,
    Required,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactSidecarRoleV1 {
    Index,
    Checksum,
    ReferenceDictionary,
    Metadata,
    Provenance,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactValidationInvariantV1 {
    NonEmpty,
    FastqRecordStructure,
    PairedRecordSynchronization,
    CoordinateSorted,
    IndexReadable,
    SampleIdentityPresent,
    ReferenceIdentityPresent,
    HeaderReferenceConsistent,
    LineageRecorded,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArtifactSidecarContractV1 {
    pub role: ArtifactSidecarRoleV1,
    pub format: ArtifactFormatV1,
    pub presence: ArtifactPresenceV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArtifactLineageContractV1 {
    pub presence: ArtifactPresenceV1,
    pub minimum_parent_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArtifactContractV1 {
    #[schemars(schema_with = "artifact_contract_schema_version")]
    pub schema_version: String,
    pub contract_id: String,
    pub class: ArtifactClassV1,
    pub formats: Vec<ArtifactFormatV1>,
    pub compressions: Vec<ArtifactCompressionV1>,
    pub cardinality: ArtifactCardinalityV1,
    pub pairing: Vec<ArtifactPairingV1>,
    pub sort_orders: Vec<ArtifactSortOrderV1>,
    pub index: ArtifactPresenceV1,
    pub sample_identity: ArtifactPresenceV1,
    pub reference_identity: ArtifactPresenceV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sidecars: Vec<ArtifactSidecarContractV1>,
    pub lineage: ArtifactLineageContractV1,
    pub validation_invariants: Vec<ArtifactValidationInvariantV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArtifactContractCatalogV1 {
    #[schemars(schema_with = "artifact_contract_catalog_schema_version")]
    pub schema_version: String,
    pub contracts: Vec<ArtifactContractV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArtifactCompatibilityCheckV1 {
    pub dimension: String,
    pub compatible: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArtifactCompatibilityReportV1 {
    pub producer_contract_id: String,
    pub consumer_contract_id: String,
    pub compatible: bool,
    pub checks: Vec<ArtifactCompatibilityCheckV1>,
}

impl ArtifactContractCatalogV1 {
    #[must_use]
    pub fn contract(&self, contract_id: &str) -> Option<&ArtifactContractV1> {
        self.contracts.iter().find(|contract| contract.contract_id == contract_id)
    }

    /// # Errors
    /// Returns an error when the catalog or one of its contracts violates the governed model.
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != ARTIFACT_CONTRACT_CATALOG_SCHEMA_V1 {
            return Err(BijuxError::validation(format!(
                "artifact contract catalog schema_version must be {ARTIFACT_CONTRACT_CATALOG_SCHEMA_V1}"
            )));
        }
        if self.contracts.is_empty() {
            return Err(BijuxError::validation("artifact contract catalog must not be empty"));
        }
        let mut ids = BTreeSet::new();
        for contract in &self.contracts {
            contract.validate()?;
            if !ids.insert(contract.contract_id.as_str()) {
                return Err(BijuxError::validation(format!(
                    "artifact contract catalog contains duplicate contract_id {}",
                    contract.contract_id
                )));
            }
        }
        Ok(())
    }
}

impl ArtifactContractV1 {
    /// # Errors
    /// Returns an error when required guarantees are missing or internally inconsistent.
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != ARTIFACT_CONTRACT_SCHEMA_V1 {
            return Err(BijuxError::validation(format!(
                "artifact contract {} schema_version must be {ARTIFACT_CONTRACT_SCHEMA_V1}",
                self.contract_id
            )));
        }
        if !self.contract_id.starts_with("bijux.") || !self.contract_id.ends_with(".v1") {
            return Err(BijuxError::validation(format!(
                "artifact contract_id must be a versioned bijux identifier: {}",
                self.contract_id
            )));
        }
        validate_nonempty_unique(&self.formats, &self.contract_id, "formats")?;
        validate_nonempty_unique(&self.compressions, &self.contract_id, "compressions")?;
        validate_nonempty_unique(&self.pairing, &self.contract_id, "pairing")?;
        validate_nonempty_unique(&self.sort_orders, &self.contract_id, "sort_orders")?;
        validate_nonempty_unique(
            &self.validation_invariants,
            &self.contract_id,
            "validation_invariants",
        )?;
        validate_class_formats(self)?;
        validate_shape(self)?;
        validate_compression(self)?;
        validate_sidecars_and_lineage(self)
    }
}

/// Compare guarantees made by a producer with requirements declared by a consumer.
#[must_use]
pub fn artifact_compatibility(
    producer: &ArtifactContractV1,
    consumer: &ArtifactContractV1,
) -> ArtifactCompatibilityReportV1 {
    let mut checks = vec![check(
        "class",
        producer.class == consumer.class,
        format!("producer={:?}, consumer={:?}", producer.class, consumer.class),
    )];
    checks.push(set_check("formats", &producer.formats, &consumer.formats));
    checks.push(set_check("compressions", &producer.compressions, &consumer.compressions));
    checks.push(check(
        "cardinality",
        producer.cardinality == consumer.cardinality,
        format!("producer={:?}, consumer={:?}", producer.cardinality, consumer.cardinality),
    ));
    checks.push(set_check("pairing", &producer.pairing, &consumer.pairing));
    checks.push(set_check("sort_orders", &producer.sort_orders, &consumer.sort_orders));
    checks.push(presence_check("index", producer.index, consumer.index));
    checks.push(presence_check(
        "sample_identity",
        producer.sample_identity,
        consumer.sample_identity,
    ));
    checks.push(presence_check(
        "reference_identity",
        producer.reference_identity,
        consumer.reference_identity,
    ));
    checks.push(sidecar_check(producer, consumer));
    checks.push(presence_check("lineage", producer.lineage.presence, consumer.lineage.presence));
    checks.push(check(
        "lineage_parent_count",
        producer.lineage.minimum_parent_count >= consumer.lineage.minimum_parent_count,
        format!(
            "producer={}, consumer={}",
            producer.lineage.minimum_parent_count, consumer.lineage.minimum_parent_count
        ),
    ));
    checks.push(guarantee_set_check(
        "validation_invariants",
        &producer.validation_invariants,
        &consumer.validation_invariants,
    ));
    let compatible = checks.iter().all(|item| item.compatible);
    ArtifactCompatibilityReportV1 {
        producer_contract_id: producer.contract_id.clone(),
        consumer_contract_id: consumer.contract_id.clone(),
        compatible,
        checks,
    }
}

/// # Errors
/// Returns an error when either contract is invalid or producer guarantees cannot satisfy the
/// consumer contract.
pub fn ensure_artifact_compatibility(
    producer: &ArtifactContractV1,
    consumer: &ArtifactContractV1,
) -> Result<ArtifactCompatibilityReportV1> {
    producer.validate()?;
    consumer.validate()?;
    let report = artifact_compatibility(producer, consumer);
    if report.compatible {
        return Ok(report);
    }
    let failed = report
        .checks
        .iter()
        .filter(|item| !item.compatible)
        .map(|item| item.dimension.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Err(BijuxError::validation(format!(
        "artifact contract {} cannot satisfy {}: incompatible {failed}",
        producer.contract_id, consumer.contract_id
    )))
}

#[must_use]
pub fn artifact_contract_schema() -> Schema {
    schemars::schema_for!(ArtifactContractV1)
}

#[must_use]
pub fn artifact_contract_catalog_schema() -> Schema {
    schemars::schema_for!(ArtifactContractCatalogV1)
}

fn artifact_contract_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": ARTIFACT_CONTRACT_SCHEMA_V1,
    })
}

fn artifact_contract_catalog_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": ARTIFACT_CONTRACT_CATALOG_SCHEMA_V1,
    })
}

#[must_use]
pub fn canonical_artifact_contract_catalog() -> ArtifactContractCatalogV1 {
    use ArtifactClassV1::{Alignment, Auxiliary, Fastq, Reference, Variant};
    use ArtifactCompressionV1::{Bgzf, Gzip, None as Uncompressed};
    use ArtifactFormatV1::{
        Bam, Database, Directory, Fasta, Fastq as FastqFormat, File, Html, Index, Json, Tabular,
        Text, Vcf,
    };
    use ArtifactPairingV1::{Interleaved, Merged, NotApplicable, PairedEnd, SingleEnd};
    use ArtifactPresenceV1::{Forbidden, Optional, Required};
    use ArtifactSortOrderV1::{Coordinate, NotApplicable as SortNotApplicable};
    use ArtifactValidationInvariantV1::{
        CoordinateSorted, FastqRecordStructure, HeaderReferenceConsistent, IndexReadable,
        LineageRecorded, NonEmpty, PairedRecordSynchronization, ReferenceIdentityPresent,
        SampleIdentityPresent,
    };

    let fastq_contract = |id: &str, pairing, synchronized| ArtifactContractV1 {
        schema_version: ARTIFACT_CONTRACT_SCHEMA_V1.to_string(),
        contract_id: id.to_string(),
        class: Fastq,
        formats: vec![FastqFormat],
        compressions: vec![Uncompressed, Gzip],
        cardinality: ArtifactCardinalityV1::One,
        pairing: vec![pairing],
        sort_orders: vec![SortNotApplicable],
        index: Forbidden,
        sample_identity: Required,
        reference_identity: Forbidden,
        sidecars: vec![],
        lineage: required_lineage(),
        validation_invariants: if synchronized {
            vec![
                NonEmpty,
                FastqRecordStructure,
                PairedRecordSynchronization,
                SampleIdentityPresent,
                LineageRecorded,
            ]
        } else {
            vec![NonEmpty, FastqRecordStructure, SampleIdentityPresent, LineageRecorded]
        },
    };

    let mut contracts = vec![
        ArtifactContractV1 {
            schema_version: ARTIFACT_CONTRACT_SCHEMA_V1.to_string(),
            contract_id: "bijux.fastq.artifact.reads.v1".to_string(),
            class: Fastq,
            formats: vec![FastqFormat],
            compressions: vec![Uncompressed, Gzip],
            cardinality: ArtifactCardinalityV1::One,
            pairing: vec![SingleEnd, PairedEnd, Interleaved, Merged],
            sort_orders: vec![SortNotApplicable],
            index: Forbidden,
            sample_identity: Required,
            reference_identity: Forbidden,
            sidecars: vec![],
            lineage: required_lineage(),
            validation_invariants: vec![
                NonEmpty,
                FastqRecordStructure,
                SampleIdentityPresent,
                LineageRecorded,
            ],
        },
        fastq_contract("bijux.fastq.artifact.single_end.v1", SingleEnd, false),
        fastq_contract("bijux.fastq.artifact.paired_end.v1", PairedEnd, true),
        fastq_contract("bijux.fastq.artifact.interleaved.v1", Interleaved, true),
        fastq_contract("bijux.fastq.artifact.merged.v1", Merged, false),
        ArtifactContractV1 {
            schema_version: ARTIFACT_CONTRACT_SCHEMA_V1.to_string(),
            contract_id: "bijux.bam.artifact.coordinate_sorted_indexed.v1".to_string(),
            class: Alignment,
            formats: vec![Bam],
            compressions: vec![Bgzf],
            cardinality: ArtifactCardinalityV1::One,
            pairing: vec![SingleEnd, PairedEnd],
            sort_orders: vec![Coordinate],
            index: Required,
            sample_identity: Required,
            reference_identity: Required,
            sidecars: vec![required_index_sidecar(Index)],
            lineage: required_lineage(),
            validation_invariants: vec![
                NonEmpty,
                CoordinateSorted,
                IndexReadable,
                SampleIdentityPresent,
                ReferenceIdentityPresent,
                HeaderReferenceConsistent,
                LineageRecorded,
            ],
        },
        ArtifactContractV1 {
            schema_version: ARTIFACT_CONTRACT_SCHEMA_V1.to_string(),
            contract_id: "bijux.vcf.artifact.coordinate_sorted_indexed.v1".to_string(),
            class: Variant,
            formats: vec![Vcf],
            compressions: vec![Bgzf],
            cardinality: ArtifactCardinalityV1::One,
            pairing: vec![NotApplicable],
            sort_orders: vec![Coordinate],
            index: Required,
            sample_identity: Required,
            reference_identity: Required,
            sidecars: vec![required_index_sidecar(Index)],
            lineage: required_lineage(),
            validation_invariants: vec![
                NonEmpty,
                CoordinateSorted,
                IndexReadable,
                SampleIdentityPresent,
                ReferenceIdentityPresent,
                HeaderReferenceConsistent,
                LineageRecorded,
            ],
        },
        ArtifactContractV1 {
            schema_version: ARTIFACT_CONTRACT_SCHEMA_V1.to_string(),
            contract_id: "bijux.vcf.artifact.reference_panel_indexed.v1".to_string(),
            class: Variant,
            formats: vec![Vcf],
            compressions: vec![Bgzf],
            cardinality: ArtifactCardinalityV1::One,
            pairing: vec![NotApplicable],
            sort_orders: vec![Coordinate],
            index: Required,
            sample_identity: Forbidden,
            reference_identity: Required,
            sidecars: vec![required_index_sidecar(Index)],
            lineage: required_lineage(),
            validation_invariants: vec![
                NonEmpty,
                CoordinateSorted,
                IndexReadable,
                ReferenceIdentityPresent,
                HeaderReferenceConsistent,
                LineageRecorded,
            ],
        },
        auxiliary_contract(
            "bijux.reference.artifact.fasta.v1",
            Reference,
            Fasta,
            vec![Uncompressed, Gzip],
            Optional,
            Forbidden,
            Required,
            vec![NonEmpty, ReferenceIdentityPresent, LineageRecorded],
        ),
        auxiliary_contract(
            "bijux.auxiliary.artifact.table.v1",
            Auxiliary,
            Tabular,
            vec![Uncompressed, Gzip],
            Forbidden,
            Optional,
            Optional,
            vec![NonEmpty, LineageRecorded],
        ),
        auxiliary_contract(
            "bijux.auxiliary.artifact.json.v1",
            Auxiliary,
            Json,
            vec![Uncompressed],
            Forbidden,
            Optional,
            Optional,
            vec![NonEmpty, LineageRecorded],
        ),
        auxiliary_contract(
            "bijux.auxiliary.artifact.database.v1",
            Auxiliary,
            Database,
            vec![Uncompressed],
            Optional,
            Forbidden,
            Optional,
            vec![NonEmpty, LineageRecorded],
        ),
        auxiliary_contract(
            "bijux.auxiliary.artifact.directory.v1",
            Auxiliary,
            Directory,
            vec![Uncompressed],
            Forbidden,
            Optional,
            Optional,
            vec![NonEmpty, LineageRecorded],
        ),
        auxiliary_contract(
            "bijux.auxiliary.artifact.file.v1",
            Auxiliary,
            File,
            vec![Uncompressed, Gzip],
            Forbidden,
            Optional,
            Optional,
            vec![NonEmpty, LineageRecorded],
        ),
        auxiliary_contract(
            "bijux.auxiliary.artifact.html.v1",
            Auxiliary,
            Html,
            vec![Uncompressed, Gzip],
            Forbidden,
            Optional,
            Optional,
            vec![NonEmpty, LineageRecorded],
        ),
        auxiliary_contract(
            "bijux.auxiliary.artifact.index.v1",
            Auxiliary,
            Index,
            vec![Uncompressed],
            Forbidden,
            Optional,
            Optional,
            vec![NonEmpty, LineageRecorded],
        ),
        auxiliary_contract(
            "bijux.auxiliary.artifact.text.v1",
            Auxiliary,
            Text,
            vec![Uncompressed, Gzip],
            Forbidden,
            Optional,
            Optional,
            vec![NonEmpty, LineageRecorded],
        ),
    ];
    contracts.sort_by(|left, right| left.contract_id.cmp(&right.contract_id));
    ArtifactContractCatalogV1 {
        schema_version: ARTIFACT_CONTRACT_CATALOG_SCHEMA_V1.to_string(),
        contracts,
    }
}

fn auxiliary_contract(
    contract_id: &str,
    class: ArtifactClassV1,
    format: ArtifactFormatV1,
    compressions: Vec<ArtifactCompressionV1>,
    index: ArtifactPresenceV1,
    sample_identity: ArtifactPresenceV1,
    reference_identity: ArtifactPresenceV1,
    validation_invariants: Vec<ArtifactValidationInvariantV1>,
) -> ArtifactContractV1 {
    ArtifactContractV1 {
        schema_version: ARTIFACT_CONTRACT_SCHEMA_V1.to_string(),
        contract_id: contract_id.to_string(),
        class,
        formats: vec![format],
        compressions,
        cardinality: ArtifactCardinalityV1::One,
        pairing: vec![ArtifactPairingV1::NotApplicable],
        sort_orders: vec![ArtifactSortOrderV1::NotApplicable],
        index,
        sample_identity,
        reference_identity,
        sidecars: vec![],
        lineage: required_lineage(),
        validation_invariants,
    }
}

const fn required_lineage() -> ArtifactLineageContractV1 {
    ArtifactLineageContractV1 { presence: ArtifactPresenceV1::Required, minimum_parent_count: 1 }
}

const fn required_index_sidecar(format: ArtifactFormatV1) -> ArtifactSidecarContractV1 {
    ArtifactSidecarContractV1 {
        role: ArtifactSidecarRoleV1::Index,
        format,
        presence: ArtifactPresenceV1::Required,
    }
}

fn validate_nonempty_unique<T>(values: &[T], contract_id: &str, field: &str) -> Result<()>
where
    T: Ord,
{
    if values.is_empty() {
        return Err(BijuxError::validation(format!(
            "artifact contract {contract_id} {field} must not be empty"
        )));
    }
    if values.iter().collect::<BTreeSet<_>>().len() != values.len() {
        return Err(BijuxError::validation(format!(
            "artifact contract {contract_id} {field} must not contain duplicates"
        )));
    }
    Ok(())
}

fn validate_class_formats(contract: &ArtifactContractV1) -> Result<()> {
    let allowed = match contract.class {
        ArtifactClassV1::Fastq => vec![ArtifactFormatV1::Fastq],
        ArtifactClassV1::Alignment => vec![ArtifactFormatV1::Bam, ArtifactFormatV1::Cram],
        ArtifactClassV1::Variant => vec![ArtifactFormatV1::Vcf, ArtifactFormatV1::Bcf],
        ArtifactClassV1::Reference => vec![ArtifactFormatV1::Fasta],
        ArtifactClassV1::Auxiliary => vec![
            ArtifactFormatV1::Tabular,
            ArtifactFormatV1::Json,
            ArtifactFormatV1::Html,
            ArtifactFormatV1::Text,
            ArtifactFormatV1::Directory,
            ArtifactFormatV1::Database,
            ArtifactFormatV1::Index,
            ArtifactFormatV1::File,
        ],
    };
    if contract.formats.iter().all(|format| allowed.contains(format)) {
        return Ok(());
    }
    Err(BijuxError::validation(format!(
        "artifact contract {} contains a format outside class {:?}",
        contract.contract_id, contract.class
    )))
}

fn validate_shape(contract: &ArtifactContractV1) -> Result<()> {
    let has_not_applicable = contract.pairing.contains(&ArtifactPairingV1::NotApplicable);
    let pairing_is_valid = match contract.class {
        ArtifactClassV1::Fastq | ArtifactClassV1::Alignment => !has_not_applicable,
        ArtifactClassV1::Variant | ArtifactClassV1::Reference | ArtifactClassV1::Auxiliary => {
            contract.pairing == [ArtifactPairingV1::NotApplicable]
        }
    };
    if !pairing_is_valid {
        return Err(BijuxError::validation(format!(
            "artifact contract {} has pairing incompatible with class {:?}",
            contract.contract_id, contract.class
        )));
    }
    if contract.class == ArtifactClassV1::Fastq
        && contract.sort_orders != [ArtifactSortOrderV1::NotApplicable]
    {
        return Err(BijuxError::validation(format!(
            "FASTQ artifact contract {} must use not_applicable sort order",
            contract.contract_id
        )));
    }
    for (presence, invariant, field) in [
        (contract.sample_identity, ArtifactValidationInvariantV1::SampleIdentityPresent, "sample"),
        (
            contract.reference_identity,
            ArtifactValidationInvariantV1::ReferenceIdentityPresent,
            "reference",
        ),
    ] {
        if presence == ArtifactPresenceV1::Required
            && !contract.validation_invariants.contains(&invariant)
        {
            return Err(BijuxError::validation(format!(
                "artifact contract {} requires {field} identity without its validation invariant",
                contract.contract_id
            )));
        }
    }
    if contract.sort_orders.contains(&ArtifactSortOrderV1::Coordinate)
        && !contract
            .validation_invariants
            .contains(&ArtifactValidationInvariantV1::CoordinateSorted)
    {
        return Err(BijuxError::validation(format!(
            "artifact contract {} allows coordinate sort without its validation invariant",
            contract.contract_id
        )));
    }
    Ok(())
}

fn validate_compression(contract: &ArtifactContractV1) -> Result<()> {
    for compression in &contract.compressions {
        let valid = match compression {
            ArtifactCompressionV1::None => true,
            ArtifactCompressionV1::Gzip => contract.formats.iter().all(|format| {
                matches!(
                    format,
                    ArtifactFormatV1::Fastq
                        | ArtifactFormatV1::Fasta
                        | ArtifactFormatV1::Tabular
                        | ArtifactFormatV1::Json
                        | ArtifactFormatV1::Html
                        | ArtifactFormatV1::Text
                        | ArtifactFormatV1::File
                        | ArtifactFormatV1::Vcf
                )
            }),
            ArtifactCompressionV1::Bgzf => contract
                .formats
                .iter()
                .all(|format| matches!(format, ArtifactFormatV1::Bam | ArtifactFormatV1::Vcf)),
        };
        if !valid {
            return Err(BijuxError::validation(format!(
                "artifact contract {} has incompatible compression {compression:?}",
                contract.contract_id
            )));
        }
    }
    Ok(())
}

fn validate_sidecars_and_lineage(contract: &ArtifactContractV1) -> Result<()> {
    let sidecars = contract.sidecars.iter().map(|sidecar| sidecar.role).collect::<BTreeSet<_>>();
    if sidecars.len() != contract.sidecars.len() {
        return Err(BijuxError::validation(format!(
            "artifact contract {} contains duplicate sidecar roles",
            contract.contract_id
        )));
    }
    let required_index = contract.sidecars.iter().any(|sidecar| {
        sidecar.role == ArtifactSidecarRoleV1::Index
            && sidecar.presence == ArtifactPresenceV1::Required
    });
    if contract.index == ArtifactPresenceV1::Required && !required_index {
        return Err(BijuxError::validation(format!(
            "artifact contract {} requires an index but not an index sidecar",
            contract.contract_id
        )));
    }
    if contract.index == ArtifactPresenceV1::Required
        && !contract.validation_invariants.contains(&ArtifactValidationInvariantV1::IndexReadable)
    {
        return Err(BijuxError::validation(format!(
            "artifact contract {} requires an index without its validation invariant",
            contract.contract_id
        )));
    }
    let valid_lineage = match contract.lineage.presence {
        ArtifactPresenceV1::Required => contract.lineage.minimum_parent_count > 0,
        ArtifactPresenceV1::Forbidden | ArtifactPresenceV1::Optional => {
            contract.lineage.minimum_parent_count == 0
        }
    };
    if !valid_lineage {
        return Err(BijuxError::validation(format!(
            "artifact contract {} has inconsistent lineage presence and parent count",
            contract.contract_id
        )));
    }
    if contract.lineage.presence == ArtifactPresenceV1::Required
        && !contract.validation_invariants.contains(&ArtifactValidationInvariantV1::LineageRecorded)
    {
        return Err(BijuxError::validation(format!(
            "artifact contract {} requires lineage without its validation invariant",
            contract.contract_id
        )));
    }
    Ok(())
}

fn check(dimension: &str, compatible: bool, detail: String) -> ArtifactCompatibilityCheckV1 {
    ArtifactCompatibilityCheckV1 { dimension: dimension.to_string(), compatible, detail }
}

fn set_check<T>(dimension: &str, producer: &[T], consumer: &[T]) -> ArtifactCompatibilityCheckV1
where
    T: std::fmt::Debug + Ord,
{
    let producer_set = producer.iter().collect::<BTreeSet<_>>();
    let consumer_set = consumer.iter().collect::<BTreeSet<_>>();
    check(
        dimension,
        producer_set.is_subset(&consumer_set),
        format!("producer={producer:?}, consumer={consumer:?}"),
    )
}

fn guarantee_set_check<T>(
    dimension: &str,
    producer: &[T],
    consumer_required: &[T],
) -> ArtifactCompatibilityCheckV1
where
    T: std::fmt::Debug + Ord,
{
    let guarantees = producer.iter().collect::<BTreeSet<_>>();
    let requirements = consumer_required.iter().collect::<BTreeSet<_>>();
    check(
        dimension,
        requirements.is_subset(&guarantees),
        format!("producer_guarantees={producer:?}, consumer_requires={consumer_required:?}"),
    )
}

fn presence_check(
    dimension: &str,
    producer: ArtifactPresenceV1,
    consumer: ArtifactPresenceV1,
) -> ArtifactCompatibilityCheckV1 {
    let compatible = match consumer {
        ArtifactPresenceV1::Forbidden => producer == ArtifactPresenceV1::Forbidden,
        ArtifactPresenceV1::Optional => true,
        ArtifactPresenceV1::Required => producer == ArtifactPresenceV1::Required,
    };
    check(dimension, compatible, format!("producer={producer:?}, consumer={consumer:?}"))
}

fn sidecar_check(
    producer: &ArtifactContractV1,
    consumer: &ArtifactContractV1,
) -> ArtifactCompatibilityCheckV1 {
    let producer_sidecars =
        producer.sidecars.iter().map(|sidecar| (sidecar.role, sidecar)).collect::<BTreeMap<_, _>>();
    let compatible = consumer.sidecars.iter().all(|required| {
        let Some(provided) = producer_sidecars.get(&required.role) else {
            return required.presence != ArtifactPresenceV1::Required;
        };
        provided.format == required.format
            && match required.presence {
                ArtifactPresenceV1::Forbidden => provided.presence == ArtifactPresenceV1::Forbidden,
                ArtifactPresenceV1::Optional => true,
                ArtifactPresenceV1::Required => provided.presence == ArtifactPresenceV1::Required,
            }
    });
    check(
        "sidecars",
        compatible,
        format!("producer={:?}, consumer={:?}", producer.sidecars, consumer.sidecars),
    )
}
