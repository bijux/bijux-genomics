# GitHub Repository Evidence Archive

This directory governs the local archive of GitHub repositories referenced as
upstream evidence by `bijux-genomics`.

## Canonical Targets

- `MANIFEST.tsv`
  tracked repository target list and planned local paths
- `mirrors/<owner>/<repo>.git`
  untracked bare clone for the local archive
- `archives/<owner>--<repo>.tar.gz`
  optional untracked compressed export generated on demand

## Sync Rule

Refresh this surface with:

```bash
python3 makes/bin/sync_science_docs_github_repos.py
```

Add `--archive-format tar.gz` when a compressed portable snapshot is required.

## Scope Boundary

This archive is for evidence-bearing upstream repositories, not general GitHub
infrastructure links such as package pages, workflow badges, pull requests, or
release status links.
