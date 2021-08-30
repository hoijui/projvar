// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// use log;

// use std::env;
use std::fmt::Display;
use std::{collections::HashMap, error::Error};

use crate::storage::{self, Storage};

type BoxResult<T> = Result<T, Box<dyn Error>>;

// #[derive(Clone)]
// #[derive(Debug)]
pub struct Variable {
    pub key: &'static str,
    pub description: &'static str,
    // pub alt_keys: Vec<&'static str>,
    pub alt_keys: &'static [&'static str],
}

impl Variable {
    /// Returns the value of the first variable that has a value associated,
    /// starting with the primary one (`key`),
    /// and continuing with `alt_keys` in order.
    ///
    /// # Errors
    ///
    /// If any error occures accessing the underlying variable storage.
    pub fn fetch_any_var(&self, storage: &dyn Storage) -> BoxResult<Option<String>> {
        let mut value = storage.get(self.key);
        if value.is_err() {
            for key in self.alt_keys {
                value = storage.get(key);
                if value.is_ok() {
                    break;
                }
            }
        }
        match value {
            Ok(value) => Ok(value),
            Err(err) => Err(Box::new(err)),
        }
    }

    /// Sets the main variable (`key`) to a specific value.
    ///
    /// # Errors
    ///
    /// If any error occures accessing the underlying variable storage.
    pub fn set_main(&self, storage: &mut dyn Storage, val: &str) -> storage::Result<()> {
        storage.set(self.key, val)
    }

    /// Sets the main variable (`key`) and all the others (`alt_keys`)
    /// to a specific value.
    ///
    /// # Errors
    ///
    /// If any error occures accessing the underlying variable storage.
    pub fn set_all(&self, storage: &mut dyn Storage, val: &str) -> storage::Result<()> {
        self.set_main(storage, val)?;
        for key in self.alt_keys {
            storage.set(key, val)?;
        }
        Ok(())
    }
}

impl Display for Variable {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        write!(formatter, "{}", self.key)?;
        Ok(())
    }
}

pub fn list(alt_keys: bool) {
    for var in (*VARS).values() {
        if alt_keys {
            log::info!(
                "{} {} - {}",
                var.key,
                var.alt_keys.join(" "),
                var.description
            );
        } else {
            log::info!("{} - {}", var.key, var.description);
        }
    }
}

pub const KEY_VERSION: &str = "PROJECT_VERSION";
pub const KEY_LICENSE: &str = "PROJECT_LICENSE";
pub const KEY_REPO_WEB_URL: &str = "PROJECT_REPO_WEB_URL";
pub const KEY_REPO_FROZEN_WEB_URL: &str = "BUILD_REPO_FROZEN_WEB_URL";
pub const KEY_REPO_CLONE_URL: &str = "PROJECT_REPO_CLONE_URL";
pub const KEY_NAME: &str = "PROJECT_NAME";
pub const KEY_VERSION_DATE: &str = "PROJECT_VERSION_DATE";
pub const KEY_BUILD_DATE: &str = "BUILD_DATE";
pub const KEY_BUILD_BRANCH: &str = "BUILD_BRANCH";
pub const KEY_BUILD_TAG: &str = "BUILD_TAG";
pub const KEY_BUILD_IDENT: &str = "BUILD_IDENT"; // TODO This name is very bad, as it makes one think of BUILD_NUMBER; choose a different one!
pub const KEY_BUILD_OS: &str = "BUILD_OS";
pub const KEY_BUILD_OS_FAMILY: &str = "BUILD_OS_FAMILY";
pub const KEY_BUILD_ARCH: &str = "BUILD_ARCH";
pub const KEY_BUILD_HOSTING_URL: &str = "BUILD_HOSTING_URL";
pub const KEY_BUILD_NUMBER: &str = "BUILD_NUMBER";
pub const KEY_CI: &str = "CI";

// pub const VARS: &[Variable] = &[
//     Variable {
//         key: KEY_VERSION,
//         description: "The project version.",
//         alt_keys: &["VERSION", "CI_COMMIT_SHORT_SHA"],
//     },
//     Variable {
//         key: KEY_LICENSE,
//         description: "Main License of the sources.",
//         alt_keys: &["LICENSE"],
//     },
//     Variable {
//         key: KEY_REPO_WEB_URL,
//         description: "The Repo web UI URL.",
//         alt_keys: &[
//             "REPO_WEB_URL",
//             "REPO",
//             "CI_PROJECT_URL",
//             "BITBUCKET_GIT_HTTP_ORIGIN",
//         ],
//     },
//     Variable {
//         key: KEY_REPO_FROZEN_WEB_URL,
//         description: "The Repo web UI URL, pointing to the specific version of this build.",
//         alt_keys: &[
//             "FROZEN_WEB_URL",
//             "COMMIT_URL",
//         ],
//     },
//     Variable {
//         key: KEY_REPO_CLONE_URL,
//         description: "The Repo clone URL.",
//         alt_keys: &[
//             "REPO_CLONE_URL",
//             "CLONE_URL",
//             "CI_REPOSITORY_URL",
//             "BITBUCKET_GIT_SSH_ORIGIN",
//         ],
//     },
//     Variable {
//         key: KEY_NAME,
//         description: "The name of the project.",
//         alt_keys: &[
//             "NAME",
//             "CI_PROJECT_NAME",
//             "APP_NAME",
//             "BITBUCKET_PROJECT_KEY",
//         ],
//     },
//     Variable {
//         key: KEY_VERSION_DATE,
//         description: "Date this version was committed to source control. ['%Y-%m-%d']",
//         alt_keys: &[
//             "VERSION_DATE",
//             "DATE",
//             "COMMIT_DATE",
//             "PROJECT_COMMIT_DATE",
//             "CI_COMMIT_TIMESTAMP",
//         ],
//     },
//     Variable {
//         key: KEY_BUILD_DATE,
//         description: "Date of this build. ['%Y-%m-%d']",
//         alt_keys: &[],
//     },
//     Variable {
//         key: KEY_BUILD_BRANCH,
//         description: "The development branch name.",
//         alt_keys: &[
//             "BRANCH",
//             "GITHUB_REF",
//             "CI_COMMIT_BRANCH",
//             "BRANCH_NAME",
//             "BITBUCKET_BRANCH",
//             "TRAVIS_BRANCH",
//         ],
//     },
//     Variable {
//         key: KEY_BUILD_TAG,
//         description: "The tag of a commit that kicked off the build. This value is only available on tags. Not available for builds against branches.",
//         alt_keys: &[
//             "TAG",
//             "GITHUB_REF",
//             "CI_COMMIT_TAG",
//             "BITBUCKET_TAG",
//             "TRAVIS_TAG",
//         ],
//     },
//     Variable {
//         key: KEY_BUILD_IDENT,
//         description: "Unique identifier of the state of the project that is being built (e.g. git commit SHA).",
//         alt_keys: &[
//             "GITHUB_SHA",
//             "CI_COMMIT_SHA",
//             "PULL_BASE_SHA",
//             "BITBUCKET_COMMIT",
//             "TRAVIS_COMMIT",
//         ],
//     },
//     Variable {
//         key: KEY_BUILD_OS,
//         description: "Operating system we are building on. (common values: 'linux', 'macos', 'windows')",
//         alt_keys: &[
//             "OS",
//             "RUNNER_OS",
//             "CI_RUNNER_EXECUTABLE_ARCH",
//             "TRAVIS_OS_NAME",
//         ],
//     },
//     Variable {
//         key: KEY_BUILD_OS_FAMILY,
//         description: "Operating system family we are building on. (should be either 'unix' or 'windows')",
//         alt_keys: &[
//             "OS_FAMILY",
//             "FAMILY",
//         ],
//     },
//     Variable {
//         key: KEY_BUILD_ARCH,
//         description: "Computer hardware architecture we are building on. (common values: 'x86', 'x86_64')",
//         alt_keys: &[
//             "ARCH",
//         ],
//     },
//     Variable {
//         key: KEY_BUILD_HOSTING_URL,
//         description: "Web URL under which the generated output will be available.",
//         alt_keys: &["HOSTING_URL", "CI_PAGES_URL"],
//     },
//     Variable {
//         key: KEY_BUILD_NUMBER,
//         description: "The build number (1, 2, 3) starts at 1 for each repo and branch.",
//         alt_keys: &[
//             "NUMBER",
//             "ID",
//             "BUILD_ID",
//             "BITBUCKET_BUILD_NUMBER",
//             "TRAVIS_BUILD_NUMBER",
//         ],
//     },
//     Variable {
//         key: KEY_CI,
//         description: "'true' if running on a CI/build-bot.",
//         alt_keys: &[],
//     },
// ];

use clap::lazy_static::lazy_static;
use maplit::hashmap;

lazy_static! {
// pub const VARS: HashMap<&'static str, Variable> = hashmap! {
pub static ref VARS: HashMap<&'static str, Variable> = hashmap! {
    KEY_VERSION => Variable {
        key: KEY_VERSION,
        description: "The project version.",
        alt_keys: &["VERSION", "CI_COMMIT_SHORT_SHA"],
    },
    KEY_LICENSE => Variable {
        key: KEY_LICENSE,
        description: "Main License of the sources.",
        alt_keys: &["LICENSE"],
    },
    KEY_REPO_WEB_URL => Variable {
        key: KEY_REPO_WEB_URL,
        description: "The Repo web UI URL.",
        alt_keys: &[
            "REPO_WEB_URL",
            "REPO",
            "CI_PROJECT_URL",
            "BITBUCKET_GIT_HTTP_ORIGIN",
        ],
    },
    KEY_REPO_FROZEN_WEB_URL => Variable {
        key: KEY_REPO_FROZEN_WEB_URL,
        description: "The Repo web UI URL, pointing to the specific version of this build.",
        alt_keys: &[
            "FROZEN_WEB_URL",
            "COMMIT_URL",
        ],
    },
    KEY_REPO_CLONE_URL => Variable {
        key: KEY_REPO_CLONE_URL,
        description: "The Repo clone URL.",
        alt_keys: &[
            "REPO_CLONE_URL",
            "CLONE_URL",
            "CI_REPOSITORY_URL",
            "BITBUCKET_GIT_SSH_ORIGIN",
        ],
    },
    KEY_NAME => Variable {
        key: KEY_NAME,
        description: "The name of the project.",
        alt_keys: &[
            "NAME",
            "CI_PROJECT_NAME",
            "APP_NAME",
            "BITBUCKET_PROJECT_KEY",
        ],
    },
    KEY_VERSION_DATE => Variable {
        key: KEY_VERSION_DATE,
        description: "Date this version was committed to source control. ['%Y-%m-%d']",
        alt_keys: &[
            "VERSION_DATE",
            "DATE",
            "COMMIT_DATE",
            "PROJECT_COMMIT_DATE",
            "CI_COMMIT_TIMESTAMP",
        ],
    },
    KEY_BUILD_DATE => Variable {
        key: KEY_BUILD_DATE,
        description: "Date of this build. ['%Y-%m-%d']",
        alt_keys: &[],
    },
    KEY_BUILD_BRANCH => Variable {
        key: KEY_BUILD_BRANCH,
        description: "The development branch name.",
        alt_keys: &[
            "BRANCH",
            "GITHUB_REF",
            "CI_COMMIT_BRANCH",
            "BRANCH_NAME",
            "BITBUCKET_BRANCH",
            "TRAVIS_BRANCH",
        ],
    },
    KEY_BUILD_TAG => Variable {
        key: KEY_BUILD_TAG,
        description: "The tag of a commit that kicked off the build. This value is only available on tags. Not available for builds against branches.",
        alt_keys: &[
            "TAG",
            "GITHUB_REF",
            "CI_COMMIT_TAG",
            "BITBUCKET_TAG",
            "TRAVIS_TAG",
        ],
    },
    KEY_BUILD_IDENT => Variable {
        key: KEY_BUILD_IDENT,
        description: "Unique identifier of the state of the project that is being built (e.g. git commit SHA).",
        alt_keys: &[
            "GITHUB_SHA",
            "CI_COMMIT_SHA",
            "PULL_BASE_SHA",
            "BITBUCKET_COMMIT",
            "TRAVIS_COMMIT",
        ],
    },
    KEY_BUILD_OS => Variable {
        key: KEY_BUILD_OS,
        description: "Operating system we are building on. (common values: 'linux', 'macos', 'windows')",
        alt_keys: &[
            "OS",
            "RUNNER_OS",
            "CI_RUNNER_EXECUTABLE_ARCH",
            "TRAVIS_OS_NAME",
        ],
    },
    KEY_BUILD_OS_FAMILY => Variable {
        key: KEY_BUILD_OS_FAMILY,
        description: "Operating system family we are building on. (should be either 'unix' or 'windows')",
        alt_keys: &[
            "OS_FAMILY",
            "FAMILY",
        ],
    },
    KEY_BUILD_ARCH => Variable {
        key: KEY_BUILD_ARCH,
        description: "Computer hardware architecture we are building on. (common values: 'x86', 'x86_64')",
        alt_keys: &[
            "ARCH",
        ],
    },
    KEY_BUILD_HOSTING_URL => Variable {
        key: KEY_BUILD_HOSTING_URL,
        description: "Web URL under which the generated output will be available.",
        alt_keys: &["HOSTING_URL", "CI_PAGES_URL"],
    },
    KEY_BUILD_NUMBER => Variable {
        key: KEY_BUILD_NUMBER,
        description: "The build number (1, 2, 3) starts at 1 for each repo and branch.",
        alt_keys: &[
            "NUMBER",
            "ID",
            "BUILD_ID",
            "BITBUCKET_BUILD_NUMBER",
            "TRAVIS_BUILD_NUMBER",
        ],
    },
    KEY_CI => Variable {
        key: KEY_CI,
        description: "'true' if running on a CI/build-bot.",
        alt_keys: &[],
    },
};
}
