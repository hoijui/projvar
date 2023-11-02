// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::cleanup;
use crate::environment::Environment;
use crate::validator;
use crate::value_conversions;
use crate::var::Key;
use crate::var::C_HIGH;
use crate::var::C_LOW;

use super::var;
use super::Hierarchy;
use super::RetrieveRes;

/// This sources values from the environment variables set by the CI in
/// [`crate::tools::git_hosting_provs::HostingType::GitLab`].
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
                | Key::BuildNumber
                | Key::BuildOsFamily
                | Key::License
                | Key::Licenses
                | Key::NameMachineReadable
                | Key::RepoCloneUrlGit
                | Key::RepoCloneUrlHttp
                | Key::RepoCloneUrlSsh
                | Key::RepoCommitPrefixUrl
                | Key::RepoIssuesUrl
                | Key::RepoRawVersionedPrefixUrl
                | Key::RepoVersionedDirPrefixUrl
                | Key::RepoVersionedFilePrefixUrl => None,
                Key::BuildBranch => var(environment, "CI_COMMIT_BRANCH", C_HIGH),
                Key::BuildHostingUrl => var(environment, "CI_PAGES_URL", C_HIGH),
                Key::BuildOs => var(environment, "CI_RUNNER_EXECUTABLE_ARCH", C_LOW), // TODO Not sure if this makes sense ... have to check in practise!
                Key::BuildTag => var(environment, "CI_COMMIT_TAG", C_HIGH),
                Key::Ci => {
                    var(environment, "CI", C_HIGH).or_else(|| Some((C_LOW, "false".to_owned())))
                }
                Key::Name => var(environment, "CI_PROJECT_NAME", C_HIGH),
                // TODO PRIO make sure to cover/handle well all of this (default format of this env var): CI_REPOSITORY_URL="https://gitlab-ci-token:[masked]@example.com/gitlab-org/gitlab-foss.git"
                Key::RepoCloneUrl => var(environment, "CI_REPOSITORY_URL", C_HIGH),
                Key::RepoWebUrl => var(environment, "CI_PROJECT_URL", C_HIGH),
                Key::Version => self
                    .retrieve(environment, Key::BuildTag)?
                    .map(|conf_val| cleanup::conf_version(environment, conf_val))
                    .filter(|conf_val| {
                        if let Ok(validity) = validator::get(key)(environment, &conf_val.1) {
                            validity.is_good()
                        } else {
                            false
                        }
                    })
                    .or_else(|| var(environment, "CI_COMMIT_SHORT_SHA", C_HIGH)),
                Key::VersionDate => {
                    // This comes in the ISO 8601 time format
                    let gitlab_commit_date = var(environment, "CI_COMMIT_TIMESTAMP", C_HIGH);
                    if let Some((confidence, val)) = gitlab_commit_date {
                        value_conversions::date_iso8601_to_our_format(environment, &val)?
                            .map(|out_date| (confidence, out_date))
                    } else {
                        None
                    }
                }
            },
        )
    }
}
