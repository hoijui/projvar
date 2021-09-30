// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::lazy_static::lazy_static;
use regex::Regex;

use crate::environment::Environment;
use crate::tools::git;
use crate::var::Key;
use std::error::Error;
use std::fmt;

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
        Some(repo) => match repo.remote_tracking_branch() {
            Ok(remote_tracking_branch) => Some(
                R_REMOTE_NAME_SELECTOR
                    .replace(&remote_tracking_branch, "$name")
                    .into_owned(),
            ),
            Err(err) => return Err(err),
        },
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

fn sha(environment: &mut Environment) -> BoxResult<Option<String>> {
    Ok(match environment.repo() {
        Some(repo) => repo.sha()?,
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

    fn retrieve(&self, environment: &mut Environment, key: Key) -> BoxResult<Option<String>> {
        Ok(match key {
            Key::Version => version(environment)?,
            Key::Name => name(environment)?,
            Key::RepoWebUrl => repo_web_url(environment)?,
            Key::RepoVersionedWebUrl => {
                let base_repo_web_url = self.retrieve(environment, Key::RepoWebUrl)?;
                let version = self.retrieve(environment, Key::Version)?;
                let commit_sha = sha(environment)?;

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

impl fmt::Display for VarSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::any::type_name::<VarSource>())
    }
}
