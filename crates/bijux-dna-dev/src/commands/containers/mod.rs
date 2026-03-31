use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate, Utc};
use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::model::container::{ContainerCommandOutcome, NativeContainerCommandKey};
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;

mod command_support;
mod content_support;
mod dispatch;
mod metadata;
mod registry_catalog;
mod runtime;
mod validation;
mod version_state;
mod versioning;

use self::command_support::*;
use self::content_support::*;
use self::registry_catalog::*;
use self::runtime::*;
use self::version_state::*;

pub fn run_native_container_command(
    key: NativeContainerCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    dispatch::run_native_container_command(key, workspace, args)
}

fn command_hostname() -> String {
    for args in [["-f"].as_slice(), [].as_slice()] {
        let mut command = std::process::Command::new("hostname");
        command.args(args);
        let Ok(output) = command.output() else {
            continue;
        };
        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !value.is_empty() {
                return value;
            }
        }
    }
    String::new()
}
