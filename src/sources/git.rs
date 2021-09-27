// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// use std::process::Command;
use crate::environment::Environment;
// use crate::settings;
// use crate::storage;
// use chrono::DateTime;
// use chrono::Local;
// use chrono::NaiveDateTime;
// use chrono::Utc;
// use clap::lazy_static::lazy_static;
// use git2::{self, Repository};
// use regex::Regex;
use crate::tools::git;
// use self::VarSource;
// use super::VarSource;
// use crate::props::version;
use crate::var::Key;
// use std::collections::HashMap;
// use std::convert::TryFrom;
// use std::env;
use std::error::Error;
// use std::fmt::Display;
use std::fmt;
use std::path::Path;

pub struct VarSource;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Returns the name of the given path (same as `basename` on UNIX systems)
fn dir_name(path: &Path) -> BoxResult<String> {
    Ok(path
        .canonicalize()?
        // .parent()
        // .ok_or_else(|| git2::Error::from_str("Unable to get containing directory's name"))?
        .file_name()
        .ok_or_else(|| git2::Error::from_str("File ends in \"..\""))?
        .to_str()
        .ok_or_else(|| git2::Error::from_str("File name is not UTF-8 compatible"))?
        .to_owned())
}

fn version(environment: &mut Environment) -> BoxResult<Option<String>> {
    Ok(if let Some(repo) = environment.repo() {
        let sc_version = repo.version().or_else(|err| {
            log::warn!("Failed to git describe (\"{}\"), using SHA instead", err);
            // repo.sha().map_or_else(|| "No SHA available to serve as version")
            repo.sha()
                .and_then(|v| v.ok_or_else(|| "No SHA available to serve as version".into()))
        })?;

        if git::is_git_dirty_version(&sc_version) {
            log::warn!(
                "Dirty project version ('{}')! (you have uncommitted changes in your project)",
                sc_version
            );
        }
        Some(sc_version)
    } else {
        None
    })
}

fn name(environment: &mut Environment) -> BoxResult<Option<String>> {
    let repo_path = environment
        .settings
        .repo_path
        .as_ref()
        // .map_or(Ok(None), |r| r.map(Some));
        .ok_or("No repo path provided")?;
    let dir_name = dir_name(repo_path)?;
    Ok(match dir_name.as_str() {
        // Filter out some common names that are not likely to be the projects name
        "src" | "target" | "build" | "master" | "main" => None,
        _other => Some(dir_name),
    })
}

fn repo_web_url(environment: &mut Environment) -> BoxResult<Option<String>> {
    if let Some(repo) = environment.repo() {
        Ok(Some(repo.remote_web_url()?))
    } else {
        Ok(None)
    }
}

fn branch(environment: &mut Environment) -> BoxResult<Option<String>> {
    if let Some(repo) = environment.repo() {
        // Ok(repo.branch().unwrap_or_else(|err| {
        //     log::warn!("Failed fetching git branch - {}", err);
        //     None
        // }))
        Ok(repo.branch()?)
    } else {
        Ok(None)
    }
}

fn tag(environment: &mut Environment) -> BoxResult<Option<String>> {
    if let Some(repo) = environment.repo() {
        Ok(repo.tag()?)
    } else {
        Ok(None)
    }
}

fn sha(environment: &mut Environment) -> BoxResult<Option<String>> {
    if let Some(repo) = environment.repo() {
        Ok(repo.sha()?)
    } else {
        Ok(None)
    }
}

fn clone_url(environment: &mut Environment) -> BoxResult<Option<String>> {
    if let Some(repo) = environment.repo() {
        Ok(Some(repo.remote_clone_url()?))
        // repo.remote_clone_url().or_else(|err| {
        //     log::warn!("Failed fetching git repo clone URL - {}", err);
        //     None
        // })
    } else {
        Ok(None)
    }
}

fn version_date(environment: &mut Environment) -> BoxResult<Option<String>> {
    let date_format = &environment.settings.date_format;
    if let Some(repo) = environment.repo() {
        Ok(Some(repo.commit_date(date_format)?))
    } else {
        Ok(None)
    }
}

fn build_hosting_url(environment: &mut Environment) -> BoxResult<Option<String>> {
    if let Some(repo) = environment.repo() {
        Ok(Some(repo.build_hosting_url()?))
    } else {
        Ok(None)
    }
}

/// This uses an alternative method to fetch certain specific variable keys values.
/// Alternative meaning here:
/// Not directly fetching it from any environment variable.
impl super::VarSource for VarSource {
    fn is_usable(&self, environment: &mut Environment) -> bool {
        environment.repo().is_some()
    }

    fn retrieve(&self, environment: &mut Environment, key: Key) -> BoxResult<Option<String>> {
        Ok(match key {
            Key::Version => version(environment)?,
            Key::Name => name(environment)?,
            Key::RepoWebUrl => repo_web_url(environment)?,
            Key::RepoFrozenWebUrl => {
                let base_repo_web_url = self.retrieve(environment, Key::RepoWebUrl)?;
                let version = self.retrieve(environment, Key::Version)?;
                let commit_sha = self.retrieve(environment, Key::BuildIdent)?;

                if let (Some(base_repo_web_url), Some(version_or_sha)) =
                    (base_repo_web_url, version.or(commit_sha))
                {
                    Some(format!("{}/tree/{}", base_repo_web_url, version_or_sha))
                } else {
                    None
                }
                // https://gitlab.com/OSEGermany/okhmanifest
                // https://gitlab.com/OSEGermany/okhmanifest/-/commit/9e1df32c42a85253af95ea2dc9311128bd8f775a
                // https://gitlab.com/OSEGermany/okhmanifest/-/tree/oldCombinedDeprecated
                // https://gitlab.com/OSEGermany/OHS-3105/-/tree/din-3105-0.10.0
                // https://gitlab.com/OSEGermany/OHS-3105/-/tree/din-spec-3105-0.10.0-179-g60c46fc

                // https://github.com/hoijui/repvar
                // https://github.com/hoijui/repvar/tree/4939bd538643bfb445167ea72b825e605f120318
            }
            Key::BuildBranch => branch(environment)?,
            Key::BuildTag => tag(environment)?,
            Key::RepoCloneUrl => clone_url(environment)?,
            Key::VersionDate => version_date(environment)?,
            Key::BuildHostingUrl => build_hosting_url(environment)?,
            Key::BuildIdent => sha(environment)?,
            Key::BuildDate
            | Key::Ci
            | Key::BuildOs
            | Key::BuildOsFamily
            | Key::BuildArch
            | Key::License
            | Key::BuildNumber => None,
        })
    }
}

impl fmt::Display for VarSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::any::type_name::<VarSource>())
    }
}
