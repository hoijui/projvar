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
            Key::Name => var(environment, "APP_NAME"),
            Key::RepoFrozenWebUrl => super::try_construct_frozen(self, environment)?,
            Key::BuildBranch => var(environment, "BRANCH_NAME"),
            Key::BuildIdent => var(environment, "PULL_BASE_SHA"),
            Key::Version => var(environment, "VERSION"),
            Key::BuildNumber => var(environment, "BUILD_NUMBER"),
            Key::RepoIssuesUrl => None,
            Key::RepoWebUrl
            | Key::Ci
            | Key::BuildTag
            | Key::RepoCloneUrl
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

impl fmt::Display for VarSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::any::type_name::<Self>())
    }
}
