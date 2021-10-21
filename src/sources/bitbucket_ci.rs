// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::environment::Environment;
use crate::var::Key;

use super::var;
use super::Hierarchy;
use super::RetrieveRes;

pub struct VarSource;

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
                | Key::BuildHostingUrl
                | Key::BuildDate
                | Key::BuildOs
                | Key::BuildOsFamily
                | Key::Licenses
                | Key::License
                | Key::VersionDate
                | Key::NameMachineReadable
                | Key::RepoCommitPrefixUrl
                | Key::RepoIssuesUrl
                | Key::RepoRawVersionedPrefixUrl
                | Key::RepoVersionedDirPrefixUrl
                | Key::RepoVersionedFilePrefixUrl => None,
                Key::BuildBranch => var(environment, "BITBUCKET_BRANCH"),
                Key::BuildNumber => var(environment, "BITBUCKET_BUILD_NUMBER"),
                Key::BuildTag => var(environment, "BITBUCKET_TAG"),
                Key::Ci => var(environment, "CI"),
                Key::Name => var(environment, "BITBUCKET_PROJECT_KEY"),
                Key::RepoCloneUrl => var(environment, "BITBUCKET_GIT_HTTP_ORIGIN"),
                Key::RepoCloneUrlSsh => var(environment, "BITBUCKET_GIT_SSH_ORIGIN"),
                Key::RepoWebUrl => {
                    // BITBUCKET_REPO_FULL_NAME = The full name of the repository
                    // (everything that comes after http://bitbucket.org/).
                    var(environment, "BITBUCKET_REPO_FULL_NAME")
                        .map(|project_slug| format!("http://bitbucket.org/{}", project_slug))
                }
                Key::Version => var(environment, "BITBUCKET_COMMIT"),
            },
        )
    }
}
