#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import re
from dataclasses import dataclass
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib


EXECUTION_SUPPORT = Path("domain/fastq/execution_support.yaml")
STAGE_DIR = Path("domain/fastq/stages")
TOOL_DIR = Path("domain/fastq/tools")
REGISTRY = Path("configs/ci/registry/tool_registry.toml")
VERSION_LOCK = Path("containers/versions/lock.json")
LICENSE_DIR = Path("containers/licenses")
DOWNLOAD_BACKLOG = Path("science/generated/current/evidence/fastq_download_backlog.tsv")
OUT_DIR = Path("science/docs/upstream/fastq/container")
QA_COVERAGE_BLOCKERS = Path("science/docs/upstream/fastq/QA_COVERAGE_BLOCKERS.tsv")
PROOF_ROOT = Path("artifacts/containers")
PLANNER_SNAPSHOT_DIR = Path("crates/bijux-dna-planner-fastq/tests/snapshots")


@dataclass(frozen=True)
class FastqDefault:
    stage_id: str
    execution_status: str
    default_tool: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate FASTQ container readiness evidence reports."
    )
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--out-dir", type=Path, default=OUT_DIR)
    return parser.parse_args()


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def yaml_scalar(text: str, key: str) -> str:
    match = re.search(rf"(?m)^\s*{re.escape(key)}:\s*['\"]?([^'\"\n]+)['\"]?\s*$", text)
    return match.group(1).strip() if match else ""


def yaml_list(text: str, key: str) -> list[str]:
    lines = text.splitlines()
    values: list[str] = []
    in_block = False
    for line in lines:
        if re.match(rf"^{re.escape(key)}:\s*$", line):
            in_block = True
            continue
        if in_block and re.match(r"^[A-Za-z0-9_]+:", line):
            break
        if in_block:
            match = re.match(r"\s*-\s*['\"]?([^'\"]+)['\"]?\s*$", line)
            if match:
                values.append(match.group(1).strip())
    return values


def yaml_named_items(text: str, key: str) -> list[str]:
    lines = text.splitlines()
    values: list[str] = []
    in_block = False
    for line in lines:
        if re.match(rf"^{re.escape(key)}:\s*$", line):
            in_block = True
            continue
        if in_block and re.match(r"^[A-Za-z0-9_]+:", line):
            break
        if in_block:
            match = re.match(r"\s*-\s*name:\s*['\"]?([^'\"]+)['\"]?\s*$", line)
            if match:
                values.append(match.group(1).strip())
    return values


def execution_defaults(root: Path) -> list[FastqDefault]:
    text = read_text(root / EXECUTION_SUPPORT)
    rows: list[FastqDefault] = []
    current: dict[str, str] = {}
    for line in text.splitlines():
        stage_match = re.match(r'\s*-\s*stage_id:\s*"([^"]+)"', line)
        if stage_match:
            if current.get("stage_id"):
                rows.append(
                    FastqDefault(
                        stage_id=current["stage_id"],
                        execution_status=current.get("execution_status", ""),
                        default_tool=current.get("default_tool", ""),
                    )
                )
            current = {"stage_id": stage_match.group(1)}
            continue
        field_match = re.match(r'\s*(execution_status|default_tool):\s*"([^"]*)"', line)
        if field_match and current:
            current[field_match.group(1)] = field_match.group(2)
    if current.get("stage_id"):
        rows.append(
            FastqDefault(
                stage_id=current["stage_id"],
                execution_status=current.get("execution_status", ""),
                default_tool=current.get("default_tool", ""),
            )
        )
    return rows


def load_stages(root: Path) -> dict[str, dict[str, list[str] | str]]:
    stages = {}
    for path in sorted((root / STAGE_DIR).glob("*.yaml")):
        if path.name == "_schema.yaml":
            continue
        text = read_text(path)
        stage_id = yaml_scalar(text, "stage_id")
        if stage_id:
            stages[stage_id] = {
                "status": yaml_scalar(text, "status"),
                "bank_hooks": yaml_list(text, "bank_hooks"),
                "outputs": yaml_named_items(text, "outputs"),
            }
    return stages


def load_tool_containers(root: Path) -> dict[str, tuple[str, str]]:
    rows = {}
    for path in sorted((root / TOOL_DIR).glob("*.yaml")):
        if path.name == "_schema.yaml":
            continue
        text = read_text(path)
        tool_id = yaml_scalar(text, "tool_id")
        if not tool_id:
            continue
        container_match = re.search(r"(?ms)^container:\n(?P<body>(?:\s+.+\n?)+)", text)
        body = container_match.group("body") if container_match else ""
        rows[tool_id] = (yaml_scalar(body, "image"), yaml_scalar(body, "digest"))
    return rows


def load_registry(root: Path) -> dict[str, dict[str, str]]:
    data = tomllib.loads(read_text(root / REGISTRY))
    rows = {}
    for item in data.get("tools", []):
        if item.get("domain") != "fastq":
            continue
        tool_id = item.get("tool_id") or item.get("id")
        rows[tool_id] = {
            "status": str(item.get("status", "")),
            "container_ref": str(item.get("container_ref", "")),
            "dockerfile": str(item.get("dockerfile", "")),
            "apptainer_def": str(item.get("apptainer_def", "")),
        }
    return rows


def load_download_backlog(root: Path) -> dict[str, dict[str, str]]:
    path = root / DOWNLOAD_BACKLOG
    if not path.exists():
        return {}
    with path.open(encoding="utf-8", newline="") as handle:
        return {
            row["tool_id"]: row
            for row in csv.DictReader(handle, delimiter="\t")
            if row.get("tool_id")
        }


def load_qa_coverage_blockers(root: Path) -> dict[str, list[str]]:
    path = root / QA_COVERAGE_BLOCKERS
    if not path.exists():
        return {}
    blockers: dict[str, list[str]] = {}
    with path.open(encoding="utf-8", newline="") as handle:
        for row in csv.DictReader(handle, delimiter="\t"):
            stage_id = row.get("stage_id", "")
            blocker = row.get("blocker", "")
            if stage_id and blocker:
                blockers.setdefault(stage_id, []).append(blocker)
    return blockers


def load_version_lock(root: Path) -> dict[str, dict[str, object]]:
    path = root / VERSION_LOCK
    if not path.exists():
        return {}
    data = json.loads(read_text(path))
    return {
        str(item.get("tool")): item
        for item in data.get("items", [])
        if item.get("tool")
    }


def load_licenses(root: Path) -> dict[str, dict[str, object]]:
    rows = {}
    for path in sorted((root / LICENSE_DIR).glob("*.license.toml")):
        data = tomllib.loads(read_text(path))
        tool_id = str(data.get("tool_id") or path.name.removesuffix(".license.toml"))
        data["_license_path"] = str(path.relative_to(root))
        rows[tool_id] = data
    return rows


def load_planner_snapshots(root: Path) -> list[dict[str, object]]:
    snapshots = []
    for path in sorted((root / PLANNER_SNAPSHOT_DIR).glob("*stage__fastq__*.json")):
        data = json.loads(read_text(path))
        data["_snapshot_path"] = str(path.relative_to(root))
        snapshots.append(data)
    return snapshots


def write_tsv(path: Path, header: list[str], rows: list[list[str]]) -> None:
    def normalize_row(row: list[str]) -> list[str]:
        normalized = ["" if cell is None else str(cell) for cell in row]
        while normalized and normalized[-1] == "":
            normalized.pop()
        return normalized

    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t", lineterminator="\n")
        writer.writerow(header)
        writer.writerows(normalize_row(row) for row in rows)


def digest_class(container_ref: str) -> str:
    if not container_ref:
        return "missing"
    if "sha256:pending" in container_ref:
        return "pending"
    if "sha256:" + ("0" * 64) in container_ref:
        return "zero_placeholder"
    if "@sha256:" in container_ref:
        return "immutable"
    return "tag_only"


def image_package(ref: str) -> str:
    if not ref:
        return ""
    package = ref.split("@", 1)[0]
    if "/" in package.rsplit("/", 1)[-1]:
        return package
    if ":" in package.rsplit("/", 1)[-1]:
        package = package.rsplit(":", 1)[0]
    return package


def proof_status(root: Path, candidates: list[Path]) -> tuple[str, str]:
    for candidate in candidates:
        path = root / candidate
        if path.exists():
            return "present", str(candidate)
    return "missing_from_snapshot", ";".join(str(candidate) for candidate in candidates)


def proof_candidates(tool_id: str) -> list[tuple[str, list[Path]]]:
    return [
        (
            "docker_cyclonedx_sbom",
            [
                PROOF_ROOT / "sbom" / tool_id / "docker-cyclonedx.json",
                PROOF_ROOT / "sbom" / f"{tool_id}.cyclonedx.json",
            ],
        ),
        (
            "docker_spdx_sbom",
            [
                PROOF_ROOT / "sbom" / tool_id / "docker-spdx.json",
                PROOF_ROOT / "sbom" / f"{tool_id}.spdx.json",
            ],
        ),
        (
            "apptainer_sbom",
            [
                PROOF_ROOT / "sbom" / tool_id / "apptainer-cyclonedx.json",
                PROOF_ROOT / "sbom" / f"{tool_id}.apptainer.cyclonedx.json",
            ],
        ),
        (
            "smoke_manifest",
            [
                PROOF_ROOT / "smoke" / tool_id / "manifest.json",
                PROOF_ROOT / "smoke" / f"{tool_id}.json",
            ],
        ),
    ]


def lock_field_status(value: object) -> str:
    if value in (None, "", 0):
        return "missing"
    if isinstance(value, str) and set(value) == {"0"}:
        return "placeholder_zero"
    return "present"


def planner_status(default_tool: str, planner_tool: str, planner_digest: object) -> str:
    findings = []
    if default_tool and planner_tool and default_tool != planner_tool:
        findings.append("default_tool_mismatch")
    if planner_digest in (None, ""):
        findings.append("digest_missing")
    return ";".join(findings) if findings else "ready"


def license_status(license_row: dict[str, object]) -> str:
    if not license_row:
        return "missing_license_file"
    findings = []
    for field in ["spdx", "upstream_license_id"]:
        value = str(license_row.get(field, ""))
        if not value:
            findings.append(f"{field}_missing")
        elif value == "NOASSERTION":
            findings.append(f"{field}_noassertion")
    if not license_row.get("upstream_url"):
        findings.append("upstream_url_missing")
    if not license_row.get("upstream_checksum"):
        findings.append("upstream_checksum_missing")
    if not license_row.get("redistribution_note"):
        findings.append("redistribution_note_missing")
    return ";".join(findings) if findings else "ready"


def package_status(registry_package: str, domain_package: str) -> str:
    findings = []
    if not registry_package:
        findings.append("registry_package_missing")
    if not domain_package:
        findings.append("domain_package_missing")
    if registry_package and domain_package and registry_package != domain_package:
        findings.append("package_mismatch")
    for label, package in [
        ("registry", registry_package),
        ("domain", domain_package),
    ]:
        if package and not package.startswith("bijuxdna/"):
            findings.append(f"{label}_namespace_review_required")
    return ";".join(findings) if findings else "ready"


def evidence_readiness(evidence: dict[str, str]) -> str:
    archive_status = evidence.get("archive_status", "missing_backlog_row")
    paper_status = evidence.get("paper_status", "missing_backlog_row")
    return "ready" if archive_status == "present" and paper_status else "needs_evidence"


def evidence_kind(evidence: dict[str, str]) -> str:
    citation = evidence.get("citation", "")
    paper_status = evidence.get("paper_status", "")
    if citation.startswith("software:") or paper_status == "software_citation_only":
        return "software_citation"
    if citation.startswith("paper:") or paper_status == "mapped":
        return "paper"
    return "missing_evidence"


def payload_access_status(evidence: dict[str, str]) -> str:
    archive_status = evidence.get("archive_status", "missing_backlog_row")
    paper_status = evidence.get("paper_status", "missing_backlog_row")
    if archive_status == "present" and paper_status in {"mapped", "software_citation_only"}:
        return "ready"
    return f"archive:{archive_status};paper:{paper_status}"


def qa_status(stage_id: str, qa_blockers: dict[str, list[str]]) -> str:
    blockers = qa_blockers.get(stage_id, [])
    return "ready" if not blockers else ";".join(sorted(blockers))


def runtime_surface_status(registry_row: dict[str, str]) -> str:
    findings = []
    if registry_row.get("status") != "production":
        findings.append("registry_not_production")
    if not registry_row.get("container_ref"):
        findings.append("missing_container_ref")
    if not registry_row.get("dockerfile"):
        findings.append("missing_dockerfile")
    if not registry_row.get("apptainer_def"):
        findings.append("missing_apptainer_def")
    return ";".join(findings) if findings else "ready"


def proof_status_by_kind(root: Path, tool_id: str) -> dict[str, str]:
    statuses = {}
    for proof_kind, candidates in proof_candidates(tool_id):
        statuses[proof_kind] = proof_status(root, candidates)[0]
    return statuses


def production_owner(blockers: list[str]) -> str:
    if any(blocker.startswith("payload_access_status:") for blocker in blockers):
        return "bijux-science"
    if any(blocker.startswith("reference_asset_status:") for blocker in blockers):
        return "bijux-domain"
    if any(blocker.startswith("license_status:") for blocker in blockers):
        return "bijux-runtime"
    if any(blocker.startswith("planner_digest_status:") for blocker in blockers):
        return "bijux-planner"
    if any(blocker.startswith("behavioral_qa_status:") for blocker in blockers):
        return "bijux-environment-qa"
    return "bijux-runtime"


def main() -> int:
    args = parse_args()
    root = args.repo_root.resolve()
    out_dir = args.out_dir if args.out_dir.is_absolute() else root / args.out_dir
    stages = load_stages(root)
    tools = load_tool_containers(root)
    registry = load_registry(root)
    backlog = load_download_backlog(root)
    qa_blockers = load_qa_coverage_blockers(root)
    version_lock = load_version_lock(root)
    licenses = load_licenses(root)
    planner_snapshots = load_planner_snapshots(root)
    default_by_stage = {
        row.stage_id: row.default_tool for row in execution_defaults(root) if row.stage_id
    }

    matrix = []
    digest_rows = []
    stages_by_tool: dict[str, list[str]] = {}
    for row in execution_defaults(root):
        tool_image, tool_digest = tools.get(row.default_tool, ("", ""))
        registry_row = registry.get(row.default_tool, {})
        container_ref = registry_row.get("container_ref", "")
        matrix.append(
            [
                row.stage_id,
                str(stages.get(row.stage_id, {}).get("status", "")),
                row.execution_status,
                row.default_tool,
                registry_row.get("status", ""),
                container_ref,
                registry_row.get("dockerfile", ""),
                registry_row.get("apptainer_def", ""),
                tool_image,
                tool_digest,
            ]
        )
        if row.default_tool:
            stages_by_tool.setdefault(row.default_tool, []).append(row.stage_id)
            digest_rows.append(
                [
                    row.stage_id,
                    row.default_tool,
                    container_ref,
                    digest_class(container_ref),
                ]
            )

    write_tsv(
        out_dir / "FASTQ_CONTAINER_DEFAULT_MATRIX.tsv",
        [
            "stage_id",
            "stage_status",
            "execution_status",
            "default_tool",
            "registry_status",
            "container_ref",
            "dockerfile",
            "apptainer_def",
            "domain_container_image",
            "domain_container_digest",
        ],
        matrix,
    )
    write_tsv(
        out_dir / "FASTQ_CONTAINER_DIGEST_CLASSES.tsv",
        ["stage_id", "default_tool", "container_ref", "digest_class"],
        digest_rows,
    )

    producers: dict[str, list[str]] = {}
    for stage_id, stage in stages.items():
        for output in stage.get("outputs", []):
            producers.setdefault(str(output), []).append(stage_id)
    asset_rows = []
    for stage_id, stage in sorted(stages.items()):
        for hook in stage.get("bank_hooks", []):
            hook = str(hook)
            if hook == "none":
                continue
            producer_stages = producers.get(hook, [])
            asset_rows.append(
                [
                    stage_id,
                    str(stage.get("status", "")),
                    hook,
                    ";".join(producer_stages) if producer_stages else "external_or_unproduced",
                    "tracked" if producer_stages else "needs_asset_authority",
                ]
            )
    write_tsv(
        out_dir / "FASTQ_CONTAINER_ASSET_HOOKS.tsv",
        ["stage_id", "stage_status", "asset_hook", "producer_stages", "readiness"],
        asset_rows,
    )
    evidence_rows = []
    seen_tools = sorted({row.default_tool for row in execution_defaults(root) if row.default_tool})
    for tool_id in seen_tools:
        evidence = backlog.get(tool_id, {})
        archive_status = evidence.get("archive_status", "missing_backlog_row")
        paper_status = evidence.get("paper_status", "missing_backlog_row")
        evidence_rows.append(
            [
                tool_id,
                evidence.get("archive_path", ""),
                archive_status,
                evidence.get("paper_root", ""),
                paper_status,
                evidence.get("citation", ""),
                evidence_readiness(evidence),
            ]
        )
    write_tsv(
        out_dir / "FASTQ_CONTAINER_EVIDENCE_STATUS.tsv",
        [
            "default_tool",
            "archive_path",
            "archive_status",
            "paper_root",
            "paper_status",
            "citation",
            "readiness",
        ],
        evidence_rows,
    )
    proof_rows = []
    for row in execution_defaults(root):
        if not row.default_tool:
            continue
        for proof_kind, candidates in proof_candidates(row.default_tool):
            status, paths = proof_status(root, candidates)
            proof_rows.append(
                [
                    row.stage_id,
                    row.default_tool,
                    proof_kind,
                    status,
                    paths,
                ]
            )
    write_tsv(
        out_dir / "FASTQ_CONTAINER_PROOF_GAPS.tsv",
        ["stage_id", "default_tool", "proof_kind", "proof_status", "expected_artifact_paths"],
        proof_rows,
    )
    lock_rows = []
    lock_fields = [
        "resolved_image_digest",
        "image_size_bytes",
        "resolved_sif_sha256",
        "frontend_resolved_sif_sha256",
        "source_sha256",
        "pinned_commit",
        "frontend_smoke_version_output_sha256",
    ]
    for tool_id in seen_tools:
        lock_item = version_lock.get(tool_id, {})
        for field in lock_fields:
            value = lock_item.get(field, "")
            lock_rows.append(
                [
                    tool_id,
                    field,
                    str(value),
                    lock_field_status(value),
                    str(lock_item.get("version", "")),
                    str(lock_item.get("status", "")),
                ]
            )
    write_tsv(
        out_dir / "FASTQ_CONTAINER_LOCK_GAPS.tsv",
        ["default_tool", "lock_field", "lock_value", "field_status", "version", "lock_status"],
        lock_rows,
    )
    license_rows = []
    for tool_id in seen_tools:
        license_row = licenses.get(tool_id, {})
        license_rows.append(
            [
                tool_id,
                str(license_row.get("_license_path", "")),
                str(license_row.get("spdx", "")),
                str(license_row.get("upstream_license_id", "")),
                str(license_row.get("upstream_url", "")),
                str(license_row.get("upstream_version", "")),
                str(license_row.get("upstream_checksum", "")),
                str(license_row.get("redistribution_note", "")),
                license_status(license_row),
            ]
        )
    write_tsv(
        out_dir / "FASTQ_CONTAINER_LICENSE_GAPS.tsv",
        [
            "default_tool",
            "license_path",
            "spdx",
            "upstream_license_id",
            "upstream_url",
            "upstream_version",
            "upstream_checksum",
            "redistribution_note",
            "license_status",
        ],
        license_rows,
    )
    package_rows = []
    for tool_id in seen_tools:
        tool_image, _tool_digest = tools.get(tool_id, ("", ""))
        registry_ref = registry.get(tool_id, {}).get("container_ref", "")
        registry_package = image_package(registry_ref)
        domain_package = image_package(tool_image)
        package_rows.append(
            [
                tool_id,
                ";".join(stages_by_tool.get(tool_id, [])),
                registry_ref,
                registry_package,
                tool_image,
                domain_package,
                digest_class(registry_ref),
                package_status(registry_package, domain_package),
            ]
        )
    write_tsv(
        out_dir / "FASTQ_CONTAINER_PACKAGE_PARITY.tsv",
        [
            "default_tool",
            "stage_ids",
            "registry_container_ref",
            "registry_package",
            "domain_container_image",
            "domain_package",
            "digest_class",
            "package_status",
        ],
        package_rows,
    )
    planner_rows = []
    for snapshot in planner_snapshots:
        stage_id = str(snapshot.get("stage_id", ""))
        image = snapshot.get("image", {})
        if not isinstance(image, dict):
            image = {}
        planner_digest = image.get("digest")
        planner_tool = str(snapshot.get("tool_id", ""))
        default_tool = default_by_stage.get(stage_id, "")
        planner_rows.append(
            [
                stage_id,
                default_tool,
                planner_tool,
                str(image.get("image", "")),
                "" if planner_digest is None else str(planner_digest),
                planner_status(default_tool, planner_tool, planner_digest),
                str(snapshot.get("_snapshot_path", "")),
            ]
        )
    write_tsv(
        out_dir / "FASTQ_CONTAINER_PLANNER_GAPS.tsv",
        [
            "stage_id",
            "execution_default_tool",
            "planner_tool",
            "planner_image",
            "planner_digest",
            "planner_status",
            "snapshot_path",
        ],
        planner_rows,
    )
    planner_status_by_stage = {row[0]: row[5] for row in planner_rows}
    closure_rows = []
    production_rows = []
    for row in execution_defaults(root):
        blockers = []
        stage = stages.get(row.stage_id, {})
        if not row.default_tool:
            status = "declared_only"
            blockers.append("no_default_tool")
            closure_rows.append(
                [
                    row.stage_id,
                    row.execution_status,
                    row.default_tool,
                    status,
                    "",
                    "",
                    "",
                    "",
                    "",
                    "",
                    "",
                    "",
                    ";".join(blockers),
                ]
            )
            continue
        tool_image, _tool_digest = tools.get(row.default_tool, ("", ""))
        registry_ref = registry.get(row.default_tool, {}).get("container_ref", "")
        digest = digest_class(registry_ref)
        if digest != "immutable":
            blockers.append(f"digest_{digest}")
        evidence_status = evidence_readiness(backlog.get(row.default_tool, {}))
        if evidence_status != "ready":
            blockers.append(evidence_status)
        license_finding = license_status(licenses.get(row.default_tool, {}))
        if license_finding != "ready":
            blockers.append("license:" + license_finding)
        registry_package = image_package(registry_ref)
        domain_package = image_package(tool_image)
        package_finding = package_status(registry_package, domain_package)
        if package_finding != "ready":
            blockers.append("package:" + package_finding)
        planner_finding = planner_status_by_stage.get(row.stage_id, "missing_planner_snapshot")
        if planner_finding != "ready":
            blockers.append("planner:" + planner_finding)
        lock_item = version_lock.get(row.default_tool, {})
        lock_blockers = [
            f"{field}:{lock_field_status(lock_item.get(field, ''))}"
            for field in lock_fields
            if lock_field_status(lock_item.get(field, "")) != "present"
        ]
        blockers.extend("lock:" + item for item in lock_blockers)
        proof_blockers = [
            proof_kind
            for proof_kind, candidates in proof_candidates(row.default_tool)
            if proof_status(root, candidates)[0] != "present"
        ]
        blockers.extend("proof:" + item for item in proof_blockers)
        asset_blockers = []
        for hook in stage.get("bank_hooks", []):
            hook = str(hook)
            if hook != "none" and hook not in producers:
                asset_blockers.append(hook)
        blockers.extend("asset:" + item for item in asset_blockers)
        evidence = backlog.get(row.default_tool, {})
        lock_item = version_lock.get(row.default_tool, {})
        registry_row = registry.get(row.default_tool, {})
        proof_statuses = proof_status_by_kind(root, row.default_tool)
        reference_asset_status = ";".join(asset_blockers) if asset_blockers else "ready"
        container_ref_status = digest
        runtime_status = runtime_surface_status(registry_row)
        planner_digest = planner_finding
        sbom_status = ";".join(
            sorted(
                f"{kind}:{status}"
                for kind, status in proof_statuses.items()
                if kind.endswith("_sbom") and status != "present"
            )
        ) or "ready"
        smoke_status = proof_statuses.get("smoke_manifest", "missing_from_snapshot")
        behavioral_status = qa_status(row.stage_id, qa_blockers)
        production_blockers = []
        for field, value in [
            ("payload_access_status", payload_access_status(evidence)),
            ("reference_asset_status", reference_asset_status),
            ("container_ref_status", container_ref_status),
            ("license_status", license_finding),
            ("runtime_surface_status", runtime_status),
            ("planner_digest_status", planner_digest),
            ("sbom_status", sbom_status),
            ("smoke_status", smoke_status),
            ("behavioral_qa_status", behavioral_status),
            ("registry_status", registry_row.get("status", "missing_registry_row")),
        ]:
            if value not in {"ready", "immutable", "production"}:
                production_blockers.append(f"{field}:{value}")
        resolved_image_digest = str(lock_item.get("resolved_image_digest", ""))
        resolved_sif_sha256 = str(lock_item.get("resolved_sif_sha256", ""))
        if lock_field_status(resolved_image_digest) != "present":
            production_blockers.append("resolved_image_digest:missing")
        if lock_field_status(resolved_sif_sha256) != "present":
            production_blockers.append("resolved_sif_sha256:missing")
        production_rows.append(
            [
                row.stage_id,
                row.default_tool,
                evidence_kind(evidence),
                evidence.get("locator", ""),
                evidence.get("citation", ""),
                evidence.get("archive_path", ""),
                payload_access_status(evidence),
                reference_asset_status,
                container_ref_status,
                resolved_image_digest,
                resolved_sif_sha256,
                license_finding,
                runtime_status,
                planner_digest,
                sbom_status,
                smoke_status,
                behavioral_status,
                registry_row.get("status", "missing_registry_row"),
                "closed" if not production_blockers else "blocked",
                ";".join(production_blockers),
                production_owner(production_blockers),
                "not_recorded",
            ]
        )
        closure_rows.append(
            [
                row.stage_id,
                row.execution_status,
                row.default_tool,
                "ready" if not blockers else "blocked",
                digest,
                evidence_status,
                license_finding,
                package_finding,
                planner_finding,
                ";".join(lock_blockers),
                ";".join(proof_blockers),
                ";".join(asset_blockers),
                ";".join(blockers),
            ]
        )
    write_tsv(
        out_dir / "FASTQ_CONTAINER_CLOSURE_SUMMARY.tsv",
        [
            "stage_id",
            "execution_status",
            "default_tool",
            "closure_status",
            "digest_class",
            "evidence_readiness",
            "license_status",
            "package_status",
            "planner_status",
            "lock_blockers",
            "proof_blockers",
            "asset_blockers",
            "blockers",
        ],
        closure_rows,
    )
    write_tsv(
        out_dir / "FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv",
        [
            "stage_id",
            "tool_id",
            "evidence_kind",
            "primary_locator",
            "supporting_locators",
            "local_payload_path",
            "payload_access_status",
            "reference_asset_status",
            "container_ref_status",
            "resolved_image_digest",
            "resolved_sif_sha256",
            "license_status",
            "runtime_surface_status",
            "planner_digest_status",
            "sbom_status",
            "smoke_status",
            "behavioral_qa_status",
            "registry_status",
            "closure_status",
            "blocking_reason",
            "owner",
            "last_verified_utc",
        ],
        production_rows,
    )
    print(
        json.dumps(
            {
                "written": [
                    str(out_dir / "FASTQ_CONTAINER_DEFAULT_MATRIX.tsv"),
                    str(out_dir / "FASTQ_CONTAINER_DIGEST_CLASSES.tsv"),
                    str(out_dir / "FASTQ_CONTAINER_ASSET_HOOKS.tsv"),
                    str(out_dir / "FASTQ_CONTAINER_EVIDENCE_STATUS.tsv"),
                    str(out_dir / "FASTQ_CONTAINER_PROOF_GAPS.tsv"),
                    str(out_dir / "FASTQ_CONTAINER_LOCK_GAPS.tsv"),
                    str(out_dir / "FASTQ_CONTAINER_LICENSE_GAPS.tsv"),
                    str(out_dir / "FASTQ_CONTAINER_PACKAGE_PARITY.tsv"),
                    str(out_dir / "FASTQ_CONTAINER_PLANNER_GAPS.tsv"),
                    str(out_dir / "FASTQ_CONTAINER_CLOSURE_SUMMARY.tsv"),
                    str(out_dir / "FASTQ_PRODUCTION_CLOSURE_LEDGER.tsv"),
                ]
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
