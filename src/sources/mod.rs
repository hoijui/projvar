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
use regex::Regex;
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
    Higher,
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

/// Tries to construct the machine-readable project name
/// from the human-readable one of a variable source.
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
pub fn try_construct_machine_readable_name_from_name<S: VarSource>(
    var_source: &S,
    environment: &mut Environment,
) -> BoxResult<Option<String>> {
    lazy_static! {
        static ref R_BAD_CHAR: Regex = Regex::new(r"[^0-9a-zA-Z_-]").unwrap();
    }

    Ok(match var_source.retrieve(environment, Key::Name)? {
        Some(human_name) => {
            let machine_name = R_BAD_CHAR.replace_all(&human_name, "_");
            Some(machine_name.into_owned())
        }
        None => None,
    })
}

/// Tries to construct the machine-readable project name
/// from the human-readable one of a variable source.
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
pub fn try_construct_machine_readable_name_from_web_url<S: VarSource>(
    var_source: &S,
    environment: &mut Environment,
) -> BoxResult<Option<String>> {
    lazy_static! {
        static ref R_NAME_EXTRACTOR: Regex = Regex::new(r"^.*/").unwrap();
    }

    Ok(match var_source.retrieve(environment, Key::RepoWebUrl)? {
        Some(web_url) => {
            let machine_name = R_NAME_EXTRACTOR.replace(&web_url, "");
            if machine_name == web_url {
                return Err(Box::new(git2::Error::from_str(
                    "Failed to extract human-readable project name from web URL",
                )));
            }
            Some(machine_name.into_owned())
        }
        None => None,
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

/// Converts an SSH clone URL to an HTTP(S) one.
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
// * git@github.com:hoijui/rust-project-scripts.git
// * https://github.com/hoijui/rust-project-scripts.git
pub fn convert_clone_ssh_to_http(clone_url_ssh: &str) -> BoxResult<String> {
    lazy_static! {
        static ref R_CLONE_URL: Regex = Regex::new(r"^((?P<protocol>[0-9a-zA-Z._-]+)://)?((?P<user>[0-9a-zA-Z._-]+)@)?(?P<host>[0-9a-zA-Z._-]+)([/:](?P<path_and_rest>.+)?)?$").unwrap();
        /* static ref R_PROTOCOL: Regex = Regex::new(r"^[a-z]+:").unwrap(); */
        /* static ref R_PROTOCOL: Regex = Regex::new(r"^[a-z]+:").unwrap(); */
        /* static ref R_COLON: Regex = Regex::new(r":([^/])").unwrap(); */
    }
    // let clone_url_https = R_PROTOCOL.replace(clone_url_ssh, ""ssh);

    let clone_url_https = R_CLONE_URL.replace(clone_url_ssh, |caps: &regex::Captures| {
        // let_named_cap!(caps, protocol);
        let_named_cap!(caps, host);
        // let_named_cap!(caps, user);
        let_named_cap!(caps, path_and_rest);
        let host = caps.name("host").map(|m| m.as_str());
        if let Some(host) = host {
            let path_and_rest = caps
                .name("path_and_rest")
                .map(|m| "/".to_owned() + m.as_str());
            format!(
                "{protocol}://{host}/{path_and_rest}",
                protocol = "https",
                // user = user.to_lowercase(),
                host = host,
                path_and_rest = path_and_rest.unwrap_or_default(),
            )
        } else {
            clone_url_ssh.to_owned()
        }
    });
    if clone_url_https == clone_url_ssh {
        Err(Box::new(""))
    } else {
        Ok(clone_url_https.as_ref().to_owned())
    }

    /* let clone_url_ssh = R_COLON.replace(clone_url_ssh, "/$1"); */
    /* if let Some(base_repo_web_url) = var_source.retrieve(environment, Key::RepoWebUrl)? { */
    /*     let mut url = Url::parse(&base_repo_web_url)?; */
    /*     if let Some(host) = url.host() { */
    /*         return Ok(match host { */
    /*             Host::Domain(constants::D_GIT_HUB_COM) => { */
    /*                 url.set_path(&format!("{}/commit", url.path())); */
    /*                 Some(url.to_string()) */
    /*             } */
    /*             Host::Domain(constants::D_GIT_LAB_COM) => { */
    /*                 url.set_path(&format!("{}/-/commit", url.path())); */
    /*                 Some(url.to_string()) */
    /*             } */
    /*             Host::Domain(constants::D_BIT_BUCKET_ORG) => { */
    /*                 url.set_path(&format!("{}/commits", url.path())); */
    /*                 Some(url.to_string()) */
    /*             } */
    /*             _ => None, */
    /*         }); */
    /*     } */
    /* } */
    /* Ok(None) */
}

/// Converts an SSH clone URL to an HTTP(S) one.
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
// * git@github.com:hoijui/rust-project-scripts.git
// * https://github.com/hoijui/rust-project-scripts.git
pub fn try_convert_clone_ssh_to_http<S: VarSource>(
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
