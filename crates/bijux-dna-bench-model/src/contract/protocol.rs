//! Validation for permanent benchmark run protocols and executable cases.

use std::collections::BTreeSet;

use bijux_dna_core::contract::canonical::to_canonical_json_bytes;
use sha2::{Digest, Sha256};

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkCaseDefinition, BenchmarkContractLock, BenchmarkCorpusRole, BenchmarkRunDefinition,
    BenchmarkRunState,
};

use super::{
    benchmark_run_directory_name, BENCHMARK_CASE_DEFINITION_SCHEMA_V1,
    BENCHMARK_RUN_DEFINITION_SCHEMA_V1,
};

/// Validate one permanent benchmark run protocol.
///
/// # Errors
/// Returns an error when identity, state, tools, source locks, corpus roles, or citation metadata
/// are incomplete or inconsistent.
pub fn validate_benchmark_run_definition(
    definition: &BenchmarkRunDefinition,
) -> Result<(), BenchError> {
    if definition.schema_version != BENCHMARK_RUN_DEFINITION_SCHEMA_V1 {
        return Err(invalid("benchmark run-definition schema mismatch"));
    }
    benchmark_run_directory_name(&definition.identity)?;
    validate_protocol_version(&definition.protocol_version)?;
    for (value, field) in [
        (&definition.title, "title"),
        (&definition.purpose, "purpose"),
        (&definition.owner, "owner"),
        (&definition.acceptance_policy_id, "acceptance_policy_id"),
        (&definition.citation_key, "citation_key"),
    ] {
        require_text(value, field)?;
    }
    if definition.owner != "bijux-dna-bench" {
        return Err(invalid("benchmark run definition has a noncanonical owner"));
    }
    validate_component(&definition.acceptance_policy_id, "acceptance_policy_id")?;
    validate_component(&definition.citation_key, "citation_key")?;
    match (definition.state, definition.state_reason.as_deref()) {
        (BenchmarkRunState::Active, None) => {}
        (BenchmarkRunState::Active, Some(_)) => {
            return Err(invalid("active benchmark run definition carries a state reason"));
        }
        (BenchmarkRunState::Reserved | BenchmarkRunState::Retired, Some(reason))
            if reason.trim().len() >= 20 => {}
        (BenchmarkRunState::Reserved | BenchmarkRunState::Retired, _) => {
            return Err(invalid(
                "reserved or retired benchmark run definition requires a substantive reason",
            ));
        }
    }
    validate_sorted_components(&definition.tool_ids, "tool_ids", true)?;
    validate_contract_locks(&definition.contracts)?;
    validate_corpus_roles(&definition.corpus_roles)?;
    validate_run_ids(&definition.supersedes_run_ids, &definition.identity.run_id)?;
    if definition.state == BenchmarkRunState::Active
        && (definition.tool_ids.is_empty()
            || definition.contracts.is_empty()
            || definition.corpus_roles.is_empty())
    {
        return Err(invalid(
            "active benchmark run definition requires tools, contracts, and corpus roles",
        ));
    }
    if definition.state == BenchmarkRunState::Reserved
        && (!definition.tool_ids.is_empty()
            || !definition.contracts.is_empty()
            || !definition.corpus_roles.is_empty())
    {
        return Err(invalid("reserved benchmark run definition cannot claim executable closure"));
    }
    Ok(())
}

/// Validate an intentional revision of a permanent benchmark protocol.
///
/// # Errors
/// Returns an error unless a same-run revision increases `protocol_version` or a replacement run
/// explicitly supersedes the prior run while preserving its semantic stage identity.
pub fn validate_benchmark_run_definition_revision(
    previous: &BenchmarkRunDefinition,
    current: &BenchmarkRunDefinition,
) -> Result<(), BenchError> {
    validate_benchmark_run_definition(previous)?;
    validate_benchmark_run_definition(current)?;
    if previous.identity == current.identity {
        if protocol_version(&current.protocol_version)?
            <= protocol_version(&previous.protocol_version)?
        {
            return Err(invalid("same-run protocol revision must increase protocol_version"));
        }
        return Ok(());
    }
    if previous.identity.domain != current.identity.domain
        || previous.identity.stage_id != current.identity.stage_id
        || !current.supersedes_run_ids.contains(&previous.identity.run_id)
    {
        return Err(invalid(
            "replacement protocol must preserve semantic stage identity and explicitly supersede the prior run",
        ));
    }
    Ok(())
}

/// Return the canonical SHA-256 identity of one valid run definition.
///
/// # Errors
/// Returns an error when the definition is invalid or cannot be canonically serialized.
pub fn benchmark_run_definition_sha256(
    definition: &BenchmarkRunDefinition,
) -> Result<String, BenchError> {
    validate_benchmark_run_definition(definition)?;
    let bytes = to_canonical_json_bytes(definition).map_err(|error| {
        invalid(format!("cannot canonicalize benchmark run definition: {error}"))
    })?;
    Ok(sha256_hex(&bytes))
}

/// Validate one source-locked benchmark case.
///
/// # Errors
/// Returns an error when case/run/tool identity, commit and digest locks, inputs, qualification,
/// or repetition policy are malformed.
pub fn validate_benchmark_case_definition(
    definition: &BenchmarkCaseDefinition,
) -> Result<(), BenchError> {
    if definition.schema_version != BENCHMARK_CASE_DEFINITION_SCHEMA_V1 {
        return Err(invalid("benchmark case-definition schema mismatch"));
    }
    benchmark_run_directory_name(&definition.run)?;
    validate_component(&definition.case_id, "case_id")?;
    validate_component(&definition.campaign_id, "campaign_id")?;
    validate_component(&definition.tool_id, "tool_id")?;
    validate_commit(&definition.repository_commit)?;
    for digest in [
        definition.corpus_snapshot_sha256.as_str(),
        definition.parameter_lock_sha256.as_str(),
        definition.execution_spec_sha256.as_str(),
        definition.qualification.receipt_sha256.as_str(),
        definition.qualification.image_sha256.as_str(),
    ] {
        validate_digest(digest)?;
    }
    if definition.qualification.tool_id != definition.tool_id {
        return Err(invalid("benchmark case qualification tool differs from case tool"));
    }
    validate_study_uri(&definition.qualification.uri)?;
    if definition.warmup_repetitions < 1 || definition.measured_repetitions < 3 {
        return Err(invalid("benchmark case repetition policy is not publishable"));
    }
    if definition.inputs.is_empty() {
        return Err(invalid("benchmark case has no immutable input locks"));
    }
    let mut roles = BTreeSet::new();
    let mut assets = BTreeSet::new();
    let mut previous_role = None;
    for input in &definition.inputs {
        validate_component(&input.role, "input role")?;
        validate_component(&input.asset_id, "input asset_id")?;
        validate_digest(&input.sha256)?;
        if previous_role.is_some_and(|role| role >= input.role.as_str())
            || !roles.insert(input.role.as_str())
            || !assets.insert(input.asset_id.as_str())
        {
            return Err(invalid("benchmark case duplicates an input role or asset"));
        }
        previous_role = Some(input.role.as_str());
    }
    Ok(())
}

fn validate_contract_locks(contracts: &[BenchmarkContractLock]) -> Result<(), BenchError> {
    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut previous_id = None;
    for contract in contracts {
        validate_component(&contract.contract_id, "contract_id")?;
        validate_repository_path(&contract.repository_path)?;
        validate_digest(&contract.sha256)?;
        if previous_id.is_some_and(|id| id >= contract.contract_id.as_str())
            || !ids.insert(contract.contract_id.as_str())
            || !paths.insert(contract.repository_path.as_str())
        {
            return Err(invalid("benchmark run definition duplicates a contract id or path"));
        }
        previous_id = Some(contract.contract_id.as_str());
    }
    Ok(())
}

fn validate_corpus_roles(roles: &[BenchmarkCorpusRole]) -> Result<(), BenchError> {
    let mut names = BTreeSet::new();
    let mut corpus_ids = BTreeSet::new();
    let mut previous_role = None;
    for role in roles {
        validate_component(&role.role, "corpus role")?;
        if previous_role.is_some_and(|name| name >= role.role.as_str())
            || !names.insert(role.role.as_str())
        {
            return Err(invalid("benchmark run definition duplicates a corpus role"));
        }
        previous_role = Some(role.role.as_str());
        validate_sorted_components(&role.corpus_ids, "corpus_ids", false)?;
        for corpus_id in &role.corpus_ids {
            if !corpus_ids.insert(corpus_id.as_str()) {
                return Err(invalid("benchmark run definition reuses a corpus across roles"));
            }
        }
    }
    Ok(())
}

fn validate_sorted_components(
    values: &[String],
    field: &str,
    allow_empty: bool,
) -> Result<(), BenchError> {
    if !allow_empty && values.is_empty() {
        return Err(invalid(format!("benchmark {field} cannot be empty")));
    }
    for value in values {
        validate_component(value, field)?;
    }
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(invalid(format!("benchmark {field} must be sorted and unique")));
    }
    Ok(())
}

fn validate_run_ids(run_ids: &[String], current: &str) -> Result<(), BenchError> {
    let mut previous = None;
    for run_id in run_ids {
        let valid = run_id
            .strip_prefix("run-")
            .and_then(|number| number.parse::<u16>().ok())
            .is_some_and(|number| (2000..3000).contains(&number));
        if !valid || run_id == current || previous.is_some_and(|value| value >= run_id.as_str()) {
            return Err(invalid(
                "benchmark superseded run ids must be canonical, sorted, and unique",
            ));
        }
        previous = Some(run_id.as_str());
    }
    Ok(())
}

fn validate_protocol_version(value: &str) -> Result<(), BenchError> {
    protocol_version(value).map(|_| ())
}

fn protocol_version(value: &str) -> Result<(u64, u64, u64), BenchError> {
    let components = value.split('.').collect::<Vec<_>>();
    if components.len() != 3
        || components
            .iter()
            .any(|part| part.is_empty() || (part.len() > 1 && part.starts_with('0')))
    {
        return Err(invalid("benchmark protocol_version must use canonical major.minor.patch"));
    }
    let parsed =
        components.iter().map(|part| part.parse::<u64>()).collect::<Result<Vec<_>, _>>().map_err(
            |_| invalid("benchmark protocol_version must use canonical major.minor.patch"),
        )?;
    Ok((parsed[0], parsed[1], parsed[2]))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}

fn validate_component(value: &str, field: &str) -> Result<(), BenchError> {
    if value.is_empty()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
        || value.starts_with(['-', '_'])
        || value.ends_with(['-', '_'])
    {
        return Err(invalid(format!("benchmark {field} is not a normalized component")));
    }
    Ok(())
}

fn validate_repository_path(path: &str) -> Result<(), BenchError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path.split('/').any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(invalid("benchmark contract path is not repository relative"));
    }
    Ok(())
}

fn validate_study_uri(uri: &str) -> Result<(), BenchError> {
    let Some(relative) = uri.strip_prefix("study://") else {
        return Err(invalid("benchmark qualification receipt must use a study URI"));
    };
    if relative.is_empty()
        || relative.starts_with('/')
        || relative.ends_with('/')
        || relative.contains(['\\', '?', '#', '%'])
        || relative.split('/').any(|part| part.is_empty() || matches!(part, "." | ".."))
    {
        return Err(invalid("benchmark qualification receipt URI is not normalized"));
    }
    Ok(())
}

fn validate_digest(value: &str) -> Result<(), BenchError> {
    if value.len() != 64
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().any(|byte| byte.is_ascii_uppercase())
        || value.bytes().all(|byte| byte == b'0')
    {
        return Err(invalid("benchmark protocol SHA-256 identity is malformed"));
    }
    Ok(())
}

fn validate_commit(value: &str) -> Result<(), BenchError> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_hexdigit())
        || value.bytes().any(|byte| byte.is_ascii_uppercase())
        || value.bytes().all(|byte| byte == b'0')
    {
        return Err(invalid("benchmark case repository commit is malformed"));
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() || value != value.trim() {
        return Err(invalid(format!("benchmark run definition {field} is blank or untrimmed")));
    }
    Ok(())
}

fn invalid(message: impl Into<String>) -> BenchError {
    BenchError::InvalidPolicy(message.into())
}
