//! Owner: bijux-dna-analyze
//! Scoring helpers for decision ranking.

mod normalization;
mod ties;
mod weights;

pub(crate) use normalization::{format_optional, min_max, normalize_inverted, penalties_for_input};
pub(crate) use ties::annotate_why_not_first;
pub(crate) use weights::{assert_metric_semantics, trace_for_input};
