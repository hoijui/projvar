// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::environment::Environment;
use crate::value_conversions::slug_to_proj_name;
use crate::var::Confidence;
use crate::var::Key;
use crate::var::C_HIGH;
use crate::var::C_LOW;

use super::var;
use super::Hierarchy;
use super::RetrieveRes;

/// This sources values from the environment variables set by the CI in
/// [`crate::tools::git_hosting_provs::HostingType::GitHub`].
pub struct VarSource;

fn build_branch(environment: &mut Environment) -> RetrieveRes {
    let refr = var(environment, "GITHUB_REF", C_HIGH);
    Ok(if let Some(refr) = refr {
        super::ref_extract_branch(&refr.1)?
    } else {
        None
    })
}

fn build_tag(environment: &mut Environment) -> RetrieveRes {
    let refr = var(environment, "GITHUB_REF", C_HIGH);
    Ok(if let Some(refr) = refr {
        super::ref_extract_tag(&refr.1)?
    } else {
        None
    })
}

fn repo_web_url(environment: &mut Environment) -> Option<(Confidence, String)> {
    match (
        environment.vars.get("GITHUB_SERVER_URL"),
        environment.vars.get("GITHUB_REPOSITORY"),
    ) {
        (Some(server), Some(repo)) => {
            // "${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}"
            // usually:
            // GITHUB_SERVER_URL="https://github.com/"
            // GITHUB_REPOSITORY="user/project"
            Some((C_HIGH, format!("{}/{}", server, repo)))
        }
        (_, _) => None,
    }
}

impl super::VarSource for VarSource {
    fn is_usable(&self, _environment: &mut Environment) -> bool {
        true
    }

    fn hierarchy(&self) -> Hierarchy {
        Hierarchy::High
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
                | Key::BuildHostingUrl
                | Key::BuildNumber
                | Key::BuildOsFamily
                | Key::License
                | Key::Licenses
                | Key::VersionDate
                | Key::NameMachineReadable
                | Key::RepoCommitPrefixUrl
                | Key::RepoCloneUrl
                | Key::RepoCloneUrlSsh
                | Key::RepoIssuesUrl
                | Key::RepoRawVersionedPrefixUrl
                | Key::RepoVersionedDirPrefixUrl
                | Key::RepoVersionedFilePrefixUrl => None,
                Key::BuildBranch => build_branch(environment)?,
                Key::BuildOs => var(environment, "RUNNER_OS", C_LOW), // TODO PRIO Not sure if this makes sense ... have to check in practise, and probably map values to our set of accepted values!
                Key::BuildTag => build_tag(environment)?,
                Key::Ci => {
                    var(environment, "CI", C_HIGH).or_else(|| Some((C_LOW, "false".to_owned())))
                }
                Key::Name => match var(environment, "GITHUB_REPOSITORY", C_HIGH) {
                    Some(rated_val) => {
                        slug_to_proj_name(Some(&rated_val.1))?.map(|val| (rated_val.0, val))
                    }
                    None => None,
                }, // usually: GITHUB_REPOSITORY="user/project"
                Key::RepoWebUrl => repo_web_url(environment),
                Key::Version => var(environment, "GITHUB_SHA", C_LOW),
            },
        )
    }
}
