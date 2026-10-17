"""Admit the producer-owned developer executable without compiling on consumers."""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


def admitted_runner(root):
    build = root / "artifacts/ci/distributed/build"
    if build.resolve() != build:
        raise ValueError("downloaded CI products have a redirected artifact owner")
    metadata = json.loads((build / "producer.json").read_text())
    source = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    if metadata["source_commit"] != source:
        raise ValueError("downloaded CI runner belongs to a different source commit")
    runner = build / "bijux-dna-dev"
    if runner.is_symlink() or not runner.is_file() or runner.resolve().parent != build.resolve():
        raise ValueError("downloaded CI runner has no physical artifact owner")
    with runner.open("rb") as binary:
        actual = hashlib.file_digest(binary, "sha256").hexdigest()
    if metadata["runner_sha256"] != actual:
        raise ValueError("downloaded CI runner checksum differs from its producer")
    runner.chmod(0o555)
    return runner


def main(arguments):
    if arguments == ["aggregate"]:
        arguments = ["--github-aggregate"]
    elif len(arguments) == 2 and arguments[0] == "consumer":
        arguments = ["--github-consumer", arguments[1]]
    else:
        raise ValueError("expected aggregate or consumer <partition>")
    root = Path(__file__).resolve().parents[2]
    runner = admitted_runner(root)
    os.execv(runner, [str(runner), "tooling", "run", "ci-test-all-sharded", *arguments])


if __name__ == "__main__":
    main(sys.argv[1:])
