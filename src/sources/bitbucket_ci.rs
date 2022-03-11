// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::cleanup;
use crate::environment::Environment;
use crate::validator;
use crate::var::Key;
use crate::var::C_HIGH;
use crate::var::C_LOW;

use super::var;
use super::Hierarchy;
use super::RetrieveRes;

/// This sources values from the environment variables set by the CI in
/// [`crate::tools::git_hosting_provs::HostingType::BitBucket`].
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
                | Key::BuildHostingUrl
                | Key::BuildDate
                | Key::BuildOs
                | Key::BuildOsFamily
                | Key::Licenses
                | Key::License
                | Key::VersionDate
                | Key::NameMachineReadable
                | Key::RepoCloneUrlSsh
                | Key::RepoCommitPrefixUrl
                | Key::RepoIssuesUrl
                | Key::RepoRawVersionedPrefixUrl
                | Key::RepoVersionedDirPrefixUrl
                | Key::RepoVersionedFilePrefixUrl => None,
                Key::BuildBranch => var(environment, "BITBUCKET_BRANCH", C_HIGH),
                Key::BuildNumber => var(environment, "BITBUCKET_BUILD_NUMBER", C_HIGH),
                Key::BuildTag => var(environment, "BITBUCKET_TAG", C_HIGH),
                Key::Ci => {
                    var(environment, "CI", C_HIGH).or_else(|| Some((C_LOW, "false".to_owned())))
                }
                Key::Name => var(environment, "BITBUCKET_PROJECT_KEY", C_HIGH),
                Key::RepoCloneUrl => var(environment, "BITBUCKET_GIT_SSH_ORIGIN", C_HIGH), // NOTE This actually contains the common SSH idnetifier type "URL", which is not a valid URL, Thus we use it here, not for RepoCloneUrlSsh
                Key::RepoCloneUrlHttp => var(environment, "BITBUCKET_GIT_HTTP_ORIGIN", C_HIGH),
                Key::RepoWebUrl => {
                    // BITBUCKET_REPO_FULL_NAME = The full name of the repository
                    // (everything that comes after http://bitbucket.org/).
                    var(environment, "BITBUCKET_REPO_FULL_NAME", C_HIGH).map(
                        |(confidence, project_slug)| {
                            (confidence, format!("http://bitbucket.org/{}", project_slug))
                        },
                    ) // TODO Maybe use a constant here? (for "http://bitbucket.org")
                }
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
                    .or_else(|| var(environment, "BITBUCKET_COMMIT", C_HIGH)),
            },
        )
    }
}
