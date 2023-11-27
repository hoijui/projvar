// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::environment::Environment;
use crate::var::Key;
use crate::var::C_HIGH;

use super::var;
use super::Hierarchy;
use super::RetrieveRes;

/// This sources values from the environment variables
/// set by the CI provider Travis,
/// which was the go-to CI for Github projects
/// until Github Actions were introduced in October 2018.
pub struct VarSource;

impl super::VarSource for VarSource {
    fn is_usable(&self, _environment: &mut Environment) -> bool {
        true
    }

    fn hierarchy(&self) -> Hierarchy {
        Hierarchy::High
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
                | Key::BuildHostingUrl
                | Key::BuildDate
                | Key::BuildOsFamily
                | Key::Ci
                | Key::Licenses
                | Key::License
                | Key::NameMachineReadable
                | Key::RepoIssuesUrl
                | Key::RepoCloneUrl
                | Key::RepoCloneUrlGit
                | Key::RepoCloneUrlHttp
                | Key::RepoCloneUrlSsh
                | Key::RepoCommitPrefixUrl
                | Key::RepoRawVersionedPrefixUrl
                | Key::RepoVersionedDirPrefixUrl
                | Key::RepoVersionedFilePrefixUrl
                | Key::RepoWebUrl
                | Key::VersionDate => None,
                Key::BuildBranch => var(environment, "TRAVIS_BRANCH", C_HIGH),
                Key::BuildNumber => var(environment, "TRAVIS_BUILD_NUMBER", C_HIGH),
                Key::BuildOs => var(environment, "TRAVIS_OS_NAME", C_HIGH),
                Key::BuildTag => var(environment, "TRAVIS_TAG", C_HIGH),
                Key::Name => crate::value_conversions::slug_to_proj_name(
                    environment.vars.get("TRAVIS_REPO_SLUG"),
                )?
                .map(|val| (C_HIGH, val)), // usually: TRAVIS_REPO_SLUG="user/project"
                Key::Version => self
                    .version_from_build_tag(environment, key)?
                    .or_else(|| var(environment, "TRAVIS_COMMIT", C_HIGH)),
            },
        )
    }
}
