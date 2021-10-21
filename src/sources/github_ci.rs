// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::environment::Environment;
use crate::value_conversions::slug_to_proj_name;
use crate::var::Key;

use super::var;
use super::Hierarchy;
use super::RetrieveRes;
pub struct VarSource;

// TODO Move this elsewhere
fn is_branch(environment: &mut Environment, refr: &str) -> RetrieveRes {
    let mut branch = None;
    if let Some(repo) = environment.repo() {
        let checked_out_branch = repo.branch()?;
        if let Some(checked_out_branch) = checked_out_branch {
            if refr.ends_with(&format!("/{}", &checked_out_branch)) {
                branch = Some(refr);
            }
        }
    }
    Ok(branch.map(std::borrow::ToOwned::to_owned))
}

// TODO Move this elsewhere
fn is_tag(environment: &mut Environment, refr: &str) -> RetrieveRes {
    let mut tag = None;
    if let Some(repo) = environment.repo() {
        let checked_out_branch = repo.tag()?;
        if let Some(checked_out_branch) = checked_out_branch {
            if refr.ends_with(&format!("/{}", &checked_out_branch)) {
                tag = Some(refr);
            }
        }
    }
    Ok(tag.map(std::borrow::ToOwned::to_owned))
}

fn build_branch(environment: &mut Environment) -> RetrieveRes {
    let refr = var(environment, "GITHUB_REF");
    Ok(if let Some(refr) = refr {
        is_branch(environment, &refr)?
    } else {
        None
    })
}

fn build_tag(environment: &mut Environment) -> RetrieveRes {
    let refr = var(environment, "GITHUB_REF");
    Ok(if let Some(refr) = refr {
        is_tag(environment, &refr)?
    } else {
        None
    })
}

fn repo_web_url(environment: &mut Environment) -> Option<String> {
    match (
        environment.vars.get("GITHUB_SERVER_URL"),
        environment.vars.get("GITHUB_REPOSITORY"),
    ) {
        (Some(server), Some(repo)) => {
            // "${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}"
            // usually:
            // GITHUB_SERVER_URL="https://github.com/"
            // GITHUB_REPOSITORY="user/project"
            Some(format!("{}/{}", server, repo))
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
                Key::BuildOs => var(environment, "RUNNER_OS"), // TODO Not sure if this makes sense ... have to check in practise!
                Key::BuildTag => build_tag(environment)?,
                Key::Ci => var(environment, "CI"),
                Key::Name => slug_to_proj_name(environment.vars.get("GITHUB_REPOSITORY"))?, // usually: GITHUB_REPOSITORY="user/project"
                Key::RepoWebUrl => repo_web_url(environment),
                Key::Version => var(environment, "GITHUB_SHA"),
            },
        )
    }
}
