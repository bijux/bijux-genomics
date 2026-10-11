# VCF adapter interface

`inputs::VcfAdapterInputs::new` admits explicit catalog and binding rows, rejects
unknown stages and repeated or blank binding identities, and takes a typed
`ReferenceFastaMaterializer`. The constructor performs no filesystem effects.
Callers may borrow the admitted catalog and bindings across adapter families.

`inputs::project_supported_vcf_bindings` projects caller-provided stage/tool pairs
onto stage templates. It rejects missing or duplicated templates and repeated
stage/tool/corpus/asset bindings. It neither reads registries nor infers tool scope.

Each family under `adapters` exports typed artifacts, command steps, rows and
reports. Collection services accept admitted inputs and an explicit repository
root; report publication also accepts an output path. Persisted schema identities
remain family-specific. Errors propagate admission, source and publication failures.

`fixtures` exposes local manifest and panel preparation used by those contracts.
Reference writing is delegated through the caller-supplied operation. Native VCF
indexing delegates to the stage execution owner. Panel metadata markers are
explicitly synthetic and are not interchangeable native reference panels.

## Tool-serving evidence

`evidence::serving::collect_vcf_tool_serving_map_rows(&VcfAdapterInputs)` returns
validated serving observations without filesystem effects.
`render_vcf_tool_serving_map(&VcfAdapterInputs, &Path, &Path)` publishes the existing
TSV and returns the compatible row counts and status report. Admission and projection
errors occur before output creation. Supported parser and governed binding semantics
remain unchanged.

## Request observations and commands

`adapters::observations::VcfAdapterObservations` borrows one admitted input snapshot
and repository context. Construction performs no effects; requesting a family/tool
acquires its validated report once. Reports remain immutable and publication clones
only the report to set destination metadata. A prepared report's output path is a
planned destination, not evidence that a file was written. Publication uses the
infrastructure atomic writer. Failed acquisition is not cached; failed publication
does not discard successfully acquired observations.

`evidence::commands::collect_vcf_adapter_command_row_map` consumes these observations,
publishes compatible family reports at their governed command-report paths, and
projects structured argv, pipeline steps and script commands. The descent adapter
remains the command authority for PLINK2 ROH. Other duplicate command authorities
are rejected. Repeated consumers in the same request reuse acquired inputs; there
is no global cache or cross-request filesystem freshness claim. This API establishes
command contracts and fixture observations, not execution of every external tool.

## Expected result declarations

`evidence::expectations` consumes the same request observation context to validate
stage output IDs, required metrics, corpus/assets and report placement. The context
supplies the admitted catalog and bindings, preventing a caller from mixing independent
metadata and acquisition snapshots. Repeated adapter reads reuse that request's acquired
reports. Unsupported duplicate output authorities fail; PLINK2 ROH explicitly belongs
to descent and Beagle imputation outputs to the imputation family.

Collection returns the existing typed expected-result rows. Publication writes the
compatible TSV atomically only after projection and governed requirements pass. These
are expected result declarations and fixture observations; external execution evidence
and scientific acceptance remain separate responsibilities.

## Declared output coverage

`evidence::output_coverage` consumes the request observation context and an explicit
run identity. Typed borrowed adapter views project raw, normalized and index output
declarations without serialization or string-based field discovery. Missing required
outputs remain incomplete; blank or duplicate artifact declarations are rejected.
The existing canonical ROH and Beagle ownership rules resolve overlapping families.

Compatible family reports are published at their governed paths. The coverage TSV is
published atomically after coverage validation. Stream and stage-result paths are
expected paths under the supplied run identity. Coverage establishes declarations;
it does not establish file existence, valid native payloads or scientific substitution.

## Report placement

`evidence::reports` projects expected results against explicitly admitted inputs.
Admission checks the complete set of stage/tool/corpus/asset bindings and their
outputs, stage metrics and catalog sections. Omitted, repeated or foreign bindings
fail before report publication. The owner retains the governed sections, summary
tables and failure columns; the CLI supplies requests and renders the response.

The pure projection performs no filesystem acquisition. The publication entry point
acquires expected observations using one request context and atomically writes the
compatible TSV after all placement contracts pass. Report declarations remain
separate from native payload validation and scientific acceptance.

## Retained scope

`evidence::scope` receives admitted VCF inputs, explicit active binding identities,
expected results and the registry root. The caller owns acquisition of all-domain
active observations. Scope admission preserves full stage/tool/corpus/asset identity
before projecting the stable pair-oriented report. Ambiguous pairs and foreign or
repeated active/expected identities are rejected. Registry decoding rejects malformed
stage identifiers instead of omitting them from the retained set.

The owner preserves active, complete and removed classifications and their governed
proof paths. TSV publication is atomic after projection and scope contracts pass.
Scope membership is distinct from native tool execution or scientific acceptance.

## Parser fixture coverage

`evidence::parsers` observes fixtures for explicit inputs and borrowed retained
bindings. Identity, adapter, parser and normalized stage schema metadata are checked
before filesystem acquisition. The fixture parser and fixture schema have their own
namespaces; they are not substitutes for the normalized stage schema identifiers.

Coverage requires governed raw files and an expected JSON envelope with matching
schema/stage/tool/parser identities and an object-valued normalized payload. This
checks the envelope, not scientific payload semantics or native parser execution.
Fixture discovery does not follow symbolic links. Existing domain parser fixture
bank tests remain responsible for exercising the normalizers.

Known missing or invalid fixtures produce an atomic diagnostic TSV and an incomplete
coverage error. Malformed requests and traversal errors leave an existing report
unchanged. The caller owns acquisition of active observations and CLI presentation.

## PCA readiness evidence

PCA command and declared output projection lives in the VCF adapter evidence owner;
scientific assessment and report publication live in the population readiness owner.
`PcaAdapterEvidence` carries a full stage/tool/corpus/asset binding, adapter/parser/schema
identity, structured command and declared output addresses through the independent
VCF evidence crate. Publication addresses are not executed scientific proof.

Population readiness checks immutable snapshot agreement before readiness locks and
native smoke execution. Adapter acquisition rejects ambiguous retained declarations
before materialization. The CLI shares one admitted adapter observation context across
common evidence and PCA projection. Script and argv publication retain their existing
paths and representations; both serialize before either file is written.

## Registry coverage audits

`evidence::registry::{matrix, orphans, undercoverage}` owns explicit-input registry
coverage reports. One typed snapshot merges the two authored registry documents,
retaining status unions and production stage membership. Conflicting binaries,
malformed arrays, blank or padded identities and repeated entries within a document
are refused before publication. Repeated tools across the two documents remain valid
when their binary declarations agree.

The matrix audit reports declaration drift, including unregistered pairs. Orphan
and undercoverage audits retain their strict coverage contracts. Each acquires its
sources once, reuses one serving report and atomically publishes compatible JSON or
TSV. These declaration audits do not establish scientific execution compatibility.

## Retained command publication

`evidence::commands` admits retained stage/tool/corpus/asset/adapter/parser/schema
identities before adapter acquisition, projects the active command set and publishes
compatible shell and structured argv artifacts. The request root must match the
observation context. Both outputs are serialized before either file is written;
each file is replaced atomically. This is not a transaction across both files.

The CLI supplies retained scope and presents the report. Existing command schema,
ordering, escaping, script structure and companion-path rules remain compatible.

Scripts written beneath the repository without parent traversal retain their
relative root and move with that repository. Scripts written elsewhere anchor to
the canonical repository path at publication time. Relative and absolute repository
literals escape shell metacharacters; path contents cannot expand into commands.
An external script must be regenerated if its repository moves. Paths must be
representable as UTF-8 when embedded into the script.

The `evidence::parsers::failure` service owns the seven governed malformed-output probes, fixture snapshots, domain-parser dispatch and compatible JSON report. It admits every source fixture and known publication destination before creating probe artifacts, refuses symbolic links and special files, and preserves unrelated evidence. Missing-index publication removes only that declared probe file. The CLI only chooses the output and presents the result. This is captured-fixture parser rejection evidence; it does not certify fresh native-tool execution. Publication is atomic per file, without filesystem locking or a multi-artifact transaction.

The required-input audit lives under `evidence::inputs` and receives admitted VCF adapter inputs. All seven adapter families share required-file admission; malformed identities, duplicate artifacts, absent files and directories are rejected. The descent missing-input probe now observes the validator response to the mutated path. These checks prove file presence requirements, not reference/sample/coordinate compatibility or fresh tool execution. Adapter-specific semantic contracts remain independently required.

The `evidence::results::missing` service owns synthetic missing-result recovery over admitted expected bindings. It validates the full ready binding set, safe artifact identities and destination ancestry before fixture effects, preserves unrelated evidence, and removes only the governed absent manifest. Original fake runtime/resource labels remain explicit. It does not certify execution or payload semantics. Writes are atomic per file; concurrent namespace changes and multi-artifact transactions are not claimed.

`evidence::metrics` owns comparable metric acquisition, projection, validation and publication. Domain VCF defines metric units and directions; benchmark model retains its report schema. `evidence::parsers::readiness` composes the fifteen independent checks over one request-scoped observation set. Its injected active-binding provider supplies independent full binding identities at the coverage check; failure is captured without skipping subsequent checks. The complete failed report is published before returning an error. Fixture declarations and synthetic failure cases remain local evidence, not fresh native execution.

`evidence::adapters` owns the fifteen adapter-readiness checks with explicit active-binding and shell-syntax observation ports. The VCF owner has no application API dependency. Shared adapter observations feed publication, output coverage and required-input probes; failed checks remain visible in a complete persisted report. Shell syntax success and argv row counts retain their narrow meaning and do not claim biological compatibility or successful tool execution.

`evidence::smoke` admits complete retained binding/default declarations and projects available host, container and external delivery paths. Host support is injected by composition and observed once per stage; wrapper availability remains distinct from execution evidence. The shared typed VCF registry also admits optional smoke metadata and rejects conflicting declarations. Reports publish atomically after all scope and wrapper contracts pass.

Retained completion owns all24 governed acceptance checks, typed persisted report admission, external execution blockers and complete diagnostic publication under `evidence::completion`. Explicit typed publication and scientific execution ports keep concrete service dependencies in composition; the owner never changes the process working directory. Ports may publish intermediate evidence; final publication is atomic per file and does not imply a multi-file transaction.

`evidence::readiness` acquires and admits complete stage evidence independently of CLI code. A request owns its `VcfReadinessEvidenceSession`; the first successful observation is borrowed by subsequent scientific stage renders. Failures remain retryable observations, and sessions never share a global cache or mutate the working directory. Reusing an observation does not claim that files cannot change concurrently.
