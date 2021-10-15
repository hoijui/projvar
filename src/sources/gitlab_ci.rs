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
                | Key::BuildNumber
                | Key::BuildOsFamily
                | Key::License
                | Key::Licenses => None,
                Key::BuildBranch => var(environment, "CI_COMMIT_BRANCH"),
                Key::BuildHostingUrl => var(environment, "CI_PAGES_URL"),
                Key::BuildOs => var(environment, "CI_RUNNER_EXECUTABLE_ARCH"), // TODO Not sure if this makes sense ... have to check in practise!
                Key::BuildTag => var(environment, "CI_COMMIT_TAG"),
                Key::Ci => var(environment, "CI"),
                Key::Name => var(environment, "CI_PROJECT_NAME"),
                Key::NameMachineReadable => {
                    super::try_construct_machine_readable_name_from_web_url(self, environment)?
                }
                Key::RepoCloneUrl => var(environment, "CI_REPOSITORY_URL"),
                Key::RepoCommitPrefixUrl => {
                    super::try_construct_commit_prefix_url(self, environment)?
                }
                Key::RepoIssuesUrl => super::try_construct_issues_url(self, environment)?,
                Key::RepoRawVersionedPrefixUrl => {
                    super::try_construct_raw_prefix_url(self, environment)?
                }
                Key::RepoVersionedDirPrefixUrl => {
                    super::try_construct_dir_prefix_url(self, environment)?
                }
                Key::RepoVersionedFilePrefixUrl => {
                    super::try_construct_file_prefix_url(self, environment)?
                }
                Key::RepoWebUrl => var(environment, "CI_PROJECT_URL"),
                Key::Version => self
                    .retrieve(environment, Key::BuildTag)?
                    .or_else(|| var(environment, "CI_COMMIT_SHORT_SHA")),
                Key::VersionDate => var(environment, "CI_COMMIT_TIMESTAMP"), // TODO This probably has to be converted/formatted
            },
        )
    }
}
