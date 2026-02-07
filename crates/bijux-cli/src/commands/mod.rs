#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::Path;

use anyhow::{anyhow, Context, Result};

pub(crate) mod bam;
pub(crate) mod bench;
pub mod cli;
pub(crate) mod command_prelude;
pub(crate) mod fastq;
pub(crate) mod report_inputs;
pub(crate) mod validation;
pub(crate) mod run_plan;

include!("policies.rs");
