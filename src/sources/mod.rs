// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod bitbucket_ci;
pub mod env;
pub mod fs;
pub mod git;
pub mod github_ci;
pub mod gitlab_ci;
pub mod jenkins_ci;
pub mod travis_ci;

use std::error::Error;

use clap::lazy_static::lazy_static;
use url::{Host, Url};

use crate::constants;
use crate::environment::Environment;
use crate::var::Key;

type BoxResult<T> = Result<T, Box<dyn Error>>;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Hierarchy {
    Low,
    Middle,
    High,
}

lazy_static! {
    static ref NO_PROPS: Vec::<String> = Vec::<String>::new();
}

pub trait VarSource {
    /// Indicates whether this source of variables is usable.
    /// It might not be usable if the underlying data-source (e.g. a file) does not exist,
    /// or is not reachable (e.g. a web URL).
    fn is_usable(&self, environment: &mut Environment) -> bool;

    /// Used to evaluate whether we preffer this sources values
    /// over the ones of an other.
    /// This is used for sorting.
    fn hierarchy(&self) -> Hierarchy;

    /// The name of this type.
    /// This is used for display and sorting.
    fn type_name(&self) -> &'static str;

    /// The properties (usually parameters to `Self::new`)
    /// of the particular instance of an object of this trait.
    /// This is used for display and sorting.
    fn properties(&self) -> &Vec<String>;

    /// As I failed to implement `fmt::Display` for all implementing structs
    /// in one impl, I took this road, which works for our case.
    fn display(&self) -> String {
        format!("{}{:?}", self.type_name(), self.properties())
    }

    /// Tries to retrieve the value of a single `key`.
    ///
    /// # Errors
    ///
    /// If the underlying data-source (e.g. a file) does not exist,
    /// or is not reachable (e.g. a web URL),
    /// or innumerable other kinds of problems,
    /// depending on the kind of the source.
    fn retrieve(&self, environment: &mut Environment, key: Key) -> BoxResult<Option<String>>;
}

pub fn var(environment: &Environment, key: &str) -> Option<String> {
    environment
        .vars
        .get(key)
        .map(std::borrow::ToOwned::to_owned)
}

/// Extracts the project name from the project slug ("user/project").
///
/// # Errors
///
/// When splitting the slug at '/' fails.
pub fn proj_name_from_slug(slug: Option<&String>) -> BoxResult<Option<String>> {
    Ok(if let Some(repo_name) = slug {
        Some(repo_name
            .split('/')
            .nth(1)
            .ok_or("Failed splitting the repository name (assumed to be \"user/repo\") into \"user\" and \"repo\"")?
            .to_owned())
    } else {
        None
    })
}

/// Tries to construct the issues URL
/// from the repo web URL property of a variable source.
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
//
// Real world issues URLs:
// * https://github.com/OPEN-NEXT/LOSH-Krawler/issues
// * https://gitlab.com/openflexure/openflexure-microscope/-/issues // NOTE That this uses an additional "-/", but leaving it out also works!
// * https://gitlab.com/openflexure/openflexure-microscope/issues // NOTE That this uses an additional "-/", but leaving it out also works!
// * https://gitlab.opensourceecology.de/hoijui/osh-tool/-/issues
// * https://gitlab.opensourceecology.de/groups/verein/projekte/losh/-/issues
// * https://bitbucket.org/Aouatef/master_arbeit/issues
pub fn try_construct_issues_url<S: VarSource>(
    var_source: &S,
    environment: &mut Environment,
) -> BoxResult<Option<String>> {
    let base_repo_web_url = var_source.retrieve(environment, Key::RepoWebUrl)?;
    Ok(base_repo_web_url.map(|base_repo_web_url| format!("{}/issues", base_repo_web_url)))
}

/// Tries to construct the raw prefix URL
/// from the repo web URL property of a variable source.
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
//
// Real world raw prefix URLs (the part in []):
// * [https://raw.githubusercontent.com/hoijui/nim-ci]/master/.github/workflows/docker.yml
// * [https://gitlab.com/OSEGermany/osh-tool/-/raw]/master/data/source_extension_formats.csv
// * [https://gitlab.com/OSEGermany/osh-tool/raw]/master/data/source_extension_formats.csv
// * [https://bitbucket.org/Aouatef/master_arbeit/raw]/ae4a42a850b359a23da2483eb8f867f21c5382d4/procExData/import.sh
pub fn try_construct_raw_prefix_url<S: VarSource>(
    var_source: &S,
    environment: &mut Environment,
) -> BoxResult<Option<String>> {
    if let Some(base_repo_web_url) = var_source.retrieve(environment, Key::RepoWebUrl)? {
        let mut url = Url::parse(&base_repo_web_url)?;
        if let Some(host) = url.host() {
            return Ok(match host {
                Host::Domain(constants::D_GIT_HUB_COM) => {
                    url.set_host(Some(constants::D_GIT_HUB_COM_RAW))?;
                    Some(url.to_string())
                }
                Host::Domain(constants::D_GIT_LAB_COM) => {
                    url.set_path(&format!("{}/-/raw", url.path()));
                    Some(url.to_string())
                }
                Host::Domain(constants::D_BIT_BUCKET_ORG) => {
                    url.set_path(&format!("{}/raw", url.path()));
                    Some(url.to_string())
                }
                _ => None,
            });
        }
    }
    Ok(None)
}

/// Tries to construct the file prefix URL
/// from the repo web URL property of a variable source.
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
//
// Real world file prefix URLs (the part in []):
// * [https://github.com/hoijui/nim-ci/blob]/master/.github/workflows/docker.yml
// * [https://gitlab.com/OSEGermany/osh-tool/-/blob]/master/data/source_extension_formats.csv
// * [https://bitbucket.org/Aouatef/master_arbeit/src]/ae4a42a850b359a23da2483eb8f867f21c5382d4/procExData/import.sh
pub fn try_construct_file_prefix_url<S: VarSource>(
    var_source: &S,
    environment: &mut Environment,
) -> BoxResult<Option<String>> {
    if let Some(base_repo_web_url) = var_source.retrieve(environment, Key::RepoWebUrl)? {
        let mut url = Url::parse(&base_repo_web_url)?;
        if let Some(host) = url.host() {
            return Ok(match host {
                Host::Domain(constants::D_GIT_HUB_COM) => {
                    url.set_path(&format!("{}/blob", url.path()));
                    Some(url.to_string())
                }
                Host::Domain(constants::D_GIT_LAB_COM) => {
                    url.set_path(&format!("{}/-/blob", url.path()));
                    Some(url.to_string())
                }
                Host::Domain(constants::D_BIT_BUCKET_ORG) => {
                    url.set_path(&format!("{}/src", url.path()));
                    Some(url.to_string())
                }
                _ => None,
            });
        }
    }
    Ok(None)
}

/// Tries to construct the directory prefix URL
/// from the repo web URL property of a variable source.
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
//
// Real world dir prefix URLs (the part in []):
// * [https://github.com/hoijui/nim-ci/tree]/master/.github/workflows/
// * [https://gitlab.com/OSEGermany/osh-tool/-/tree]/master/data/
// * [https://bitbucket.org/Aouatef/master_arbeit/src]/ae4a42a850b359a23da2483eb8f867f21c5382d4/procExData/
pub fn try_construct_dir_prefix_url<S: VarSource>(
    var_source: &S,
    environment: &mut Environment,
) -> BoxResult<Option<String>> {
    if let Some(base_repo_web_url) = var_source.retrieve(environment, Key::RepoWebUrl)? {
        let mut url = Url::parse(&base_repo_web_url)?;
        if let Some(host) = url.host() {
            return Ok(match host {
                Host::Domain(constants::D_GIT_HUB_COM) => {
                    url.set_path(&format!("{}/tree", url.path()));
                    Some(url.to_string())
                }
                Host::Domain(constants::D_GIT_LAB_COM) => {
                    url.set_path(&format!("{}/-/tree", url.path()));
                    Some(url.to_string())
                }
                Host::Domain(constants::D_BIT_BUCKET_ORG) => {
                    url.set_path(&format!("{}/src", url.path()));
                    Some(url.to_string())
                }
                _ => None,
            });
        }
    }
    Ok(None)
}

/// Tries to construct the commit prefix URL
/// from the repo web URL property of a variable source.
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
//
// Real world commit prefix URLs (the part in []):
// * [https://github.com/hoijui/nim-ci/commit]/ae4a42a850b359a23da2483eb8f867f21c5382d4
// * [https://gitlab.com/OSEGermany/osh-tool/-/commit]/ae4a42a850b359a23da2483eb8f867f21c5382d4
// * [https://bitbucket.org/Aouatef/master_arbeit/commits]/ae4a42a850b359a23da2483eb8f867f21c5382d4
pub fn try_construct_commit_prefix_url<S: VarSource>(
    var_source: &S,
    environment: &mut Environment,
) -> BoxResult<Option<String>> {
    if let Some(base_repo_web_url) = var_source.retrieve(environment, Key::RepoWebUrl)? {
        let mut url = Url::parse(&base_repo_web_url)?;
        if let Some(host) = url.host() {
            return Ok(match host {
                Host::Domain(constants::D_GIT_HUB_COM) => {
                    url.set_path(&format!("{}/commit", url.path()));
                    Some(url.to_string())
                }
                Host::Domain(constants::D_GIT_LAB_COM) => {
                    url.set_path(&format!("{}/-/commit", url.path()));
                    Some(url.to_string())
                }
                Host::Domain(constants::D_BIT_BUCKET_ORG) => {
                    url.set_path(&format!("{}/commits", url.path()));
                    Some(url.to_string())
                }
                _ => None,
            });
        }
    }
    Ok(None)
}
