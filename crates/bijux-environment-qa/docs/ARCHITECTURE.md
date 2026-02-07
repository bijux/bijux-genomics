# Architecture

## Why QA is a separate crate
QA is heavy and effectful (docker runs, large IO, optional network). Keeping it separate prevents production crates from inheriting those dependencies and side effects.

## Modules
- image_qa/
- bin/* wrappers

## Data flow
- Image inputs → QA reports.
