# PHASES

This document maps BAM pipeline phases to their params/metrics modules.

## Pre
Stages that prepare or validate inputs before core processing.
- Params: `params/pre/*`
- Metrics: `metrics/pre/*`

## Core
Stages that operate on primary BAM processing (align, filter, markdup, etc.).
- Params: `params/core/*`
- Metrics: `metrics/core/*`

## Downstream
Stages that derive reports/analytics from core BAM outputs.
- Params: `params/downstream/*`
- Metrics: `metrics/downstream/*`
