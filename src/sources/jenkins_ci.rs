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

    fn retrieve(&self, environment: &mut Environment, key: Key) -> BoxResult<Option<String>> {
        Ok(match key {
            Key::Name => var(environment, "APP_NAME"),
            Key::RepoVersionedWebUrl => super::try_construct_versioned(self, environment)?,
            Key::BuildBranch => var(environment, "BRANCH_NAME"),
            Key::Version => var(environment, "VERSION"), // Alternatively (but makes no sense to use): var(environment, "PULL_BASE_SHA")
            Key::BuildNumber => var(environment, "BUILD_NUMBER"),
            Key::RepoIssuesUrl
            | Key::RepoWebUrl
            | Key::Ci
            | Key::BuildTag
            | Key::RepoCloneUrl
            | Key::RepoRawVersionedPrefixUrl
            | Key::RepoVersionedFilePrefixUrl
            | Key::RepoVersionedDirPrefixUrl
            | Key::BuildHostingUrl
            | Key::BuildOs
            | Key::VersionDate
            | Key::BuildDate
            | Key::BuildOsFamily
            | Key::BuildArch
            | Key::License => None,
        })
    }
}
