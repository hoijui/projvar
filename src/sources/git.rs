// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::var::Key;
use crate::{environment::Environment, value_conversions};

use super::{Hierarchy, RetrieveRes};
pub struct VarSource;

fn version(environment: &mut Environment) -> RetrieveRes {
    Ok(match environment.repo() {
        Some(repo) => {
            let sc_version = repo.version().or_else(|err| {
                log::warn!("Failed to git describe (\"{}\"), using SHA instead", err);
                repo.sha()
                    .and_then(|v| v.ok_or_else(|| "No SHA available to serve as version".into()))
            })?;
            Some(sc_version)
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
            repo.branch()?
        }
        None => None,
    })
}

fn tag(environment: &mut Environment) -> RetrieveRes {
    Ok(match environment.repo() {
        Some(repo) => repo.tag()?,
        None => None,
    })
}

fn clone_url(environment: &mut Environment) -> RetrieveRes {
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

fn version_date(environment: &mut Environment) -> RetrieveRes {
    let date_format = &environment.settings.date_format;
    Ok(match environment.repo() {
        Some(repo) => Some(repo.commit_date(date_format)?),
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
                | Key::RepoCommitPrefixUrl
                | Key::RepoIssuesUrl
                | Key::RepoRawVersionedPrefixUrl
                | Key::RepoVersionedDirPrefixUrl
                | Key::RepoVersionedFilePrefixUrl
                | Key::RepoWebUrl => None,
                Key::BuildBranch => branch(environment)?,
                Key::BuildTag => tag(environment)?,
                Key::RepoCloneUrl => value_conversions::clone_url_conversion_option(
                    clone_url(environment)?.as_ref(),
                    value_conversions::Protocol::Https,
                )?,
                Key::RepoCloneUrlSsh => value_conversions::clone_url_conversion_option(
                    clone_url(environment)?.as_ref(),
                    value_conversions::Protocol::Ssh,
                )?,
                Key::Version => version(environment)?,
                Key::VersionDate => version_date(environment)?,
            },
        )
    }
}
