# Iteration 01 HPC Campaign Goals

This report tracks the first delivery slice for the HPC campaign foundation.

| Goal | Status | Implementation summary |
| --- | --- | --- |
| G032 | done | Added shared layout contract (`CampaignLayout`) for corpora, databases, images, scratch, logs, encrypted results/code, appraiser imports, and baselines. |
| G033 | done | Added campaign profile scaffolding command and versioned profile files under `configs/hpc/campaign/`. |
| G034 | done | Added confidential env-file loading (`security.env_file`, CLI override `--env-file`) and `.gitignore` coverage for local env inputs. |
| G035 | done | Added tracked-config confidentiality guard that refuses secret-like entries and tracked Slurm account/project fields. |
| G036 | done | Added Slurm account/project resolution from env sources and redacted reporting in preflight/dry-run output. |
| G037 | done | Added per-site profile resolution, including file-backed profiles (`site-profiles/*.toml`) and builtin fallbacks. |
| G038 | done | Added per-user policy loading from `user.policy.toml` and explicit reporting of policy application state. |
| G039 | done | Added campaign preflight command and contract checks (schema, layout validity, Slurm resolution, template validity, env-file permissions, resource templates). |
| G040 | done | Added campaign dry-run command with deterministic planned job expansion and resolved output locations. |
| G074 | done | Added resource template catalog and stage-family template defaults for job-level resource selection. |

Additional implemented support:

- G065 path token templating and required template-token validation.
