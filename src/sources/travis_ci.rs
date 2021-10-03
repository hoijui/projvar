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
            Key::RepoVersionedWebUrl => super::try_construct_versioned(self, environment)?,
            Key::BuildBranch => var(environment, "TRAVIS_BRANCH"),
            Key::BuildTag => var(environment, "TRAVIS_TAG"),
            Key::BuildOs => var(environment, "TRAVIS_OS_NAME"),
            Key::Version => var(environment, "TRAVIS_COMMIT"),
            Key::BuildNumber => var(environment, "TRAVIS_BUILD_NUMBER"),
            Key::Name => super::proj_name_from_slug(environment.vars.get("TRAVIS_REPO_SLUG"))?, // usually: TRAVIS_REPO_SLUG="user/project"
            Key::RepoIssuesUrl
            | Key::RepoWebUrl
            | Key::Ci
            | Key::RepoCloneUrl
            | Key::RepoRawVersionedPrefixUrl
            | Key::RepoVersionedFilePrefixUrl
            | Key::RepoVersionedDirPrefixUrl
            | Key::BuildHostingUrl
            | Key::VersionDate
            | Key::BuildDate
            | Key::BuildOsFamily
            | Key::BuildArch
            | Key::License => None,
        })
    }
}
