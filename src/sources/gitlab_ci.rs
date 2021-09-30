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
            Key::Name => var(environment, "CI_PROJECT_NAME"),
            Key::RepoWebUrl => var(environment, "CI_PROJECT_URL"),
            Key::RepoVersionedWebUrl => super::try_construct_versioned(self, environment)?,
            Key::RepoIssuesUrl => super::try_construct_issues_url(self, environment)?,
            Key::Ci => var(environment, "CI"),
            Key::BuildBranch => var(environment, "CI_COMMIT_BRANCH"),
            Key::BuildTag => var(environment, "CI_COMMIT_TAG"),
            Key::RepoCloneUrl => var(environment, "CI_REPOSITORY_URL"),
            Key::BuildHostingUrl => var(environment, "CI_PAGES_URL"),
            Key::BuildOs => var(environment, "CI_RUNNER_EXECUTABLE_ARCH"), // TODO Not sure if this makes sense ... have to check in practise!
            Key::VersionDate => var(environment, "CI_COMMIT_TIMESTAMP"), // TODO This probably has to be converted/formatted
            Key::Version => self
                .retrieve(environment, Key::BuildTag)?
                .or_else(|| var(environment, "CI_COMMIT_SHORT_SHA")),
            Key::BuildDate
            | Key::BuildOsFamily
            | Key::BuildArch
            | Key::License
            | Key::BuildNumber => None,
        })
    }
}

impl fmt::Display for VarSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::any::type_name::<VarSource>())
    }
}
