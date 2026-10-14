//! Strict allele parsing and dosage/zygosity observations for mini-fixture truth.
use std::collections::BTreeSet;

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub(super) struct ParsedVariantRecord {
    pub(super) contig: String,
    pub(super) position: u64,
    pub(super) sample_genotypes: Vec<ParsedGenotype>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ParsedGenotype {
    Missing,
    Unphased(Vec<u32>),
    Phased(Vec<u32>),
}

impl ParsedGenotype {
    pub(super) fn dosage(&self) -> Option<u64> {
        match self {
            Self::Missing => None,
            Self::Unphased(alleles) | Self::Phased(alleles) => {
                Some(alleles.iter().map(|allele| u64::from(*allele > 0)).sum())
            }
        }
    }

    pub(super) fn is_homozygous(&self) -> bool {
        match self {
            Self::Missing => false,
            Self::Unphased(alleles) | Self::Phased(alleles) => {
                !alleles.is_empty() && alleles.iter().all(|allele| *allele == alleles[0])
            }
        }
    }

    pub(super) fn is_hom_ref(&self) -> bool {
        match self {
            Self::Unphased(alleles) | Self::Phased(alleles) => {
                !alleles.is_empty() && alleles.iter().all(|allele| *allele == 0)
            }
            Self::Missing => false,
        }
    }

    pub(super) fn is_hom_alt(&self) -> bool {
        match self {
            Self::Unphased(alleles) | Self::Phased(alleles) => {
                !alleles.is_empty()
                    && alleles.iter().all(|allele| *allele == alleles[0] && *allele > 0)
            }
            Self::Missing => false,
        }
    }

    pub(super) fn is_het(&self) -> bool {
        match self {
            Self::Unphased(alleles) | Self::Phased(alleles) => {
                let distinct = alleles.iter().collect::<BTreeSet<_>>();
                distinct.len() > 1
            }
            Self::Missing => false,
        }
    }
}

pub(super) fn parse_variant_record(fields: &[&str]) -> Result<ParsedVariantRecord> {
    let position =
        fields[1].parse::<u64>().with_context(|| format!("parse VCF position `{}`", fields[1]))?;
    let format = fields[8];
    let format_keys = format.split(':').collect::<Vec<_>>();
    let gt_index = format_keys
        .iter()
        .position(|token| *token == "GT")
        .ok_or_else(|| anyhow!("VCF truth parser requires GT in FORMAT"))?;
    let sample_genotypes = fields[9..]
        .iter()
        .map(|sample_payload| {
            let tokens = sample_payload.split(':').collect::<Vec<_>>();
            let gt = tokens.get(gt_index).copied().unwrap_or(".");
            parse_genotype(gt)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(ParsedVariantRecord { contig: fields[0].to_string(), position, sample_genotypes })
}

fn parse_genotype(gt: &str) -> Result<ParsedGenotype> {
    if matches!(gt, "." | "./." | ".|." | "./" | ".|") {
        return Ok(ParsedGenotype::Missing);
    }
    if let Some((left, right)) = gt.split_once('|') {
        return Ok(ParsedGenotype::Phased(vec![
            parse_allele_token(left)?,
            parse_allele_token(right)?,
        ]));
    }
    if let Some((left, right)) = gt.split_once('/') {
        return Ok(ParsedGenotype::Unphased(vec![
            parse_allele_token(left)?,
            parse_allele_token(right)?,
        ]));
    }
    Ok(ParsedGenotype::Unphased(vec![parse_allele_token(gt)?]))
}

fn parse_allele_token(token: &str) -> Result<u32> {
    token.parse::<u32>().with_context(|| format!("parse genotype allele token `{token}`"))
}
