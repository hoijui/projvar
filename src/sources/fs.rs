// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::Local;
use clap::lazy_static::lazy_static;
use regex::Regex;

use crate::environment::Environment;
use crate::std_error;
use crate::var::{Confidence, Key, C_HIGH, C_LOW, C_MIDDLE};
use std::path::{Path, PathBuf};
use std::{env, fs};

use super::{Hierarchy, RetrieveRes};

/// Sources values from the file-system and OS supplied environment variables.
pub struct VarSource;

fn repo_path<'a>(environment: &'a mut Environment) -> Result<&'a PathBuf, std_error::Error> {
    environment
        .settings
        .repo_path
        .as_ref()
        .ok_or(std_error::Error::None)
}

/// Returns the name of the given path (same as `basename` on UNIX systems)
fn dir_name(path: &Path) -> Result<String, std_error::Error> {
    Ok(path
        .canonicalize()?
        .file_name()
        .ok_or(std_error::Error::PathNotAFile)?
        .to_str()
        .ok_or(std_error::Error::NotValidUtf8)?
        .to_owned())
}

/// Read the whole file
fn file_content(path: &Path) -> RetrieveRes {
    Ok(if path.exists() && path.is_file() {
        let content = fs::read_to_string(path)?;
        Some((C_HIGH, content))
    } else {
        None
    })
}

fn licenses(environment: &mut Environment) -> Result<Option<Vec<String>>, std_error::Error> {
    lazy_static! {
        static ref R_TXT_SUFFIX: Regex = Regex::new(r"\.txt$").unwrap();
    }
    let repo_path = repo_path(environment)?;
    let licenses_dir = repo_path.join("LICENSES");
    if licenses_dir.is_dir() {
        let mut licenses = Vec::<String>::new();
        for file in licenses_dir.read_dir()? {
            let file_name = file?.file_name();
            let file_name = file_name.to_str().ok_or(std_error::Error::NotValidUtf8)?;
            if R_TXT_SUFFIX.is_match(file_name) {
                licenses.push(R_TXT_SUFFIX.replace(file_name, "").into_owned());
            }
        }
        Ok(Some(licenses))
    } else {
        Ok(None)
    }
}

fn version(environment: &mut Environment) -> RetrieveRes {
    Ok(match &environment.settings.repo_path {
        Some(repo_path) => {
            let version_file = repo_path.join("VERSION");
            file_content(&version_file)?
        }
        _ => None,
    })
}

fn name(environment: &mut Environment) -> RetrieveRes {
    let dir_name = dir_name(repo_path(environment)?)?;
    Ok(match dir_name.to_lowercase().as_str() {
        // Filter out some common directory names that are not likely to be the projects name
        "src" | "target" | "build" | "master" | "main" | "develop" | "git" | "repo" | "repos"
        | "scm" | "trunk" => None,
        _ => Some((C_LOW, dir_name)),
    })
}

fn build_date(environment: &mut Environment) -> String {
    let now = Local::now();
    now.format(&environment.settings.date_format).to_string()
}

fn build_os(_environment: &mut Environment) -> (Confidence, String) {
    // See here for possible values:
    // <https://doc.rust-lang.org/std/env/consts/constant.OS.html>
    // Most common values: "linux", "macos", "windows"
    (C_LOW, env::consts::OS.to_owned()) // TODO Maybe move to a new source "env.rs"? AND Map to our own values!
}

fn build_os_family(_environment: &mut Environment) -> (Confidence, String) {
    // Possible values: "unix", "windows"
    // <https://doc.rust-lang.org/std/env/consts/constant.FAMILY.html>
    // format!("{}", env::consts::FAMILY)
    (C_LOW, env::consts::FAMILY.to_owned()) // TODO Maybe move to a new source "env.rs"?
}

fn build_arch(_environment: &mut Environment) -> (Confidence, String) {
    // See here for possible values:
    // <https://doc.rust-lang.org/std/env/consts/constant.ARCH.html>
    // Most common values: "x86", "x86_64"
    (C_LOW, env::consts::ARCH.to_owned()) // TODO Maybe move to a new source "env.rs"?
}

/// This uses an alternative method to fetch certain specific variable keys values.
/// Alternative meaning here:
/// Not directly fetching it from any environment variable.
impl super::VarSource for VarSource {
    fn is_usable(&self, environment: &mut Environment) -> bool {
        environment.repo().is_some()
    }

    fn hierarchy(&self) -> Hierarchy {
        Hierarchy::Low
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<VarSource>()
    }

    fn properties(&self) -> &Vec<String> {
        &super::NO_PROPS
    }

    #[remain::check]
    fn retrieve(&self, environment: &mut Environment, key: Key) -> RetrieveRes {
        Ok(
            #[remain::sorted]
            match key {
                Key::BuildArch => Some(build_arch(environment)),
                Key::BuildBranch
                | Key::BuildHostingUrl
                | Key::BuildNumber
                | Key::BuildTag
                | Key::Ci
                | Key::License
                | Key::RepoCloneUrl
                | Key::RepoCloneUrlSsh
                | Key::RepoCommitPrefixUrl
                | Key::RepoIssuesUrl
                | Key::RepoRawVersionedPrefixUrl
                | Key::RepoVersionedDirPrefixUrl
                | Key::RepoVersionedFilePrefixUrl
                | Key::RepoWebUrl
                | Key::VersionDate
                | Key::NameMachineReadable => None,
                Key::BuildDate => Some((C_HIGH, build_date(environment))),
                Key::BuildOs => Some(build_os(environment)),
                Key::BuildOsFamily => Some(build_os_family(environment)),
                Key::Licenses => licenses(environment)?.map(|lv| (C_HIGH, lv.join(", "))), // TODO Later on, rather create an SPDX expressions, maybe by using OR instead of ',' to join ... but can we really?
                Key::Name => name(environment)?,
                Key::Version => version(environment)?,
            },
        )
    }
}
