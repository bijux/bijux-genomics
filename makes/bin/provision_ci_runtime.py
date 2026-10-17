"""Build the pinned native test runtime inside repository-owned artifacts."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile
import tempfile
import urllib.request


VERSION = "bcftools 1.20"
SOURCE_URL = "https://github.com/samtools/bcftools/releases/download/1.20/bcftools-1.20.tar.bz2"
SOURCE_SHA256 = "312b8329de5130dd3a37678c712951e61e5771557c7129a70a327a300fda8620"


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def require_version(executable):
    result = subprocess.run([str(executable), "--version"], capture_output=True, text=True, check=True)
    if result.stdout.splitlines()[0] != VERSION:
        raise ValueError(f"expected {VERSION}: {result.stdout.splitlines()[0]}")


def checked_archive(path):
    if digest(path) != SOURCE_SHA256:
        raise ValueError("native runtime source checksum does not match the pinned release")
    return path


def provision(workspace, runtime, jobs):
    boundary = workspace.resolve() / "artifacts"
    runtime = runtime.resolve()
    if not runtime.is_relative_to(boundary) or runtime == boundary:
        raise ValueError("native test runtime must stay beneath repository artifacts/")
    if not 1 <= jobs <= 16:
        raise ValueError("native runtime build jobs must be between 1 and 16")
    runtime.mkdir(parents=True, exist_ok=True)
    executable = runtime / "bcftools/install/bin/bcftools"
    if not executable.resolve().is_relative_to(runtime):
        raise ValueError("native runtime executable escapes its artifact owner")
    if executable.exists():
        require_version(executable)
    else:
        if platform.system() != "Linux":
            raise ValueError("automatic runtime compilation requires Linux; provision the exact release locally")
        owner = runtime / "bcftools"
        owner.mkdir(exist_ok=True)
        archive = owner / "bcftools-1.20.tar.bz2"
        if not archive.exists():
            partial = owner / "bcftools-1.20.tar.bz2.partial"
            with urllib.request.urlopen(SOURCE_URL, timeout=120) as response, partial.open("wb") as output:
                shutil.copyfileobj(response, output)
            checked_archive(partial).replace(archive)
        checked_archive(archive)
        with tempfile.TemporaryDirectory(prefix="source-", dir=owner) as scratch:
            with tarfile.open(archive, "r:bz2") as source:
                source.extractall(scratch, filter="data")
            source_root = Path(scratch) / "bcftools-1.20"
            install = owner / "install"
            with (owner / "build.log").open("w") as log:
                for command in (["./configure", f"--prefix={install}"],
                                ["make", f"-j{jobs}"], ["make", "install"]):
                    subprocess.run(command, cwd=source_root, stdout=log, stderr=subprocess.STDOUT, check=True)
        require_version(executable)
    record = {"executable": str(executable), "version": VERSION,
              "executable_sha256": digest(executable), "source_url": SOURCE_URL,
              "source_sha256": SOURCE_SHA256, "platform": platform.platform()}
    (runtime / "native-test-runtime.json").write_text(json.dumps(record, indent=2) + "\n")
    print(f"native test runtime ready: {executable}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runtime-root", required=True, type=Path)
    parser.add_argument("--jobs", type=int, default=min(os.cpu_count() or 1, 16))
    args = parser.parse_args()
    provision(Path(__file__).resolve().parents[2], args.runtime_root, args.jobs)


if __name__ == "__main__":
    main()
