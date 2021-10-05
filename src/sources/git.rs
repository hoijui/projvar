// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::lazy_static::lazy_static;
use regex::Regex;

use crate::environment::Environment;
use crate::tools::git;
use crate::var::Key;
use std::error::Error;

use super::Hierarchy;
pub struct VarSource;

type BoxResult<T> = Result<T, Box<dyn Error>>;

fn version(environment: &mut Environment) -> BoxResult<Option<String>> {
    Ok(match environment.repo() {
        Some(repo) => {
            let sc_version = repo.version().or_else(|err| {
                log::warn!("Failed to git describe (\"{}\"), using SHA instead", err);
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
        }
        None => None,
    })
}

fn name(environment: &mut Environment) -> BoxResult<Option<String>> {
    lazy_static! {
        static ref R_REMOTE_NAME_SELECTOR: Regex = Regex::new(r"^.*/(?P<name>[^/]+)$").unwrap();
    }

    Ok(match environment.repo() {
        Some(repo) => {
            let repo_web_url = repo.remote_web_url()?;
            let name = R_REMOTE_NAME_SELECTOR.replace(&repo_web_url, "$name");
            if name == repo_web_url {
                // return Err(Box::new(Error::new(""))); // TODO Create a propper sources::Error type, and use it here
                None // HACK
            } else {
                Some(name.into_owned())
            }
        }
        None => None,
    })
}

fn repo_web_url(environment: &mut Environment) -> BoxResult<Option<String>> {
    Ok(match environment.repo() {
        Some(repo) => Some(repo.remote_web_url()?),
        None => None,
    })
}

fn branch(environment: &mut Environment) -> BoxResult<Option<String>> {
    Ok(match environment.repo() {
        Some(repo) => {
            // Ok(repo.branch().unwrap_or_else(|err| {
            //     log::warn!("Failed fetching git branch - {}", err);
            //     None
            // }))
            repo.branch()?
        }
        None => None,
    })
}

fn tag(environment: &mut Environment) -> BoxResult<Option<String>> {
    Ok(match environment.repo() {
        Some(repo) => repo.tag()?,
        None => None,
    })
}

fn clone_url(environment: &mut Environment) -> BoxResult<Option<String>> {
    Ok(match environment.repo() {
        Some(repo) => {
            Some(repo.remote_clone_url()?)
            // repo.remote_clone_url().or_else(|err| {
            //     log::warn!("Failed fetching git repo clone URL - {}", err);
            //     None
            // })
        }
        None => None,
    })
}

fn issues_url(environment: &mut Environment) -> BoxResult<Option<String>> {
    Ok(match environment.repo() {
        Some(repo) => {
            Some(repo.issues_url()?)
            // repo.remote_clone_url().or_else(|err| {
            //     log::warn!("Failed fetching git repo clone URL - {}", err);
            //     None
            // })
        }
        None => None,
    })
}

fn version_date(environment: &mut Environment) -> BoxResult<Option<String>> {
    let date_format = &environment.settings.date_format;
    Ok(match environment.repo() {
        Some(repo) => Some(repo.commit_date(date_format)?),
        None => None,
    })
}

fn build_hosting_url(environment: &mut Environment) -> BoxResult<Option<String>> {
    Ok(match environment.repo() {
        Some(repo) => Some(repo.build_hosting_url()?),
        None => None,
    })
}

/// This uses an alternative method to fetch certain specific variable keys values.
/// Alternative meaning here:
/// Not directly fetching it from any environment variable.
impl super::VarSource for VarSource {
    fn is_usable(&self, environment: &mut Environment) -> bool {
        environment.repo().is_some()
    }

    fn hierarchy(&self) -> Hierarchy {
        Hierarchy::Middle
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<VarSource>()
    }

    fn properties(&self) -> &Vec<String> {
        &super::NO_PROPS
    }

    fn retrieve(&self, environment: &mut Environment, key: Key) -> BoxResult<Option<String>> {
        Ok(match key {
            Key::Version => version(environment)?,
            Key::Name => name(environment)?,
            Key::RepoWebUrl => repo_web_url(environment)?,
            Key::BuildBranch => branch(environment)?,
            Key::BuildTag => tag(environment)?,
            Key::RepoCloneUrl => clone_url(environment)?,
            Key::RepoRawVersionedPrefixUrl => {
                super::try_construct_raw_prefix_url(self, environment)?
            }
            Key::RepoVersionedFilePrefixUrl => {
                super::try_construct_file_prefix_url(self, environment)?
            }
            Key::RepoVersionedDirPrefixUrl => {
                super::try_construct_dir_prefix_url(self, environment)?
            }
            Key::RepoCommitPrefixUrl => super::try_construct_commit_prefix_url(self, environment)?,
            Key::RepoIssuesUrl => issues_url(environment)?,
            Key::VersionDate => version_date(environment)?,
            Key::BuildHostingUrl => build_hosting_url(environment)?,
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
