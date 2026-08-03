#!/usr/bin/env python3
"""Import and validate immutable LUNARC container-run provenance."""

from __future__ import annotations

import argparse
from collections import Counter, defaultdict
import hashlib
import json
from pathlib import Path
import re
import shutil
import sys
from typing import Any, Iterable


RUN_ID = re.compile(r"run-[1-9][0-9]{3,}")
TRACKED_RUN = re.compile(r"(run-[1-9][0-9]{3,})-([a-z0-9_]+)")
LOCK_SCHEMA = "bijux.container.run_evidence_lock.v1"
INDEX_SCHEMA = "bijux.container.run_evidence_index.v1"
COPIED_EVIDENCE = (
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
        record = validate_run(path)
        run_id = str(record["run_id"])
        if run_id in identities:
            raise RunEvidenceError(f"duplicate tracked run identity: {run_id}")
        identities.add(run_id)
        records.append(record)
    return sorted(records, key=lambda record: int(str(record["run_id"])[4:]))


def index_rows(root: Path, records: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [
        {
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
        for record in records
    ]


def missing_run_ids(records: list[dict[str, Any]]) -> list[str]:
    numbers = sorted(int(str(record["run_id"])[4:]) for record in records)
    if not numbers:
        return []
    present = set(numbers)
    return [
        f"run-{number}"
        for number in range(numbers[0], numbers[-1] + 1)
        if number not in present
    ]


def render_index(root: Path, output: Path) -> int:
    records = validate_runs(root)
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
                "missing_run_ids": missing_run_ids(records),
            },
            "runs": rows,
        },
    )
    return len(rows)


def render_report(root: Path, output: Path) -> int:
    records = validate_runs(root)
    by_tool: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for record in records:
        by_tool[str(record["tool"])].append(record)
    execution_counts = Counter(str(record["execution_status"]) for record in records)
    science_counts = Counter(str(record["scientific_status"]) for record in records)
    gaps = missing_run_ids(records)
    lines = [
        "<!-- GENERATED FILE - DO NOT EDIT -->",
        "<!-- source: python3 containers/runs/manage.py render-report -->",
        "# LUNARC container run evidence",
        "",
        "This report indexes immutable execution evidence. A successful process is not",
        "scientific validation: every imported historical run retains its independently",
        "recorded scientific status, and no tool is promoted by this report.",
        "",
        "## Archive boundary",
        "",
        f"- Governed runs: **{len(records)}**",
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
        "- Unassigned historical identifiers: "
        + (", ".join(f"`{gap}`" for gap in gaps) if gaps else "none"),
        "",
        "The gaps are not silently filled. Failed and incomplete attempts remain part of",
        "the audit history. Large results stay on the governed execution site; each run",
        "directory records their paths, sizes, and SHA-256 digests.",
        "",
        "## Evidence by tool",
        "",
        "| Tool | Attempts | Passed | Failed/incomplete | Latest run | Scientific claim |",
        "|---|---:|---:|---:|---|---|",
    ]
    for tool, tool_records in sorted(by_tool.items()):
        latest = max(tool_records, key=lambda row: int(str(row["run_id"])[4:]))
        passed = sum(row["execution_status"] == "passed" for row in tool_records)
        not_passed = len(tool_records) - passed
        science = sorted({str(row["scientific_status"]) for row in tool_records})
        run_link = f"../runs/{latest['run_id']}-{tool}/"
        lines.append(
            f"| `{tool}` | {len(tool_records)} | {passed} | {not_passed} | "
            f"[{latest['run_id']}]({run_link}) | {', '.join(science)} |"
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
            print(f"validated {len(records)} governed run records")
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
