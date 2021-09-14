// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::environment::Environment;
use crate::var::Key;
use std::error::Error;
use std::fmt;

use super::var;

pub struct VarSource;

type BoxResult<T> = Result<T, Box<dyn Error>>;

impl super::VarSource for VarSource {
    fn is_usable(&self, _environment: &mut Environment) -> bool {
        true
    }

    fn retrieve(&self, environment: &mut Environment, key: Key) -> BoxResult<Option<String>> {
        Ok(match key {
            Key::RepoFrozenWebUrl => super::try_construct_frozen(self, environment)?,
            Key::BuildBranch => var(environment, "TRAVIS_BRANCH"),
            Key::BuildTag => var(environment, "TRAVIS_TAG"),
            Key::BuildOs => var(environment, "TRAVIS_OS_NAME"),
            Key::BuildIdent => var(environment, "TRAVIS_COMMIT"),
            Key::BuildNumber => var(environment, "TRAVIS_BUILD_NUMBER"),
            Key::Name => super::proj_name_from_slug(environment.vars.get("TRAVIS_REPO_SLUG"))?, // usually: TRAVIS_REPO_SLUG="user/project"
            Key::RepoWebUrl
            | Key::Ci
            | Key::RepoCloneUrl
            | Key::BuildHostingUrl
            | Key::VersionDate
            | Key::Version
            | Key::BuildDate
            | Key::BuildOsFamily
            | Key::BuildArch
            | Key::License => None,
        })
    }
}

impl fmt::Display for VarSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::any::type_name::<VarSource>())
    }
}
