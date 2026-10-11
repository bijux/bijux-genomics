"""Admit literal repository Make gates from canonical workflow structure."""

from __future__ import annotations

import argparse
import importlib.util
from pathlib import Path
import re
import shlex

# Load only the admitted YAML parser; unrelated standards commands are not runtime dependencies.
_parser_path = (
    Path(__file__).resolve().parents[2]
    / ".github/scripts/workflow_execution/yaml_io.py"
)
_parser_spec = importlib.util.spec_from_file_location(
    "bijux_workflow_yaml", _parser_path
)
if _parser_spec is None or _parser_spec.loader is None:
    raise RuntimeError("canonical workflow YAML parser is unavailable")
_parser = importlib.util.module_from_spec(_parser_spec)
_parser_spec.loader.exec_module(_parser)
parse_workflow = _parser.parse_workflow


TEST_PREPARATION_TARGETS = frozenset(
    {
        "test",
        "test-fast",
        "test-slow",
        "test-all",
        "test-rs",
        "test-slow-rs",
        "test-all-rs",
        "_test",
        "_test-fast",
        "_test-slow",
        "coverage",
        "coverage-workspace",
        "coverage-rs",
        "ci-fast",
        "_ci-fast",
        "_ci-slow",
        "_ci-profile-fast",
        "_ci-profile-slow",
        "_unit-contract-fast",
        "_test-profile-invariants",
        "science-fixtures-fast",
        "bench-active-fast",
        "bench-parser-fast",
        "bench-adapter-fast",
        "_release-readiness",
        "_fastq-container-readiness",
    }
)


def workflow_gates(content: bytes, label: str) -> list[str]:
    """Require unambiguous literal Make commands and preserve their first order."""
    document = parse_workflow(content, label)
    targets: list[str] = []
    for job_id, job in document["jobs"].items():
        if not isinstance(job_id, str) or not isinstance(job, dict):
            raise ValueError("workflow job must be a named object")
        if "uses" in job:
            raise ValueError(
                "remote reusable jobs cannot be executed as local Make gates"
            )
        steps = job.get("steps")
        if not isinstance(steps, list):
            raise ValueError("workflow job must declare concrete steps")
        for step in steps:
            if not isinstance(step, dict):
                raise ValueError("workflow step must be an object")
            if "run" not in step:
                continue
            command = step["run"]
            if not isinstance(command, str) or "uses" in step:
                raise ValueError("workflow run step must declare one text command")
            for line in command.splitlines():
                lexer = shlex.shlex(line, posix=True, punctuation_chars=";&|()")
                lexer.whitespace_split = True
                tokens = list(lexer)
                if not any(Path(token).name in {"make", "gmake"} for token in tokens):
                    continue
                if (
                    len(tokens) != 2
                    or tokens[0] != "make"
                    or re.fullmatch(r"[A-Za-z0-9_-]+", tokens[1]) is None
                ):
                    raise ValueError(
                        "workflow Make command must have one literal target"
                    )
                target = tokens[1]
                if target not in targets:
                    targets.append(target)
    if not targets:
        raise ValueError("workflow declares no literal Make gates")
    names = [target.lstrip("_").replace("_", "-") for target in targets]
    if len(names) != len(set(names)):
        raise ValueError("workflow gate artifact identities collide")
    return targets


def main() -> int:
    """Print the admitted targets and explicit test preparation requirement."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("workflow", type=Path)
    args = parser.parse_args()
    targets = workflow_gates(args.workflow.read_bytes(), str(args.workflow))
    print(
        "test-preparation"
        if TEST_PREPARATION_TARGETS.intersection(targets)
        else "retained-gates"
    )
    for target in targets:
        print(target)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
