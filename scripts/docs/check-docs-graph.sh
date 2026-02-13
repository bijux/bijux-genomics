#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys
from collections import defaultdict, deque

root = Path(sys.argv[1])
docs = root / "docs"
graph = docs / "DOCS_GRAPH.toml"
if not graph.exists():
    print("docs-graph: missing docs/DOCS_GRAPH.toml", file=sys.stderr)
    sys.exit(1)

edges = defaultdict(list)
graph_nodes = set()
cur_from = None
in_children = False
for raw in graph.read_text(encoding="utf-8").splitlines():
    line = raw.strip()
    if not line or line.startswith("#"):
        continue
    if line.startswith("[[edge]]"):
        cur_from = None
        in_children = False
        continue
    if line.startswith("from = "):
        cur_from = line.split("=",1)[1].strip().strip('"')
        graph_nodes.add(cur_from)
        continue
    if line.startswith("children = ["):
        in_children = True
        continue
    if in_children:
        if line == "]":
            in_children = False
            continue
        child = line.rstrip(",").strip().strip('"')
        if cur_from:
            edges[cur_from].append(child)
            graph_nodes.add(child)

if "docs/index.md" not in edges:
    print("docs-graph: docs/index.md missing from graph roots", file=sys.stderr)
    sys.exit(1)

# Graph node validity
graph_node_errors = []
for n in sorted(graph_nodes):
    p = root / n
    if not p.exists():
        graph_node_errors.append(f"missing graph node target: {n}")

# Missing target file links in markdown
pat = re.compile(r'\[[^\]]*\]\(([^)]+)\)')
link_errors = []
for md in docs.rglob("*.md"):
    text = md.read_text(encoding="utf-8")
    for target in pat.findall(text):
        t = target.strip()
        if not t or t.startswith(("http://","https://","mailto:","#")):
            continue
        t = t.split('#',1)[0]
        if not t:
            continue
        cand = (root / t.lstrip('/')) if t.startswith('/') else (md.parent / t)
        if not cand.exists():
            link_errors.append(f"{md.relative_to(root).as_posix()} -> {target}")

# Section folder must have index.md if it contains markdown files
index_errors = []
for d in [docs] + [p for p in docs.rglob("*") if p.is_dir()]:
    md_files = [p for p in d.glob("*.md")]
    if not md_files:
        continue
    if (d / "index.md").exists():
        continue
    # allow generated graph file at docs root as exception file; not a directory issue
    index_errors.append(f"{d.relative_to(root).as_posix()}")

# Reachability via graph edges
all_docs = {p.relative_to(root).as_posix() for p in docs.rglob("*.md")}
reachable = set()
q = deque(["docs/index.md"])
while q:
    n = q.popleft()
    if n in reachable:
        continue
    reachable.add(n)
    for c in edges.get(n, []):
        if c not in reachable:
            q.append(c)

# docs/DOCS_GRAPH.toml itself is config for graph and can be excluded from reachability set
all_docs_no_graph = {p for p in all_docs if p != "docs/DOCS_GRAPH.toml"}
unreach = sorted(all_docs_no_graph - reachable)

failed = False
if link_errors:
    failed = True
    print("docs-graph: missing markdown link targets:", file=sys.stderr)
    for e in sorted(link_errors):
        print(f"  - {e}", file=sys.stderr)
if graph_node_errors:
    failed = True
    print("docs-graph: graph contains missing nodes:", file=sys.stderr)
    for e in graph_node_errors:
        print(f"  - {e}", file=sys.stderr)
if index_errors:
    failed = True
    print("docs-graph: section folder lacks index.md:", file=sys.stderr)
    for e in sorted(index_errors):
        print(f"  - {e}", file=sys.stderr)
if unreach:
    failed = True
    print("docs-graph: docs not reachable from docs/index.md via docs/DOCS_GRAPH.toml:", file=sys.stderr)
    for e in unreach:
        print(f"  - {e}", file=sys.stderr)

if failed:
    sys.exit(1)

print("docs-graph: OK")
PY
