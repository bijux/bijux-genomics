#!/usr/bin/env python3
"""Import and validate immutable LUNARC container-run provenance."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import sys
from collections import Counter, defaultdict
from collections.abc import Iterable
from pathlib import Path
from typing import Any

RUN_ID = re.compile(r"run-[1-9][0-9]{3,}")
TRACKED_RUN = re.compile(r"(run-[1-9][0-9]{3,})-([a-z0-9_]+)")
LOCK_SCHEMA = "bijux.container.run_evidence_lock.v1"
INDEX_SCHEMA = "bijux.container.run_evidence_index.v2"
ATTEMPT_LEDGER_SCHEMA = "bijux.container.run_attempt_ledger.v1"
COPIED_EVIDENCE = (
    "config-governance.yml",
    "config-pipeline.yml",
    "config-credential.yml",
    "evidence.json",
    "slurm-accounting.json",
)
SECRET_FIELD_NAMES = {
    "access_key",
    "api_key",
    "credential",
    "password",
    "private_key",
    "secret",
    "token",
    "value",
}


class RunEvidenceError(RuntimeError):
    """Raised when run evidence cannot be admitted without ambiguity."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".partial")
    temporary.write_text(
        json.dumps(value, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    temporary.replace(path)


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise RunEvidenceError(f"cannot read JSON evidence {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise RunEvidenceError(f"JSON evidence must be an object: {path}")
    return value


def yaml_scalars(path: Path) -> dict[tuple[str, ...], str]:
    """Read the scalar mapping subset used by archived run contracts.

    The evidence importer intentionally has no runtime dependency on PyYAML.
    It does not execute or rewrite YAML; it only locates plain mapping keys
    needed to authenticate the archived identity and redaction contract.
    """
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as exc:
        raise RunEvidenceError(f"cannot read YAML evidence {path}: {exc}") from exc
    scalars: dict[tuple[str, ...], str] = {}
    parents: list[tuple[int, str]] = []
    for number, raw_line in enumerate(lines, start=1):
        if not raw_line.strip() or raw_line.lstrip().startswith("#"):
            continue
        if "\t" in raw_line[: len(raw_line) - len(raw_line.lstrip())]:
            raise RunEvidenceError(f"YAML indentation uses a tab at {path}:{number}")
        indentation = len(raw_line) - len(raw_line.lstrip(" "))
        line = raw_line.strip()
        if line.startswith("-"):
            continue
        match = re.fullmatch(r"([A-Za-z0-9_]+):(?:\s*(.*))?", line)
        if match is None:
            continue
        key, raw_value = match.groups()
        while parents and parents[-1][0] >= indentation:
            parents.pop()
        current = tuple(parent for _, parent in parents) + (key,)
        value = (raw_value or "").strip().strip("'\"")
        if value:
            scalars[current] = value
            if value.startswith("{") and value.endswith("}"):
                for child in re.finditer(
                    r"(?:^|,)\s*([A-Za-z0-9_]+):\s*([^,}]+)",
                    value[1:-1],
                ):
                    child_key, child_value = child.groups()
                    scalars[current + (child_key,)] = child_value.strip().strip("'\"")
        else:
            parents.append((indentation, key))
    return scalars


def validate_credential_reference(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    findings = sorted(
        {
            match.group(1).lower()
            for match in re.finditer(r"(?m)^\s*([A-Za-z0-9_]+)\s*:", text)
            if match.group(1).lower() in SECRET_FIELD_NAMES
        }
    )
    if findings:
        raise RunEvidenceError(
            f"credential reference contains forbidden value fields: {findings}"
        )
    scalars = yaml_scalars(path)
    credentials_required = scalars.get(("credentials_required",))
    if credentials_required not in {"true", "false"}:
        raise RunEvidenceError(
            f"credentials_required must be explicit boolean text in {path}"
        )
    references_are_empty = bool(re.search(r"(?m)^references:\s*\[\s*\]\s*$", text))
    references_are_present = bool(re.search(r"(?m)^references:\s*$", text))
    if (credentials_required == "true") != references_are_present:
        raise RunEvidenceError(
            f"credentials_required disagrees with references in {path}"
        )
    if credentials_required == "false" and not references_are_empty:
        raise RunEvidenceError(f"empty credential references must use [] in {path}")
    inline_redaction = bool(
        re.search(r"(?m)^redaction:\s*\{\s*publish_values:\s*false\s*\}\s*$", text)
    )
    if scalars.get(("redaction", "publish_values")) != "false" and not inline_redaction:
        raise RunEvidenceError(
            f"credential reference lacks a no-publication rule: {path}"
        )


def result_directories(result_root: Path) -> Iterable[tuple[str, str, Path]]:
    """Yield historical nested and current flat result directories once."""
    seen: set[Path] = set()
    for tool_root in sorted(result_root.iterdir()):
        if not tool_root.is_dir() or tool_root.name == "python-cache":
            continue
        for candidate in sorted(tool_root.glob("run-*")):
            if candidate.is_dir() and RUN_ID.fullmatch(candidate.name):
                resolved = candidate.resolve()
                if resolved not in seen:
                    seen.add(resolved)
                    yield candidate.name, tool_root.name, candidate
    for candidate in sorted(result_root.glob("run-*-*")):
        match = TRACKED_RUN.fullmatch(candidate.name)
        if candidate.is_dir() and match:
            resolved = candidate.resolve()
            if resolved not in seen:
                seen.add(resolved)
                yield match.group(1), match.group(2), candidate


def file_inventory(result: Path) -> list[dict[str, Any]]:
    records = []
    for path in sorted(result.rglob("*")):
        if not path.is_file():
            continue
        records.append(
            {
                "path": path.relative_to(result).as_posix(),
                "size_bytes": path.stat().st_size,
                "sha256": sha256_file(path),
            }
        )
    return records


def evidence_record(
    run_id: str,
    tool: str,
    result: Path,
    result_root: Path,
) -> dict[str, Any]:
    evidence_path = result / "evidence.json"
    evidence = load_json(evidence_path) if evidence_path.is_file() else {}
    if evidence and evidence.get("run_id") != run_id:
        raise RunEvidenceError(
            f"evidence run identity disagrees with its directory: {evidence_path}"
        )
    pipeline_path = result / "config-pipeline.yml"
    credential_path = result / "config-credential.yml"
    if not pipeline_path.is_file() or not credential_path.is_file():
        raise RunEvidenceError(f"run lacks archived pipeline contracts: {result}")
    validate_credential_reference(credential_path)
    pipeline = yaml_scalars(pipeline_path)
    if pipeline.get(("run", "id")) != run_id or pipeline.get(("run", "tool")) != tool:
        raise RunEvidenceError(
            f"pipeline identity disagrees with result path: {pipeline_path}"
        )
    accounting_path = result / "slurm-accounting.json"
    accounting = load_json(accounting_path) if accounting_path.is_file() else {}
    return {
        "schema_version": LOCK_SCHEMA,
        "run_id": run_id,
        "tool": tool,
        "execution_status": evidence.get("status", "incomplete"),
        "scientific_status": "not_assessed",
        "source_result": (
            Path("results") / "container-validation" / result.relative_to(result_root)
        ).as_posix(),
        "repository_commit": evidence.get(
            "repository_commit", pipeline.get(("repository", "commit"), "")
        ),
        "slurm_job_id": evidence.get("runtime", {}).get(
            "slurm_job_id", accounting.get("slurm_job_id", "")
        ),
        "image": evidence.get("image", pipeline.get("image", {})),
        "failures": evidence.get("failures", ["execution evidence is absent"]),
        "source_files": file_inventory(result),
        "tracked_files": {},
    }


def import_evidence(
    result_root: Path,
    output_root: Path,
    minimum_run: int | None,
    maximum_run: int | None,
) -> int:
    admitted = 0
    for run_id, tool, result in result_directories(result_root.absolute()):
        number = int(run_id.removeprefix("run-"))
        if minimum_run is not None and number < minimum_run:
            continue
        if maximum_run is not None and number > maximum_run:
            continue
        record = evidence_record(run_id, tool, result, result_root.absolute())
        target = output_root / f"{run_id}-{tool}"
        if target.exists():
            raise RunEvidenceError(
                f"refusing to replace tracked run evidence: {target}"
            )
        target.mkdir(parents=True)
        try:
            tracked_files: dict[str, dict[str, Any]] = {}
            for name in COPIED_EVIDENCE:
                source = result / name
                if not source.is_file():
                    continue
                destination = target / name
                shutil.copyfile(source, destination)
                tracked_files[name] = {
                    "size_bytes": destination.stat().st_size,
                    "sha256": sha256_file(destination),
                }
            record["tracked_files"] = tracked_files
            write_json(target / "evidence.lock.json", record)
        except Exception:
            shutil.rmtree(target)
            raise
        admitted += 1
    return admitted


def validate_run(path: Path) -> dict[str, Any]:
    match = TRACKED_RUN.fullmatch(path.name)
    if match is None:
        raise RunEvidenceError(f"tracked run directory has invalid identity: {path}")
    record = load_json(path / "evidence.lock.json")
    if record.get("schema_version") != LOCK_SCHEMA:
        raise RunEvidenceError(f"unsupported evidence lock schema: {path}")
    if record.get("run_id") != match.group(1) or record.get("tool") != match.group(2):
        raise RunEvidenceError(f"tracked directory and evidence lock disagree: {path}")
    repository_commit = record.get("repository_commit")
    if (
        not isinstance(repository_commit, str)
        or re.fullmatch(r"[0-9a-f]{40}", repository_commit) is None
    ):
        raise RunEvidenceError(f"repository commit is not immutable: {path}")
    source_result = record.get("source_result")
    expected_prefix = "results/container-validation/"
    if not isinstance(source_result, str) or not source_result.startswith(
        expected_prefix
    ):
        raise RunEvidenceError(f"source result is outside governed results: {path}")
    source_files = record.get("source_files")
    if not isinstance(source_files, list) or not source_files:
        raise RunEvidenceError(f"source file inventory is absent: {path}")
    source_paths: set[str] = set()
    for source_file in source_files:
        if not isinstance(source_file, dict):
            raise RunEvidenceError(f"source file inventory row is invalid: {path}")
        source_path = source_file.get("path")
        source_size = source_file.get("size_bytes")
        source_digest = source_file.get("sha256")
        if (
            not isinstance(source_path, str)
            or not source_path
            or Path(source_path).is_absolute()
            or ".." in Path(source_path).parts
            or source_path in source_paths
        ):
            raise RunEvidenceError(f"source file inventory path is invalid: {path}")
        if not isinstance(source_size, int) or source_size < 0:
            raise RunEvidenceError(f"source file inventory size is invalid: {path}")
        if (
            not isinstance(source_digest, str)
            or re.fullmatch(r"[0-9a-f]{64}", source_digest) is None
        ):
            raise RunEvidenceError(f"source file inventory digest is invalid: {path}")
        source_paths.add(source_path)
    tracked_files = record.get("tracked_files")
    if not isinstance(tracked_files, dict):
        raise RunEvidenceError(f"tracked_files must be an object: {path}")
    for name, expected in tracked_files.items():
        candidate = path / name
        if not candidate.is_file():
            raise RunEvidenceError(f"tracked evidence is absent: {candidate}")
        if candidate.stat().st_size != expected.get("size_bytes"):
            raise RunEvidenceError(f"tracked evidence size changed: {candidate}")
        if sha256_file(candidate) != expected.get("sha256"):
            raise RunEvidenceError(f"tracked evidence digest changed: {candidate}")
    credential = path / "config-credential.yml"
    if credential.is_file():
        validate_credential_reference(credential)
    return record


def validate_runs(root: Path) -> list[dict[str, Any]]:
    records = []
    identities: set[str] = set()
    for path in sorted(root.glob("run-*-*")):
        if not path.is_dir():
            continue
        if (path / "config-campaign.json").is_file():
            continue
        if not (path / "evidence.lock.json").is_file():
            raise RunEvidenceError(f"run directory lacks governed evidence: {path}")
        record = validate_run(path)
        run_id = str(record["run_id"])
        if run_id in identities:
            raise RunEvidenceError(f"duplicate tracked run identity: {run_id}")
        identities.add(run_id)
        records.append(record)
    return sorted(records, key=lambda record: int(str(record["run_id"])[4:]))


def run_identity(root: Path, record: dict[str, Any]) -> dict[str, str]:
    run_path = root / f"{record['run_id']}-{record['tool']}"
    pipeline_path = run_path / "config-pipeline.yml"
    pipeline = yaml_scalars(pipeline_path)
    identity = {
        "pipeline_schema_version": pipeline.get(("schema_version",), ""),
        "campaign": pipeline.get(("run", "campaign"), ""),
        "corpus": pipeline.get(("run", "corpus"), ""),
        "sample": pipeline.get(("run", "sample"), ""),
        "domain": pipeline.get(("run", "domain"), ""),
        "stage": pipeline.get(("run", "stage"), ""),
    }
    missing = [name for name, value in identity.items() if not value]
    if missing:
        raise RunEvidenceError(
            f"run pipeline lacks qualification identity {missing}: {pipeline_path}"
        )
    return identity


def final_evidence(
    records: list[dict[str, Any]],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    by_tool: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for record in records:
        by_tool[str(record["tool"])].append(record)
    selected = []
    superseded = []
    for tool, tool_records in sorted(by_tool.items()):
        passed = [
            record for record in tool_records if record["execution_status"] == "passed"
        ]
        if not passed:
            raise RunEvidenceError(
                f"{tool}: no successful execution can be published as final evidence"
            )
        final = max(passed, key=lambda record: int(str(record["run_id"])[4:]))
        selected.append(final)
        superseded.extend(record for record in tool_records if record is not final)
    return (
        sorted(selected, key=lambda record: str(record["tool"])),
        sorted(superseded, key=lambda record: int(str(record["run_id"])[4:])),
    )


def validate_final_evidence(records: list[dict[str, Any]]) -> None:
    selected, superseded = final_evidence(records)
    if superseded:
        identities = ", ".join(str(record["run_id"]) for record in superseded)
        raise RunEvidenceError(
            "tracked run surface contains superseded attempts; publish final evidence first: "
            + identities
        )
    if any(record["execution_status"] != "passed" for record in selected):
        raise RunEvidenceError("final evidence contains a non-passing execution")


def validate_attempt_ledger(path: Path, records: list[dict[str, Any]]) -> None:
    ledger = load_json(path)
    if ledger.get("schema_version") != ATTEMPT_LEDGER_SCHEMA:
        raise RunEvidenceError(f"unsupported attempt ledger schema: {path}")
    attempts = ledger.get("attempts")
    if not isinstance(attempts, list):
        raise RunEvidenceError(f"attempt ledger rows are absent: {path}")
    final_by_id = {str(record["run_id"]): record for record in records}
    seen: set[str] = set()
    for attempt in attempts:
        if not isinstance(attempt, dict):
            raise RunEvidenceError(f"attempt ledger row is invalid: {path}")
        run_id = attempt.get("run_id")
        final_id = attempt.get("superseded_by")
        digest = attempt.get("evidence_lock_sha256")
        if not isinstance(run_id, str) or RUN_ID.fullmatch(run_id) is None:
            raise RunEvidenceError(f"attempt ledger run identity is invalid: {path}")
        if run_id in seen or run_id in final_by_id:
            raise RunEvidenceError(
                f"attempt ledger run identity is duplicated: {run_id}"
            )
        seen.add(run_id)
        final = final_by_id.get(str(final_id))
        if final is None or final["tool"] != attempt.get("tool"):
            raise RunEvidenceError(f"attempt ledger replacement is invalid: {run_id}")
        if not isinstance(digest, str) or re.fullmatch(r"[0-9a-f]{64}", digest) is None:
            raise RunEvidenceError(f"attempt ledger lock digest is invalid: {run_id}")
    summary = ledger.get("summary")
    if not isinstance(summary, dict):
        raise RunEvidenceError(f"attempt ledger summary is absent: {path}")
    if summary.get("published_run_count") != len(records):
        raise RunEvidenceError(f"attempt ledger published count drifted: {path}")
    if summary.get("superseded_attempt_count") != len(attempts):
        raise RunEvidenceError(f"attempt ledger attempt count drifted: {path}")


def attempt_ledger_rows(
    root: Path,
    records: list[dict[str, Any]],
    existing_attempts: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    selected, superseded = final_evidence(records)
    final_by_tool = {str(record["tool"]): str(record["run_id"]) for record in selected}
    new_attempts = [
        {
            "run_id": record["run_id"],
            "tool": record["tool"],
            "execution_status": record["execution_status"],
            "repository_commit": record["repository_commit"],
            "slurm_job_id": record["slurm_job_id"],
            "source_result": record["source_result"],
            "source_file_count": len(record["source_files"]),
            "evidence_lock_sha256": sha256_file(
                root / f"{record['run_id']}-{record['tool']}" / "evidence.lock.json"
            ),
            "failures": record.get("failures", []),
            "superseded_by": final_by_tool[str(record["tool"])],
        }
        for record in superseded
    ]
    merged = {str(attempt["run_id"]): dict(attempt) for attempt in existing_attempts}
    for attempt in new_attempts:
        merged[str(attempt["run_id"])] = attempt
    for attempt in merged.values():
        tool = str(attempt.get("tool", ""))
        if tool not in final_by_tool:
            raise RunEvidenceError(
                f"{tool}: attempt ledger has no final published execution"
            )
        attempt["superseded_by"] = final_by_tool[tool]
    return sorted(merged.values(), key=lambda row: int(str(row["run_id"])[4:]))


def publish_final_evidence(
    root: Path, ledger_path: Path, apply: bool
) -> tuple[int, int]:
    records = validate_runs(root)
    selected, superseded = final_evidence(records)
    existing_attempts: list[dict[str, Any]] = []
    if ledger_path.is_file():
        existing = load_json(ledger_path)
        rows = existing.get("attempts")
        if existing.get("schema_version") != ATTEMPT_LEDGER_SCHEMA or not isinstance(
            rows, list
        ):
            raise RunEvidenceError(f"existing attempt ledger is invalid: {ledger_path}")
        existing_attempts = [dict(row) for row in rows if isinstance(row, dict)]
        if len(existing_attempts) != len(rows):
            raise RunEvidenceError(
                f"existing attempt ledger contains an invalid row: {ledger_path}"
            )
    attempts = attempt_ledger_rows(root, records, existing_attempts)
    ledger = {
        "schema_version": ATTEMPT_LEDGER_SCHEMA,
        "summary": {
            "published_run_count": len(selected),
            "superseded_attempt_count": len(attempts),
            "execution_status": dict(
                sorted(
                    Counter(
                        str(record["execution_status"]) for record in attempts
                    ).items()
                )
            ),
        },
        "attempts": attempts,
    }
    if not apply:
        return len(selected), len(attempts)
    write_json(ledger_path, ledger)
    for record in superseded:
        path = root / f"{record['run_id']}-{record['tool']}"
        shutil.rmtree(path)
    validate_final_evidence(validate_runs(root))
    return len(selected), len(attempts)


def index_rows(root: Path, records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    rows = []
    for record in records:
        row = {
            "run_id": record["run_id"],
            "tool": record["tool"],
            "execution_status": record["execution_status"],
            "scientific_status": record["scientific_status"],
            "repository_commit": record["repository_commit"],
            "slurm_job_id": record["slurm_job_id"],
            "source_result": record["source_result"],
            "source_file_count": len(record["source_files"]),
            "evidence_lock_sha256": sha256_file(
                root / f"{record['run_id']}-{record['tool']}" / "evidence.lock.json"
            ),
        }
        row.update(run_identity(root, record))
        rows.append(row)
    return rows


def render_index(root: Path, output: Path) -> int:
    records = validate_runs(root)
    validate_final_evidence(records)
    validate_attempt_ledger(root / "attempt-ledger.json", records)
    rows = index_rows(root, records)
    write_json(
        output,
        {
            "schema_version": INDEX_SCHEMA,
            "summary": {
                "run_count": len(rows),
                "tool_count": len({row["tool"] for row in rows}),
                "execution_status": dict(
                    sorted(Counter(row["execution_status"] for row in rows).items())
                ),
                "scientific_status": dict(
                    sorted(Counter(row["scientific_status"] for row in rows).items())
                ),
            },
            "runs": rows,
        },
    )
    return len(rows)


def render_report(root: Path, output: Path) -> int:
    records = validate_runs(root)
    validate_final_evidence(records)
    validate_attempt_ledger(root / "attempt-ledger.json", records)
    by_tool: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for record in records:
        by_tool[str(record["tool"])].append(record)
    execution_counts = Counter(str(record["execution_status"]) for record in records)
    science_counts = Counter(str(record["scientific_status"]) for record in records)
    lines = [
        "<!-- GENERATED FILE - DO NOT EDIT -->",
        "<!-- source: python3 containers/runs/manage.py render-report -->",
        "# LUNARC container run evidence",
        "",
        "This report indexes the final successful immutable execution admitted for each",
        "tool. A successful process is not scientific validation, and no tool is qualified",
        "by this report.",
        "",
        "## Archive boundary",
        "",
        f"- Final governed runs: **{len(records)}**",
        f"- Tools represented: **{len(by_tool)}**",
        "- Execution status: "
        + ", ".join(
            f"**{status} {count}**"
            for status, count in sorted(execution_counts.items())
        ),
        "- Scientific status: "
        + ", ".join(
            f"**{status} {count}**" for status, count in sorted(science_counts.items())
        ),
        "",
        "Superseded successful, failed, and incomplete attempts are summarized in the",
        "checksummed attempt ledger without presenting them as current evidence. Their raw",
        "results remain on the governed execution site. Large results stay there; each final",
        "run directory records paths, sizes, and SHA-256 digests.",
        "",
        "The machine-readable [attempt ledger](../runs/attempt-ledger.json) preserves",
        "the identities and lock hashes of superseded attempts.",
        "",
        "## Evidence by tool",
        "",
        "| Tool | Final run | Corpus | Declared stage | Execution | Scientific claim |",
        "|---|---|---|---|---|---|",
    ]
    for tool, tool_records in sorted(by_tool.items()):
        latest = max(tool_records, key=lambda row: int(str(row["run_id"])[4:]))
        science = sorted({str(row["scientific_status"]) for row in tool_records})
        run_link = f"../runs/{latest['run_id']}-{tool}/"
        identity = run_identity(root, latest)
        lines.append(
            f"| `{tool}` | [{latest['run_id']}]({run_link}) | "
            f"`{identity['corpus']}` | `{identity['stage']}` | "
            f"{latest['execution_status']} | {', '.join(science)} |"
        )
    lines.extend(
        [
            "",
            "## Interpretation",
            "",
            "`passed` means the archived Slurm execution completed its declared runtime",
            "contract. `not_assessed` means no predeclared scientific acceptance threshold",
            "was admitted for that run. Release readiness additionally requires reproducible",
            "build, security, benchmark, and freshness evidence; none of those claims can be",
            "inferred from process exit status.",
            "",
            "The machine-readable companion is [run-evidence.json](run-evidence.json).",
        ]
    )
    output.parent.mkdir(parents=True, exist_ok=True)
    temporary = output.with_suffix(output.suffix + ".partial")
    temporary.write_text("\n".join(lines) + "\n", encoding="utf-8")
    temporary.replace(output)
    return len(records)


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    commands = root.add_subparsers(dest="command", required=True)
    importer = commands.add_parser("import-evidence")
    importer.add_argument("--result-root", type=Path, required=True)
    importer.add_argument("--output-root", type=Path, required=True)
    importer.add_argument("--minimum-run", type=int)
    importer.add_argument("--maximum-run", type=int)
    validator = commands.add_parser("validate")
    validator.add_argument("--runs-root", type=Path, required=True)
    publisher = commands.add_parser("publish-final-evidence")
    publisher.add_argument("--runs-root", type=Path, required=True)
    publisher.add_argument("--ledger", type=Path, required=True)
    publisher.add_argument("--apply", action="store_true")
    index = commands.add_parser("render-index")
    index.add_argument("--runs-root", type=Path, required=True)
    index.add_argument("--output", type=Path, required=True)
    report = commands.add_parser("render-report")
    report.add_argument("--runs-root", type=Path, required=True)
    report.add_argument("--output", type=Path, required=True)
    return root


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        if args.command == "import-evidence":
            count = import_evidence(
                args.result_root,
                args.output_root,
                args.minimum_run,
                args.maximum_run,
            )
            print(f"imported {count} governed run records")
            return 0
        if args.command == "validate":
            records = validate_runs(args.runs_root)
            validate_final_evidence(records)
            validate_attempt_ledger(args.runs_root / "attempt-ledger.json", records)
            print(f"validated {len(records)} governed run records")
            return 0
        if args.command == "publish-final-evidence":
            selected, superseded = publish_final_evidence(
                args.runs_root, args.ledger, args.apply
            )
            action = "published" if args.apply else "would publish"
            print(
                f"{action} {selected} final records; {superseded} attempts superseded"
            )
            return 0
        if args.command == "render-index":
            count = render_index(args.runs_root, args.output)
            print(f"indexed {count} governed run records")
            return 0
        count = render_report(args.runs_root, args.output)
        print(f"reported {count} governed run records")
        return 0
    except RunEvidenceError as exc:
        print(f"container run evidence failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
