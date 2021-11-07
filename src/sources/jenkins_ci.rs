// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::environment::Environment;
use crate::var::Key;
use crate::var::C_HIGH;

use super::var;
use super::Hierarchy;
use super::RetrieveRes;

/// This sources values from the environment variables set by the CI provider Jenkins.
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
                | Key::BuildDate
                | Key::BuildHostingUrl
                | Key::BuildOs
                | Key::BuildOsFamily
                | Key::BuildTag
                | Key::Ci
                | Key::License
                | Key::Licenses
                | Key::NameMachineReadable
                | Key::RepoCloneUrl
                | Key::RepoCloneUrlSsh
                | Key::RepoCommitPrefixUrl
                | Key::RepoIssuesUrl
                | Key::RepoRawVersionedPrefixUrl
                | Key::RepoVersionedDirPrefixUrl
                | Key::RepoVersionedFilePrefixUrl
                | Key::RepoWebUrl
                | Key::VersionDate => None,
                Key::BuildBranch => var(environment, "BRANCH_NAME", C_HIGH),
                Key::BuildNumber => var(environment, "BUILD_NUMBER", C_HIGH),
                Key::Name => var(environment, "APP_NAME", C_HIGH),
                Key::Version => var(environment, "VERSION", C_HIGH), // Alternatively (but makes no sense to use): var(environment, "PULL_BASE_SHA")
            },
        )
    }
}
