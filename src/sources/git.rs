// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::environment::Environment;
use crate::var::{Key, C_HIGH};

use super::{Hierarchy, RetrieveRes};

/// Sources values by querrying the `git` CLI tool.
/// In reality, we use a git library, but the effect is the same.
pub struct VarSource;

fn version(environment: &mut Environment) -> RetrieveRes {
    Ok(match environment.repo() {
        Some(repo) => {
            let sc_version = repo.version().or_else(|err| {
                log::warn!("Failed to git describe (\"{err}\"), using SHA instead");
                repo.sha()
                    .and_then(|v| v.ok_or_else(|| "No SHA available to serve as version".into()))
            })?;
            Some((C_HIGH, sc_version))
        }
        None => None,
    })
}

fn branch(environment: &mut Environment) -> RetrieveRes {
    Ok(match environment.repo() {
        Some(repo) => {
            // Ok(repo.branch().unwrap_or_else(|err| {
            //     log::warn!("Failed fetching git branch - {}", err);
            //     None
            // }))
            repo.branch()?.map(|val| (C_HIGH, val))
        }
        None => None,
    })
}

fn tag(environment: &mut Environment) -> RetrieveRes {
    Ok(match environment.repo() {
        Some(repo) => repo.tag()?.map(|val| (C_HIGH, val)),
        None => None,
    })
}

fn clone_url(environment: &mut Environment) -> RetrieveRes {
    Ok(match environment.repo() {
        Some(repo) => repo
            .remote_clone_url()?
            .map(|remote_clone_url| (C_HIGH, remote_clone_url)),
        None => None,
    })
}

fn version_date(environment: &mut Environment) -> RetrieveRes {
    let date_format = environment.settings.date_format.clone();
    Ok(match &environment.repo() {
        Some(repo) => Some((C_HIGH, repo.commit_date(&date_format)?)),
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
                Key::BuildArch
                | Key::BuildDate
                | Key::BuildNumber
                | Key::BuildOs
                | Key::BuildOsFamily
                | Key::Ci
                | Key::License
                | Key::Licenses
                | Key::BuildHostingUrl
                | Key::Name
                | Key::NameMachineReadable
                | Key::RepoCloneUrlGit
                | Key::RepoCloneUrlHttp
                | Key::RepoCloneUrlSsh
                | Key::RepoCommitPrefixUrl
                | Key::RepoIssuesUrl
                | Key::RepoRawVersionedPrefixUrl
                | Key::RepoVersionedDirPrefixUrl
                | Key::RepoVersionedFilePrefixUrl
                | Key::RepoWebUrl => None,
                Key::BuildBranch => branch(environment)?,
                Key::BuildTag => tag(environment)?,
                Key::RepoCloneUrl => clone_url(environment)?
                    .map(|rated_value| rated_value.1)
                    .map(|val| (C_HIGH, val)),
                Key::Version => version(environment)?,
                Key::VersionDate => version_date(environment)?,
            },
        )
    }
}
