// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::tools::git_hosting_provs::HostingType;
use thiserror::Error;

use clap::lazy_static::lazy_static;
use regex::Regex;
use url::Url;

use crate::constants;
use crate::environment::Environment;
use crate::var::Key;

type Res = Result<Option<String>, Error>;

/// This enumerates all possible errors returned by this module.
#[derive(Error, Debug)]
pub enum Error {
    /// A required properties value could not be evaluated
    #[error("The input value(s) for the conversion were not recognized/usable")]
    BadInputValue {
        key: Key,
        msg: String,
        input: String,
    },

    /// A required properties value could not be evaluated
    #[error("The input value(s) for the conversion were not recognized/usable")]
    BadInputValueErr {
        key: Key,
        msg: String,
        input: String,
        #[from(url::parser::ParseError)]
        source: Box<dyn std::error::Error>,
    },
}

/// Extracts the project name from the project slug,
/// which might be "user/project" or "user/group/sub-group/project".
///
/// # Errors
///
/// When splitting the slug at '/' fails.
pub fn slug_to_proj_name(slug: Option<&String>) -> Res {
    Ok(if let Some(slug) = slug {
        Some(slug
            .split('/')
            .nth(1) // TODO We actually need the last, not the 2nd!
            .ok_or(Error::BadInputValue {
                key: Key::NameMachineReadable,
                msg: r#"Failed splitting off the project name from the project slug, which is assumed to be "user/project" or "user/group/sub-group/project""#.to_owned(),
                input: slug.clone(),
            })?
            .to_owned())
    } else {
        None
    })
}

/// Tries to construct the machine-readable project name
/// from the human-readable one.
/// See also [`crate::validator::validate_name`].
///
/// # Errors
///
/// If the resulting name is empty.
pub fn name_to_machine_readable_name(_environment: &Environment, human_name: &str) -> Res {
    lazy_static! {
        static ref R_BAD_CHAR: Regex = Regex::new(r"[^0-9a-zA-Z_-]").unwrap();
    }

    let machine_name = R_BAD_CHAR.replace_all(human_name, "_");
    if machine_name.is_empty() {
        return Err(Error::BadInputValue {
            key: Key::NameMachineReadable,
            msg: "The input resulted in an empty machine-readable name".to_owned(),
            input: human_name.to_owned(),
        });
    }
    Ok(Some(machine_name.into_owned()))
}

/// Tries to construct the machine-readable project name
/// from the human-readable one of a variable source.
/// See also [`crate::validator::validate_name_machine_readable`].
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
pub fn web_url_to_machine_readable_name(_environment: &Environment, web_url: &str) -> Res {
    lazy_static! {
        static ref R_NAME_EXTRACTOR: Regex = Regex::new(r"^.*/").unwrap();
    }

    let machine_name = R_NAME_EXTRACTOR.replace(web_url, "");
    if machine_name.as_ref() == web_url {
        return Err(Error::BadInputValue {
            key: Key::NameMachineReadable,
            msg: "Failed to extract human-readable project name from web URL".to_owned(),
            input: web_url.to_owned(),
        });
    }
    Ok(Some(machine_name.into_owned()))
}

/// Tries to construct the issues URL
/// from the repo web URL property of a variable source.
/// See also [`crate::validator::validate_repo_issues_url`].
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
//
// Real world issues URLs:
// * https://github.com/OPEN-NEXT/LOSH-Krawler/issues
// * https://gitlab.com/openflexure/openflexure-microscope/-/issues // NOTE That this uses an additional "-/", but leaving it out also works!
// * https://gitlab.com/openflexure/openflexure-microscope/issues
// * https://gitlab.opensourceecology.de/hoijui/osh-tool/-/issues
// * https://gitlab.opensourceecology.de/groups/verein/projekte/losh/-/issues
// * https://bitbucket.org/Aouatef/master_arbeit/issues
pub fn web_url_to_issues_url(environment: &Environment, web_url: &str) -> Res {
    match Url::parse(web_url) {
        Err(err) => Err(Error::BadInputValueErr {
            key: Key::RepoIssuesUrl,
            msg: "Not a valid URL".to_owned(),
            input: web_url.to_owned(),
            source: Box::new(err),
        }),
        Ok(mut url) => {
            Ok(match environment.settings.hosting_type(&url) {
                HostingType::BitBucket | HostingType::GitHub => {
                    url.set_path(&format!("{}/issues", url.path()));
                    Some(url.to_string())
                }
                HostingType::GitLab => {
                    url.set_path(&format!("{}/-/issues", url.path()));
                    Some(url.to_string())
                }
                _ => None, // TODO Implement the others!
            })
        }
    }
}

/// Tries to construct a repo raw versioned prefix URL
/// from a repo web URL.
/// See also [`crate::validator::validate_repo_raw_versioned_prefix_url`].
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
pub fn web_url_to_raw_prefix_url(environment: &Environment, web_url: &str) -> Res {
    match Url::parse(web_url) {
        Err(err) => Err(Error::BadInputValueErr {
            key: Key::RepoRawVersionedPrefixUrl,
            msg: "Not a valid URL".to_owned(),
            input: web_url.to_owned(),
            source: Box::new(err),
        }),
        Ok(mut url) => {
            Ok(match environment.settings.hosting_type(&url) {
                HostingType::GitHub => {
                    url.set_host(Some(constants::D_GIT_HUB_COM_RAW))
                        .map_err(|err| Error::BadInputValueErr {
                            key: Key::RepoRawVersionedPrefixUrl,
                            msg: format!(
                                "Failed to parse '{}' host for URL",
                                constants::D_GIT_HUB_COM_RAW
                            ),
                            input: web_url.to_owned(),
                            source: Box::new(err),
                        })?;
                    Some(url.to_string())
                }
                HostingType::GitLab => {
                    url.set_path(&format!("{}/-/raw", url.path()));
                    Some(url.to_string())
                }
                HostingType::BitBucket => {
                    url.set_path(&format!("{}/raw", url.path()));
                    Some(url.to_string())
                }
                _ => None, // TODO Implement the others!
            })
        }
    }
}

/// Tries to construct the file prefix URL
/// from the repo web URL property of a variable source.
/// See also [`crate::validator::validate_repo_versioned_file_prefix_url`].
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
//
// Real world file prefix URLs (the part in []):
// * [https://github.com/hoijui/nim-ci/blob]/master/.github/workflows/docker.yml
// * [https://gitlab.com/OSEGermany/osh-tool/-/blob]/master/data/source_extension_formats.csv
// * [https://bitbucket.org/Aouatef/master_arbeit/src]/ae4a42a850b359a23da2483eb8f867f21c5382d4/procExData/import.sh
pub fn web_url_to_versioned_file_prefix_url(environment: &Environment, web_url: &str) -> Res {
    match Url::parse(web_url) {
        Err(err) => Err(Error::BadInputValueErr {
            key: Key::RepoVersionedFilePrefixUrl,
            msg: "Not a valid URL".to_owned(),
            input: web_url.to_owned(),
            source: Box::new(err),
        }),
        Ok(mut url) => {
            Ok(match environment.settings.hosting_type(&url) {
                HostingType::GitHub => {
                    url.set_path(&format!("{}/blob", url.path()));
                    Some(url.to_string())
                }
                HostingType::GitLab => {
                    url.set_path(&format!("{}/-/blob", url.path()));
                    Some(url.to_string())
                }
                HostingType::BitBucket => {
                    url.set_path(&format!("{}/src", url.path()));
                    Some(url.to_string())
                }
                _ => None, // TODO Implement the others!
            })
        }
    }
}

/// Tries to construct the directory prefix URL
/// from the repo web URL property of a variable source.
/// See also [`crate::validator::validate_repo_versioned_dir_prefix_url`].
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
//
// Real world dir prefix URLs (the part in []):
// * [https://github.com/hoijui/nim-ci/tree]/master/.github/workflows/
// * [https://gitlab.com/OSEGermany/osh-tool/-/tree]/master/data/
// * [https://bitbucket.org/Aouatef/master_arbeit/src]/ae4a42a850b359a23da2483eb8f867f21c5382d4/procExData/
pub fn web_url_to_versioned_dir_prefix_url(environment: &Environment, web_url: &str) -> Res {
    match Url::parse(web_url) {
        Err(err) => Err(Error::BadInputValueErr {
            key: Key::RepoVersionedDirPrefixUrl,
            msg: "Not a valid URL".to_owned(),
            input: web_url.to_owned(),
            source: Box::new(err),
        }),
        Ok(mut url) => {
            Ok(match environment.settings.hosting_type(&url) {
                HostingType::GitHub => {
                    url.set_path(&format!("{}/tree", url.path()));
                    Some(url.to_string())
                }
                HostingType::GitLab => {
                    url.set_path(&format!("{}/-/tree", url.path()));
                    Some(url.to_string())
                }
                HostingType::BitBucket => {
                    url.set_path(&format!("{}/src", url.path()));
                    Some(url.to_string())
                }
                _ => None, // TODO Implement the others!
            })
        }
    }
}

/// Tries to construct the commit prefix URL
/// from the repo web URL property of a variable source.
/// See also [`crate::validator::validate_repo_commit_prefix_url`].
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
//
// Real world commit prefix URLs (the part in []):
// * [https://github.com/hoijui/nim-ci/commit]/ae4a42a850b359a23da2483eb8f867f21c5382d4
// * [https://gitlab.com/OSEGermany/osh-tool/-/commit]/ae4a42a850b359a23da2483eb8f867f21c5382d4
// * [https://bitbucket.org/Aouatef/master_arbeit/commits]/ae4a42a850b359a23da2483eb8f867f21c5382d4
pub fn web_url_to_commit_prefix_url(environment: &Environment, web_url: &str) -> Res {
    match Url::parse(web_url) {
        Err(err) => Err(Error::BadInputValueErr {
            key: Key::RepoCommitPrefixUrl,
            msg: "Not a valid URL".to_owned(),
            input: web_url.to_owned(),
            source: Box::new(err),
        }),
        Ok(mut url) => {
            Ok(match environment.settings.hosting_type(&url) {
                HostingType::GitHub => {
                    url.set_path(&format!("{}/commit", url.path()));
                    Some(url.to_string())
                }
                HostingType::GitLab => {
                    url.set_path(&format!("{}/-/commit", url.path()));
                    Some(url.to_string())
                }
                HostingType::BitBucket => {
                    url.set_path(&format!("{}/commits", url.path()));
                    Some(url.to_string())
                }
                _ => None, // TODO Implement the others!
            })
        }
    }
}

/// Converts any kind of clone URL to an HTTP(S) or SSH one.
/// See also [`crate::validator::validate_repo_clone_url`]
/// and [`crate::validator::validate_repo_clone_url_ssh`].
///
/// # Errors
///
/// If conversion failed, usually due to an invalid input URL.
///
/// If the user in the URL suggests a non-public access URL. // TODO
pub fn clone_url_conversion(any_clone_url: &str, to_http: bool) -> Res {
    lazy_static! {
        // This matches all these 3 types of clone URLs:
        // * git@github.com:hoijui/rust-project-scripts.git
        // * ssh://github.com/hoijui/rust-project-scripts.git
        // * https://github.com/hoijui/rust-project-scripts.git
        static ref R_CLONE_URL: Regex = Regex::new(r"^((?P<protocol>[0-9a-zA-Z._-]+)://)?((?P<user>[0-9a-zA-Z._-]+)@)?(?P<host>[0-9a-zA-Z._-]+)([/:](?P<path_and_rest>.+)?)?$").unwrap();
    }
    let to_protocol_str = if to_http { "https" } else { "ssh" };
    let clone_url_res = R_CLONE_URL.replace(any_clone_url, |caps: &regex::Captures| {
        //crate::tools::git::let_named_cap!(caps, protocol);
        // crate::tools::git::let_named_cap!(caps, host);
        crate::tools::git::let_named_cap!(caps, user);
        // crate::tools::git::let_named_cap!(caps, path_and_rest);
        let host = caps.name("host").map(|m| m.as_str());
        let user_at = if user.is_empty() {
            String::new()
        } else {
            user.to_owned() + "@"
        };
        if let Some(host) = host {
            let path_and_rest = caps
                .name("path_and_rest")
                .map(|m| "/".to_owned() + m.as_str());
            if to_http {
                format!(
                    "{protocol}://{host}/{path_and_rest}",
                    protocol = to_protocol_str,
                    // user = user.to_lowercase(),
                    host = host,
                    path_and_rest = path_and_rest.unwrap_or_default(),
                )
            } else {
                format!(
                    "{user}{host}/{path_and_rest}",
                    // "{protocol}://{host}/{path_and_rest}", // TODO This is the (URL spec compatible) alternative
                    // protocol = to_protocol_str,
                    user = user_at.to_lowercase(),
                    host = host,
                    path_and_rest = path_and_rest.unwrap_or_default(),
                )
            }
        } else {
            any_clone_url.to_owned()
        }
    });
    if clone_url_res == any_clone_url {
        Err(Error::BadInputValue {
            key: if to_http {
                Key::RepoCloneUrl
            } else {
                Key::RepoCloneUrlSsh
            },
            msg: format!(
                "Unable to convert clone URL to {} using regex '{}'",
                to_protocol_str,
                R_CLONE_URL.as_str()
            ),
            input: any_clone_url.to_owned(),
        })
    } else {
        Ok(Some(clone_url_res.as_ref().to_owned()))
    }
}

/// Converts any kind of clone URL (wrapped in an `Option`) to an HTTP(S) or SSH one.
/// See [`clone_url_conversion`].
///
/// # Errors
///
/// If conversion failed, usually due to an invalid input URL.
///
/// If the user in the URL suggests a non-public access URL.
pub fn clone_url_conversion_option(any_clone_url: Option<&String>, to_http: bool) -> Res {
    Ok(match any_clone_url {
        Some(clone_url) => clone_url_conversion(clone_url, to_http)?,
        None => None,
    })
}
