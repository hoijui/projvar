// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::Local;
use cli_utils::BoxError;

use crate::cleanup;
use crate::environment::Environment;
use crate::license;
use crate::std_error;
use crate::var::{Confidence, Key, C_HIGH, C_LOW};
use std::path::{Path, PathBuf};
use std::{env, fs};

use super::{Hierarchy, RetrieveRes};

/// Sources values from the file-system and OS supplied environment variables.
pub struct VarSource;

fn repo_path(environment: &'_ mut Environment) -> Result<&'_ PathBuf, std_error::Error> {
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

/// Reads the whole file
fn file_content(path: &Path) -> RetrieveRes {
    Ok(if path.exists() && path.is_file() {
        let content = fs::read_to_string(path)?;
        Some((C_HIGH, content))
    } else {
        None
    })
}

/// Returns a list of SPDX license identifiers.
/// It looks for the REUSE "LICENSES" dir in the project root,
/// and returns the file names of the containing "*.txt" files.
fn licenses_from_dir(repo_path: &Path) -> Result<Option<Vec<String>>, std_error::Error> {
    let licenses_dir = repo_path.join("LICENSES");
    if licenses_dir.is_dir() {
        log::trace!("Fetching (REUSE) licenses from {licenses_dir:#?} ...");
        let mut licenses = Vec::<String>::new();
        for file in licenses_dir.read_dir()? {
            let file_name = file?.file_name();
            let file_name = file_name.to_str().ok_or(std_error::Error::NotValidUtf8)?;
            if file_name.to_lowercase().ends_with(".txt") {
                let license_id = file_name.get(0 .. file_name.len() - 4)
                    .ok_or("This should be impossible, as we already checked that the extension is there.")
                    .map_err(BoxError::from)?.to_owned();
                log::trace!("Found (REUSE) license: {license_id}");
                licenses.push(license_id);
            }
        }
        Ok(Some(licenses))
    } else {
        Ok(None)
    }
}

/// Returns a list of SPDX license identifiers.
/// It searches for "(LICEN[CS]E|COPYING).*"" files in the project root dir,
/// and figures out which license it contains.
fn licenses_from_files(repo_path: &Path) -> Result<Option<Vec<String>>, std_error::Error> {
    let licenses = license::get_licenses(&repo_path.display().to_string())?;
    Ok(if licenses.is_empty() {
        None
    } else {
        Some(licenses)
    })
}

fn licenses(
    environment: &mut Environment,
    files_first: bool,
) -> Result<Option<Vec<String>>, std_error::Error> {
    let repo_path = repo_path(environment)?;
    let fetcher_functions = if files_first {
        &[licenses_from_files, licenses_from_dir]
    } else {
        &[licenses_from_dir, licenses_from_files]
    };
    for lff in fetcher_functions {
        let licenses = lff(repo_path)?;
        if licenses.is_some() {
            log::trace!("Found licenses: {licenses:#?}");
            return Ok(licenses);
        }
    }
    Ok(None)
}

/// Extracts a single license if there is only a single license,
/// otherwise returns `None`.
fn license(environment: &mut Environment) -> Result<Option<String>, std_error::Error> {
    if let Some(licenses) = licenses(environment, true)? {
        if licenses.len() == 1 {
            return Ok(licenses.first().map(ToOwned::to_owned));
        }
    }
    Ok(None)
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
        std::any::type_name::<Self>()
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
                | Key::RepoCloneUrl
                | Key::RepoCloneUrlGit
                | Key::RepoCloneUrlHttp
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
                Key::License => license(environment)?.map(|val| (C_HIGH, val)),
                Key::Licenses => licenses(environment, false)?.map(|mut lv| {
                    lv.sort();
                    (C_HIGH, lv.join(", "))
                }), // TODO Later on, rather create an SPDX expressions, maybe by using OR instead of ',' to join ... but can we really?
                Key::Name => name(environment)?,
                Key::Version => version(environment)?
                    .map(|conf_val| cleanup::conf_version(environment, conf_val)),
            },
        )
    }
}
