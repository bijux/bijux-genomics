"""Exercise real local Git acquisition without network or the user's Git config."""
import csv
import os
from pathlib import Path
import subprocess
import sys
import tempfile

script = Path(sys.argv[1]).resolve()
with tempfile.TemporaryDirectory(dir=sys.argv[2]) as directory:
    root = Path(directory).resolve()
    repository = root / "product"
    source = root / "origin"
    repository.mkdir()
    source.mkdir()
    config = root / "gitconfig"
    config.write_text(f'[url "{source.as_uri()}"]\n\tinsteadOf = https://github.com/fixture/tool.git\n')
    env = {
        **os.environ,
        "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_CONFIG_GLOBAL": str(config),
        "GIT_ALLOW_PROTOCOL": "file",
        "GIT_TERMINAL_PROMPT": "0",
    }
    env.pop("BIJUX_SCIENCE_ARCHIVE_ROOT", None)
    def git(*args):
        return subprocess.check_output(["git", *args], cwd=source, env=env, text=True).strip()
    git("init", "-q", "-b", "main")
    (source / "README.md").write_text("immutable source\n")
    git("add", "README.md")
    git("-c", "user.name=fixture", "-c", "user.email=fixture@example.invalid", "commit", "-qm", "fixture")
    head = git("rev-parse", "HEAD")
    docs = repository / "docs/20-science"
    docs.mkdir(parents=True)
    (docs / "source.md").write_text("Upstream: https://github.com/fixture/tool\n")
    def run(*args, **extra_env):
        return subprocess.run([sys.executable, "-B", str(script), "--repo-root", str(repository), *args],
                              env={**env, **extra_env}, text=True, capture_output=True, timeout=30)
    evidence = root / "bijux-genomics-science"
    result = run("--skip-sync")
    assert result.returncode == 0, result.stderr
    assert not evidence.exists()
    result = run("--archive-format", "tar.gz")
    assert result.returncode == 0, result.stderr
    mirror = evidence / "upstream/github-repos/mirrors/fixture/tool.git"
    assert git("--git-dir", str(mirror), "rev-parse", "HEAD") == head
    assert (evidence / "upstream/github-repos/archives/fixture--tool.tar.gz").is_file()
    assert not (repository / "science/docs/upstream/github-repos/mirrors").exists()
    with (repository / "science/docs/upstream/github-repos/MANIFEST.tsv").open() as stream:
        rows = list(csv.DictReader(stream, delimiter="\t"))
    assert len(rows) == 1 and rows[0]["mirror_path"].startswith("science/docs/upstream/")
    result = run()
    assert result.returncode == 0, result.stderr
    assert git("--git-dir", str(mirror), "rev-parse", "HEAD") == head
    alternative = root / "custom evidence"
    result = run("--evidence-root", str(alternative))
    assert result.returncode == 0, result.stderr
    assert (alternative / "upstream/github-repos/mirrors/fixture/tool.git").is_dir()
    configured = root / "environment archive"
    result = run(BIJUX_SCIENCE_ARCHIVE_ROOT=str(configured))
    assert result.returncode == 0, result.stderr
    assert (configured / "upstream/github-repos/mirrors/fixture/tool.git").is_dir()
    for args in [
        ("--evidence-root", str(repository / "science/docs")),
        ("--mirror-root", str(repository / "downloads")),
        ("--mirror-root", "../../escaped"),
        ("--archive-root", str(repository / "downloads")),
    ]:
        assert run(*args).returncode != 0, args
    alias = root / "archive-alias"
    alias.symlink_to(repository, target_is_directory=True)
    assert run("--evidence-root", str(alias)).returncode != 0
    redirected = root / "redirected archive"
    owner = redirected / "upstream/github-repos/mirrors/fixture"
    owner.parent.mkdir(parents=True)
    owner.symlink_to(repository, target_is_directory=True)
    assert run("--evidence-root", str(redirected)).returncode != 0
    assert not (repository / "tool.git").exists()
    assert not (repository / "downloads").exists()
    print("External defaults, clone/fetch, export, locator compatibility, override and rejection checks passed.")
