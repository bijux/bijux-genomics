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
OUT_DIR = Path("science-docs/upstream/fastq/container")


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


def write_tsv(path: Path, header: list[str], rows: list[list[str]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t", lineterminator="\n")
        writer.writerow(header)
        writer.writerows(rows)


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


def main() -> int:
    args = parse_args()
    root = args.repo_root.resolve()
    out_dir = args.out_dir if args.out_dir.is_absolute() else root / args.out_dir
    stages = load_stages(root)
    tools = load_tool_containers(root)
    registry = load_registry(root)

    matrix = []
    digest_rows = []
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
    print(
        json.dumps(
            {
                "written": [
                    str(out_dir / "FASTQ_CONTAINER_DEFAULT_MATRIX.tsv"),
                    str(out_dir / "FASTQ_CONTAINER_DIGEST_CLASSES.tsv"),
                    str(out_dir / "FASTQ_CONTAINER_ASSET_HOOKS.tsv"),
                ]
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
