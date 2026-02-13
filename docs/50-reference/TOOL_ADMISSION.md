# TOOL_ADMISSION

How a new tool is admitted into production workflows.

## Required Path
1. Add tool/domain metadata in authored domain sources.
2. Regenerate CI configs (`tool_registry`, `stages`, `required_tools`, `images`).
3. Add/update container build artifacts under `containers/`.
4. Ensure smoke/QA coverage passes for the tool runtime paths.
5. Update relevant docs (`TOOL_INDEX`, science/operations notes).

## Admission Gate
A tool is considered admitted only when registry, containers, QA, and docs are all consistent and CI passes.

## Purpose
This document defines the intended behavior and navigation contract for this topic.

## Scope
Applies only to the files and workflows referenced in this document.

## Non-goals
- Not a replacement for lower-level implementation or crate-specific contracts.

## Contracts
- Content here is normative where explicitly stated.

