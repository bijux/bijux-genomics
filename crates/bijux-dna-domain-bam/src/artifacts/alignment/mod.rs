//! Format adapters share one miniature alignment document and header-preservation policy.
//! This internal boundary performs fixture IO; it does not choose tools or evaluate scientific reports.
mod bam;
mod document;
mod header;
mod sam;

use anyhow::{anyhow, Result};
use bam::parse_tiny_bam;
pub(super) use document::{TinySamDocument, TinySamRecord};
pub(super) use header::merge as merge_header;
pub(super) use sam::{parse_tiny_sam, write_tiny_sam_document, write_tiny_sam_from_document};
use std::path::Path;

pub(super) fn parse_tiny_alignment(path: &Path) -> Result<TinySamDocument> {
    match path.extension().and_then(std::ffi::OsStr::to_str) {
        Some("bam") => parse_tiny_bam(path).or_else(|bam_error| {
            parse_tiny_sam(path).map_err(|sam_error| {
                anyhow!(
                    "failed to parse {} as BAM ({bam_error}) or SAM ({sam_error})",
                    path.display()
                )
            })
        }),
        _ => parse_tiny_sam(path),
    }
}
