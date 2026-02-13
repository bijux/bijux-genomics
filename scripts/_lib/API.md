# scripts/_lib API

Exported functions (stable):
- `die <msg> [code]`: print fatal error and exit.
- `warn <msg>`: print warning to stderr.
- `info <msg>`: print info message to stderr.
- `require_stable_env`: enforce `TZ=UTC` and `LC_ALL=C`.
- `require_cmd <name>`: fail if command is unavailable.
- `require_file <path>`: fail if file is missing.
- `require_dir <path>`: fail if directory is missing.
- `require_env <VAR>`: fail if env var is missing/empty.
- `repo_root`: print repo root path.
- `ensure_artifacts_dir <path>`: allow writes only under `artifacts/` or `$ISO_ROOT`.
- `write_artifact <path> [content...]`: safe artifact write helper.
- `compat_sed_inplace <expr> <file>`: portable in-place sed update for macOS/Linux.
- `compat_readlink_f <path>`: portable realpath equivalent.

Naming rules:
- Public functions are listed above and may be used by scripts outside `_lib`.
- Private helpers in `_lib/common.sh` must use `_internal_` prefix.
- Scripts outside `_lib` must not define duplicate API functions.
