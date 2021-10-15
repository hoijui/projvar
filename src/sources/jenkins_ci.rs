// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::environment::Environment;
use crate::var::Key;
use std::error::Error;

use super::var;
use super::Hierarchy;

pub struct VarSource;

type BoxResult<T> = Result<T, Box<dyn Error>>;

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
    fn retrieve(&self, environment: &mut Environment, key: Key) -> BoxResult<Option<String>> {
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
                | Key::RepoCloneUrl
                | Key::RepoCommitPrefixUrl
                | Key::RepoIssuesUrl
                | Key::RepoRawVersionedPrefixUrl
                | Key::RepoVersionedDirPrefixUrl
                | Key::RepoVersionedFilePrefixUrl
                | Key::RepoWebUrl
                | Key::VersionDate => None,
                Key::BuildBranch => var(environment, "BRANCH_NAME"),
                Key::BuildNumber => var(environment, "BUILD_NUMBER"),
                Key::Name => var(environment, "APP_NAME"),
                Key::NameMachineReadable => {
                    super::try_construct_machine_readable_name_from_name(self, environment)?
                }
                Key::Version => var(environment, "VERSION"), // Alternatively (but makes no sense to use): var(environment, "PULL_BASE_SHA")
            },
        )
    }
}
