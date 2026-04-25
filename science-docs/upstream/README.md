# Science Docs Upstream Archive

`science-docs/upstream/` is the canonical local archive root for upstream source
material used to support science, container, and provenance review in
`bijux-genomics`.

## Scope

- GitHub repository mirrors
- manual upstream release downloads
- local snapshots of upstream source material that should stay outside Git

## Layout

- `github-repos/README.md`
  contract for GitHub repository evidence mirrors
- `github-repos/MANIFEST.tsv`
  tracked repository target manifest
- `github-repos/mirrors/**`
  untracked local bare clones
- `github-repos/archives/**`
  optional untracked compressed exports

## Rules

- keep the archive payloads untracked
- keep the target list and archive contract tracked
- prefer one stable upstream location per evidence family instead of ad hoc
  directories at the root of `science-docs/`
