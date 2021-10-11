// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::tools::git_hosting_provs::{HostingType, PublicSite};
use crate::var::Key;
use crate::{constants, environment::Environment};
use chrono::{DateTime, NaiveDateTime};
use clap::lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;
use url::Url;

pub type Result = std::result::Result<Option<Warning>, Error>;
pub type Validator = fn(&mut Environment, &str) -> Result;

// See these resources for implement our own, custom errors
// accoridng to rust best practises for errors (and error handling):
// * good, simple intro:
//   <https://nick.groenen.me/posts/rust-error-handling/>
// * very nice, extensive, detailed example:
//   https://www.lpalmieri.com/posts/error-handling-rust/#removing-the-boilerplate-with-thiserror

/// This enumerates all possible errors returned by this module.
#[derive(Error, Debug)]
pub enum Warning {
    /// A non-required properties value could not be evaluated;
    /// no source returned a (valid) value for it.
    #[error("No value found for a property")]
    Missing,

    /// The evaluated value is usable, but not optimal.
    #[error("The value '{value}' is usable, but not optimal - {msg}")]
    SuboptimalValue { msg: String, value: String },

    /// We have no way to check this value for validity,
    /// but at least were not able to prove it invalid.
    #[error("No method to check validity of value '{value}'")]
    Unknown { value: String },
}

/// This enumerates all possible errors returned by this module.
#[derive(Error, Debug)]
pub enum Error {
    /// Represents an empty source. For example, an empty text file being given
    /// as input to `count_words()`.
    // #[error("Source contains no data")]
    // EmptySource,

    // /// Represents a failure to read from input.
    // #[error("Read error")]
    // ReadError { source: std::io::Error },

    /// A required properties value could not be evaluated
    #[error("No value found for a required property")]
    Missing,

    /// The evaluated value is not usable.
    /// It make sno sense for this property as it is,
    /// but it is close to a value that would make sense,
    /// so it might contain a typo, one small part is missing or too much,
    /// or something similar.
    #[error("The value '{value}' is unfit for this key, but only just - {msg}")]
    AlmostUsableValue { msg: String, value: String },

    /// The evaluated value is not usable.
    /// It makes no sense for this property.
    #[error("The value '{value}' is unfit for this key - {msg}")]
    BadValue { msg: String, value: String },

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IO(#[from] std::io::Error),
}

fn missing(environment: &mut Environment, key: Key) -> Result {
    if environment.settings.required_keys.contains(&key) {
        Err(Error::Missing)
    } else {
        Ok(Some(Warning::Missing))
    }
}

fn validate_version(environment: &mut Environment, value: &str) -> Result {
    lazy_static! {
        // The official SemVer regex as of September 2021, taken from
        // https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
        // TODO Think of what to do if we have a "v" prefix, as in "v1.2.3" -> best: remove it, but where.. a kind of pre-validator function?
        static ref R_SEM_VERS_RELEASE: Regex = Regex::new(r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)$").unwrap();
        static ref R_SEM_VERS: Regex = Regex::new(r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$").unwrap();
        static ref R_GIT_VERS: Regex = Regex::new(r"^((g[0-9a-f]{7})|((0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)))(-(0|[1-9]\d*)-(g[0-9a-f]{7}))?((-dirty(-broken)?)|-broken(-dirty)?)?$").unwrap();
        static ref R_GIT_SHA: Regex = Regex::new(r"^g?[0-9a-f]{7,40}$").unwrap();
        static ref R_UNKNOWN_VERS: Regex = Regex::new(r"^($|#|//)").unwrap();
    }
    // log::info!("Validating version: '{}' ...", value);
    if R_SEM_VERS_RELEASE.is_match(value) {
        Ok(None)
    } else if R_SEM_VERS.is_match(value) || R_GIT_VERS.is_match(value) {
        Ok(Some(Warning::SuboptimalValue {
            msg: "This version is technically good, but not a release-version (i.e., does not look so nice)".to_owned(),
            value: value.to_owned(),
        }))
    } else if R_GIT_SHA.is_match(value) {
        Ok(Some(Warning::SuboptimalValue {
            msg:
                "This version is technically ok, but not a release-version, and not human-readable"
                    .to_owned(),
            value: value.to_owned(),
        }))
    } else if R_UNKNOWN_VERS.is_match(value) {
        missing(environment, Key::Version)
    } else {
        Err(Error::BadValue {
            msg: "Not a valid version".to_owned(),
            value: value.to_owned(),
        })
    }
}

fn validate_license(environment: &mut Environment, value: &str) -> Result {
    if value.is_empty() {
        missing(environment, Key::License)
    } else if constants::SPDX_IDENTS.contains(&value) {
        Ok(None)
    } else {
        Ok(Some(Warning::SuboptimalValue {
            msg: "Not a recognized SPDX license identifier".to_owned(),
            value: value.to_owned(),
        }))
    }
}

fn validate_licenses(environment: &mut Environment, value: &str) -> Result {
    if value.is_empty() {
        missing(environment, Key::Licenses)
    } else {
        for license in value.split(',') {
            let license = license.trim();
            if !constants::SPDX_IDENTS.contains(&license) {
                return Ok(Some(Warning::SuboptimalValue {
                    msg: format!(
                        "Not all of these are recognized SPDX license identifiers: {}",
                        license
                    ),
                    value: value.to_owned(),
                }));
            }
        }
        Ok(None)
    }
}

fn check_public_url(
    _environment: &mut Environment,
    value: &str,
    allow_ssh: bool,
) -> std::result::Result<Url, Error> {
    match Url::parse(value) {
        Err(_err) => Err(Error::BadValue {
            msg: "Not a valid URL".to_owned(),
            value: value.to_owned(),
        }),
        Ok(url) => {
            if !(["http", "https"].contains(&url.scheme()) || (allow_ssh && url.scheme() == "ssh"))
            {
                Err(Error::AlmostUsableValue {
                    msg: format!(
                        "Should use one of these as protocol(scheme): [http, https{}]",
                        if allow_ssh { ", ssh" } else { "" }
                    ),
                    value: value.to_owned(),
                })
            } else if url.username() != "" {
                Err(Error::AlmostUsableValue {
                    msg: format!(
                        "Should be anonymous access, but specifies a user-name: {}",
                        url.username()
                    ),
                    value: value.to_owned(),
                })
            } else if let Some(_pw) = url.password() {
                Err(Error::AlmostUsableValue {
                    msg: "Should be anonymous access, but contains a password".to_owned(),
                    value: value.to_owned(),
                })
            } else if let Some(query) = url.query() {
                Err(Error::AlmostUsableValue {
                    msg: format!(
                        "Should be a simple URL, but uses query arguments: {}",
                        query
                    ),
                    value: value.to_owned(),
                })
            } else if let Some(fragment) = url.fragment() {
                Err(Error::AlmostUsableValue {
                    msg: format!("Should be a simple URL, but uses a fragment: {}", fragment),
                    value: value.to_owned(),
                })
            } else {
                Ok(url)
            }
        }
    }
}

fn check_empty(_environment: &mut Environment, value: &str, part_desc: &str) -> Result {
    if value.is_empty() {
        Err(Error::BadValue {
            msg: format!("{} can not be empty", part_desc),
            value: value.to_owned(),
        })
    } else {
        Ok(None)
    }
}

fn eval_hosting_type(environment: &mut Environment, url: &Url) -> HostingType {
    let configured_ht = environment.settings.hosting_type;
    if let HostingType::Unknown = configured_ht {
        HostingType::from(PublicSite::from(url.host()))
    } else {
        configured_ht
    }
}

fn eval_hosting_type_from_hosting_suffix(environment: &mut Environment, url: &Url) -> HostingType {
    let configured_ht = environment.settings.hosting_type;
    if let HostingType::Unknown = configured_ht {
        HostingType::from(PublicSite::from_hosting_domain_option(url.host().as_ref()))
    } else {
        configured_ht
    }
}

fn check_url_path(value: &str, url_desc: &str, url: &Url, path_reg: Option<&Regex>) -> Result {
    if let (Some(path_reg), Some(host)) = (path_reg, url.host().as_ref()) {
        if path_reg.is_match(url.path()) {
            Ok(None)
        } else {
            Err(Error::AlmostUsableValue {
                msg: format!(
                    r#"For {}, this path part of the {} URL is invalid: "{}"; it should match "{}""#,
                    host,
                    url_desc,
                    url.path(),
                    path_reg.as_str()
                ),
                value: value.to_owned(),
            })
        }
    } else {
        Ok(Some(Warning::Unknown {
            value: value.to_owned(),
        }))
    }
}

fn check_url_host(value: &str, url_desc: &str, url: &Url, host_reg: Option<&Regex>) -> Result {
    if let (Some(host_reg), Some(host)) = (host_reg, url.host().as_ref()) {
        let host_str = host.to_string();
        if host_reg.is_match(&host_str) {
            Ok(None)
        } else {
            Err(Error::AlmostUsableValue {
                msg: format!(
                    r#"For {}, this host part of the {} URL is invalid: "{}"; it should match "{}""#,
                    host,
                    url_desc,
                    url.path(),
                    host_reg.as_str()
                ),
                value: value.to_owned(),
            })
        }
    } else {
        Ok(Some(Warning::Unknown {
            value: value.to_owned(),
        }))
    }
}

fn validate_repo_web_url(environment: &mut Environment, value: &str) -> Result {
    lazy_static! {
        static ref R_GIT_HUB_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)/?$").unwrap();
        static ref R_GIT_LAB_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/((?P<structure>[^/]+)/)*(?P<repo>[^/]+)/?$").unwrap();
        static ref R_BIT_BUCKET_PATH: Regex = (*R_GIT_HUB_PATH).clone();
    }

    let url = check_public_url(environment, value, false)?;
    let hosting_type = eval_hosting_type(environment, &url);
    let host_reg: Option<&Regex> = match hosting_type {
        HostingType::GitHub => Some(&R_GIT_HUB_PATH),
        HostingType::GitLab => Some(&R_GIT_LAB_PATH),
        HostingType::BitBucket => Some(&R_BIT_BUCKET_PATH),
        _ => None, // TODO Implement the others
    };
    check_url_path(value, "versioned web", &url, host_reg)
}

// * git@bitbucket.org:Aouatef/master_arbeit.git
// * https://hoijui@bitbucket.org/Aouatef/master_arbeit.git
fn validate_repo_clone_url(environment: &mut Environment, value: &str) -> Result {
    lazy_static! {
        // NOTE We only accept the user "git", as it stands for anonymous access
        static ref R_SSH_CLONE_URL: Regex = Regex::new(r"^(?P<user>git@)?(?P<host>[^/:]+)((:|/)(?P<path>.+))?$").unwrap();
        static ref R_GIT_HUB_PATH: Regex = Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)\.git$").unwrap();
        static ref R_GIT_LAB_PATH: Regex = Regex::new(r"^/(?P<user>[^/]+)/((?P<structure>[^/]+)/)*(?P<repo>[^/]+)\.git$").unwrap();
        static ref R_BIT_BUCKET_PATH: Regex = (*R_GIT_HUB_PATH).clone();
    }

    let url = match check_public_url(environment, value, true) {
        Ok(url) => url,
        Err(err_orig) => {
            let ssh_value = R_SSH_CLONE_URL.replace(value, "ssh://$host/$path");
            match check_public_url(environment, &ssh_value, true) {
                Ok(url) => url,
                // If also the ssh_value failed to parse,
                // return the error concerning the failed parsing of the original value.
                Err(_err_ssh) => return Err(err_orig), // Err(_err_ssh) => return Err(_err_ssh),
            }
        }
    };

    let hosting_type = eval_hosting_type(environment, &url);
    let host_reg: Option<&Regex> = match hosting_type {
        HostingType::GitHub => Some(&R_GIT_HUB_PATH),
        HostingType::GitLab => Some(&R_GIT_LAB_PATH),
        HostingType::BitBucket => Some(&R_BIT_BUCKET_PATH),
        _ => None, // TODO Implement the others
    };
    check_url_path(value, "repo clone", &url, host_reg)
}

/// See also `sources::try_construct_raw_prefix_url`.
// * https://raw.githubusercontent.com/hoijui/nim-ci/master/.github/workflows/docker.yml
// * https://gitlab.com/OSEGermany/osh-tool/-/raw/master/data/source_extension_formats.csv
// * https://gitlab.com/OSEGermany/osh-tool/raw/master/data/source_extension_formats.csv
// * https://bitbucket.org/Aouatef/master_arbeit/raw/ae4a42a850b359a23da2483eb8f867f21c5382d4/procExData/import.sh
fn validate_repo_raw_versioned_prefix_url(environment: &mut Environment, value: &str) -> Result {
    lazy_static! {
        static ref R_GIT_HUB_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)$").unwrap();
        static ref R_GIT_LAB_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/((?P<structure>[^/]+)/)*(?P<repo>[^/]+)/(-/)?raw$")
                .unwrap();
        static ref R_BIT_BUCKET_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)/raw$").unwrap();
    }

    let url = check_public_url(environment, value, false)?;
    let hosting_type = eval_hosting_type(environment, &url);
    let host_reg: Option<&Regex> = match hosting_type {
        HostingType::GitHub => Some(&R_GIT_HUB_PATH),
        HostingType::GitLab => Some(&R_GIT_LAB_PATH),
        HostingType::BitBucket => Some(&R_BIT_BUCKET_PATH),
        _ => None, // TODO Implement the others
    };
    check_url_path(value, "raw versioned prefix", &url, host_reg)
}

/// See also `sources::try_construct_file_prefix_url`.
fn validate_repo_versioned_file_prefix_url(environment: &mut Environment, value: &str) -> Result {
    lazy_static! {
        static ref R_GIT_HUB_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)/blob$").unwrap();
        static ref R_GIT_LAB_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/((?P<structure>[^/]+)/)*(?P<repo>[^/]+)/(-/)?blob$")
                .unwrap();
        static ref R_BIT_BUCKET_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)/src$").unwrap();
    }

    let url = check_public_url(environment, value, false)?;
    let hosting_type = eval_hosting_type(environment, &url);
    let host_reg: Option<&Regex> = match hosting_type {
        HostingType::GitHub => Some(&R_GIT_HUB_PATH),
        HostingType::GitLab => Some(&R_GIT_LAB_PATH),
        HostingType::BitBucket => Some(&R_BIT_BUCKET_PATH),
        _ => None, // TODO Implement the others
    };
    check_url_path(value, "versioned file prefix", &url, host_reg)
}

/// See also `sources::try_construct_file_prefix_url`.
fn validate_repo_versioned_dir_prefix_url(environment: &mut Environment, value: &str) -> Result {
    lazy_static! {
        static ref R_GIT_HUB_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)/tree$").unwrap();
        static ref R_GIT_LAB_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/((?P<structure>[^/]+)/)*(?P<repo>[^/]+)/(-/)?tree$")
                .unwrap();
        static ref R_BIT_BUCKET_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)/src$").unwrap();
    }

    let url = check_public_url(environment, value, false)?;
    let hosting_type = eval_hosting_type(environment, &url);
    let host_reg: Option<&Regex> = match hosting_type {
        HostingType::GitHub => Some(&R_GIT_HUB_PATH),
        HostingType::GitLab => Some(&R_GIT_LAB_PATH),
        HostingType::BitBucket => Some(&R_BIT_BUCKET_PATH),
        _ => None, // TODO Implement the others
    };
    check_url_path(value, "versioned dir prefix", &url, host_reg)
}

/// See also `sources::try_construct_commit_prefix_url`.
fn validate_repo_commit_prefix_url(environment: &mut Environment, value: &str) -> Result {
    lazy_static! {
        static ref R_GIT_HUB_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)/commit$").unwrap();
        static ref R_GIT_LAB_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/((?P<structure>[^/]+)/)*(?P<repo>[^/]+)/(-/)?commit$")
                .unwrap();
        static ref R_BIT_BUCKET_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)/commits$").unwrap();
    }

    let url = check_public_url(environment, value, false)?;
    let hosting_type = eval_hosting_type(environment, &url);
    let host_reg: Option<&Regex> = match hosting_type {
        HostingType::GitHub => Some(&R_GIT_HUB_PATH),
        HostingType::GitLab => Some(&R_GIT_LAB_PATH),
        HostingType::BitBucket => Some(&R_BIT_BUCKET_PATH),
        _ => None, // TODO Implement the others
    };
    check_url_path(value, "commit prefix", &url, host_reg)
}

fn validate_repo_issues_url(environment: &mut Environment, value: &str) -> Result {
    lazy_static! {
        static ref R_GIT_HUB_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)/issues$").unwrap();
        static ref R_GIT_LAB_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/((?P<structure>[^/]+)/)*(?P<repo>[^/]+)/(-/)?issues$")
                .unwrap();
        static ref R_BIT_BUCKET_PATH: Regex =
            Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)/issues$").unwrap();
    }

    let url = check_public_url(environment, value, false)?;
    let hosting_type = eval_hosting_type(environment, &url);
    let host_reg: Option<&Regex> = match hosting_type {
        HostingType::GitHub => Some(&R_GIT_HUB_PATH),
        HostingType::GitLab => Some(&R_GIT_LAB_PATH),
        HostingType::BitBucket => Some(&R_BIT_BUCKET_PATH),
        _ => None, // TODO Implement the others
    };
    check_url_path(value, "issues", &url, host_reg)
}

fn validate_build_hosting_url(environment: &mut Environment, value: &str) -> Result {
    lazy_static! {
        static ref R_GIT_HUB_HOST: Regex = Regex::new(r"^(?P<user>[^/.]+)\.github\.io$").unwrap();
        static ref R_GIT_LAB_HOST: Regex = Regex::new(r"^(?P<user>[^/.]+)\.gitlab\.io$").unwrap();
        // NOTE BitBucket does not have this feature, it only supports one "page" repo per user, not per repo
    }

    let url = check_public_url(environment, value, false)?;
    let hosting_type = eval_hosting_type_from_hosting_suffix(environment, &url);
    let host_reg: Option<&Regex> = match hosting_type {
        HostingType::GitHub => Some(&R_GIT_HUB_HOST),
        HostingType::GitLab => Some(&R_GIT_LAB_HOST),
        _ => None, // TODO Implement the others (BitBucket does not have pages though, so skip it!)
    };
    check_url_host(value, "build hosting", &url, host_reg)
}

fn validate_name(environment: &mut Environment, value: &str) -> Result {
    check_empty(environment, value, "Project name (human-readable)")
}

fn validate_name_machine_readable(environment: &mut Environment, value: &str) -> Result {
    lazy_static! {
        static ref R_MACHINE_READABLE: Regex = Regex::new(r"^[0-9a-zA-Z_-]+$").unwrap();
    }

    check_empty(environment, value, "Project name (machine-readable)")?;
    if R_MACHINE_READABLE.is_match(value) {
        Ok(None)
    } else {
        Err(Error::BadValue {
            msg: format!(
                "Name is not machine-readable, does not match '{}'",
                R_MACHINE_READABLE.as_str()
            ),
            value: value.to_owned(),
        })
    }
}

fn check_date(environment: &mut Environment, value: &str, date_desc: &str) -> Result {
    if value.is_empty() {
        Err(Error::BadValue {
            msg: format!("{} date can not be empty", date_desc),
            value: value.to_owned(),
        })
    } else if NaiveDateTime::parse_from_str(value, &environment.settings.date_format).is_err()
        && DateTime::parse_from_str(value, &environment.settings.date_format).is_err()
    {
        // log::error!("XXX {}", NaiveDateTime::parse_from_str(value, &environment.settings.date_format).unwrap_err());
        Err(Error::BadValue {
            msg: format!(
                r#"Not a {} date according to the date-format "{}""#,
                date_desc, environment.settings.date_format
            ),
            value: value.to_owned(),
        })
    } else {
        Ok(None)
    }
}

fn validate_version_date(environment: &mut Environment, value: &str) -> Result {
    check_date(environment, value, "version")
}

fn validate_build_date(environment: &mut Environment, value: &str) -> Result {
    check_date(environment, value, "build")
}

fn validate_build_branch(environment: &mut Environment, value: &str) -> Result {
    check_empty(environment, value, "Branch")
}

fn validate_build_tag(environment: &mut Environment, value: &str) -> Result {
    check_empty(environment, value, "Tag")
}

fn validate_build_os(environment: &mut Environment, value: &str) -> Result {
    check_empty(environment, value, "Build OS") // TODO Maybe add a list of known good, and mark the others as Ok(Some(Warning::Unknown))
}

fn validate_build_os_family(environment: &mut Environment, value: &str) -> Result {
    check_empty(environment, value, "Build OS Family")?;
    if constants::VALID_OS_FAMILIES.contains(&value) {
        Ok(None)
    } else {
        // todo!();
        // Err(Error::SuboptimalValue {
        //     msg: "TODO".to_owned(), // TODO
        //     value: value.to_owned(),
        // })
        Err(Error::BadValue {
            msg: format!(
                "Only these values are valid: {}",
                constants::VALID_OS_FAMILIES.join(", ")
            ),
            value: value.to_owned(),
        })
    }
}

fn validate_build_arch(environment: &mut Environment, value: &str) -> Result {
    check_empty(environment, value, "Build arch")?;
    if constants::VALID_ARCHS.contains(&value) {
        Ok(None)
    } else {
        // todo!();
        // Err(Error::SuboptimalValue {
        //     msg: "TODO".to_owned(), // TODO
        //     value: value.to_owned(),
        // })
        Err(Error::BadValue {
            msg: format!(
                "Only these values are valid: {}",
                constants::VALID_ARCHS.join(", ")
            ),
            value: value.to_owned(),
        })
    }
}

fn validate_build_number(environment: &mut Environment, value: &str) -> Result {
    check_empty(environment, value, "Build number")?;
    match value.parse::<i32>() {
        Err(_err) => Ok(Some(Warning::SuboptimalValue {
            msg: "It is generally recommended and assumed that the build number is an integer (a positive, whole number)".to_owned(),
            value: value.to_owned(),
            })),
        Ok(_int_value) => Ok(None)
    }
}

fn validate_ci(environment: &mut Environment, value: &str) -> Result {
    check_empty(environment, value, "CI")?;
    match value {
        "true" => Ok(None),
        &_ => Ok(Some(Warning::SuboptimalValue {
            msg: r#"CI should either be unset or set to "true""#.to_owned(),
            value: value.to_owned(),
        })),
    }
}

#[must_use]
pub fn get(key: Key) -> Validator {
    // TODO This match could be written by a macro
    match key {
        Key::Version => validate_version,
        Key::License => validate_license,
        Key::Licenses => validate_licenses,
        Key::RepoWebUrl => validate_repo_web_url,
        Key::RepoCloneUrl => validate_repo_clone_url,
        Key::RepoRawVersionedPrefixUrl => validate_repo_raw_versioned_prefix_url,
        Key::RepoVersionedFilePrefixUrl => validate_repo_versioned_file_prefix_url,
        Key::RepoVersionedDirPrefixUrl => validate_repo_versioned_dir_prefix_url,
        Key::RepoCommitPrefixUrl => validate_repo_commit_prefix_url,
        Key::RepoIssuesUrl => validate_repo_issues_url,
        Key::Name => validate_name,
        Key::NameMachineReadable => validate_name_machine_readable,
        Key::VersionDate => validate_version_date,
        Key::BuildDate => validate_build_date,
        Key::BuildBranch => validate_build_branch,
        Key::BuildTag => validate_build_tag,
        Key::BuildOs => validate_build_os,
        Key::BuildOsFamily => validate_build_os_family,
        Key::BuildArch => validate_build_arch,
        Key::BuildHostingUrl => validate_build_hosting_url,
        Key::BuildNumber => validate_build_number,
        Key::Ci => validate_ci,
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom:
    // importing names from outer (for mod tests) scope.
    use super::*;

    lazy_static! {
        static ref VE_ALMOST_USABLE_VALUE: Error = Error::AlmostUsableValue {
            msg: String::default(),
            value: String::default(),
        };
        static ref VE_MISSING: Error = Error::Missing {};
        static ref VE_BAD_VALUE: Error = Error::BadValue {
            msg: String::default(),
            value: String::default(),
        };
        static ref VW_SUBOPTIMAL_VALUE: Warning = Warning::SuboptimalValue {
            msg: String::default(),
            value: String::default(),
        };
    }

    fn variant_eq<T>(a: &T, b: &T) -> bool {
        std::mem::discriminant(a) == std::mem::discriminant(b)
    }
    // %Y-%m-%d %H:%M:%S\"", value: "2021-09-21 06:27:37
    // #[test]
    // fn date_time() -> Result<(), chrono::ParseError> {

    //     let custom = DateTime::parse_from_str("2021-09-21 06:27:37", "%Y-%m-%d %H:%M:%S")?;
    //     println!("{}", custom);
    //     let custom = chrono::NaiveDateTime::parse_from_str("2021-09-21 06:27:37", "%Y-%m-%d %H:%M:%S")?;
    //     println!("{}", custom);

    //     Ok(())
    // }

    fn is_optimal(res: Result) -> bool {
        variant_eq(&res.unwrap(), &None)
    }

    fn is_suboptimal(res: Result) -> bool {
        if let Some(err) = res.unwrap() {
            variant_eq(&err, &VW_SUBOPTIMAL_VALUE)
        } else {
            false
        }
    }

    fn is_almost_usable(res: Result) -> bool {
        variant_eq(&res.unwrap_err(), &VE_ALMOST_USABLE_VALUE)
    }

    fn is_missing_err(res: Result) -> bool {
        variant_eq(&res.unwrap_err(), &VE_MISSING)
    }

    fn is_bad_value(res: Result) -> bool {
        variant_eq(&res.unwrap_err(), &VE_BAD_VALUE)
    }

    #[test]
    fn test_validate_version() {
        let mut environment = Environment::stub();
        let full_sha = "cf73ea34fcc785b1ac44ffb20d655c917e77c83d";
        let double_sha =
            "cf73ea34fcc785b1ac44ffb20d655c917e77c83dcf73ea34fcc785b1ac44ffb20d655c917e77c83d";

        // Good cases
        for sha_length in 7..full_sha.len() {
            assert!(is_suboptimal(validate_version(
                &mut environment,
                &full_sha[0..sha_length],
            )));
        }
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "gad8f844"
        )));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "gad8f844-dirty"
        )));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "gad8f844-broken"
        )));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "gad8f844-dirty-broken"
        )));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "gad8f844-broken-dirty"
        )));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "0.1.19-12-gad8f844"
        )));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "0.1.19-12-gad8f844-dirty"
        )));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "0.1.19-12-gad8f844-broken"
        )));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "0.1.19-12-gad8f844-dirty-broken"
        )));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "0.1.19-12-gad8f844-broken-dirty"
        )));
        assert!(is_optimal(validate_version(&mut environment, "0.1.19")));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "0.1.19-dirty"
        )));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "0.1.19-broken"
        )));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "0.1.19-dirty-broken"
        )));
        assert!(is_suboptimal(validate_version(
            &mut environment,
            "0.1.19-broken-dirty"
        )));

        // Bad cases
        assert!(is_missing_err(validate_version(&mut environment, "")));
        assert!(is_bad_value(validate_version(&mut environment, "gabcdefg")));
        // Too short SHAs
        for sha_length in 1..7 {
            assert!(is_bad_value(validate_version(
                &mut environment,
                &full_sha[0..sha_length],
            )));
        }
        // Too long SHAs
        for sha_length in full_sha.len() + 1..double_sha.len() {
            assert!(is_bad_value(validate_version(
                &mut environment,
                &double_sha[0..sha_length],
            )));
        }
        // TODO Add some more bad cases. producing various different errors
    }

    #[test]
    fn test_validate_license() {
        let mut environment = Environment::stub();
        assert!(is_optimal(validate_license(&mut environment, "GPL-3.0")));
        assert!(is_optimal(validate_license(
            &mut environment,
            "GPL-3.0-or-later"
        )));
        assert!(is_optimal(validate_license(&mut environment, "GPL-2.0")));
        assert!(is_optimal(validate_license(
            &mut environment,
            "GPL-2.0-or-later"
        )));
        assert!(is_optimal(validate_license(&mut environment, "AGPL-3.0")));
        assert!(is_optimal(validate_license(
            &mut environment,
            "AGPL-3.0-or-later"
        )));
        assert!(is_optimal(validate_license(&mut environment, "CC0-1.0")));
        assert!(is_suboptimal(validate_license(&mut environment, "CC0-2.0")));
        assert!(is_suboptimal(validate_license(&mut environment, "CC02.0")));
        assert!(is_suboptimal(validate_license(&mut environment, "GPL")));
        assert!(is_suboptimal(validate_license(&mut environment, "AGPL")));
        assert!(is_suboptimal(validate_license(
            &mut environment,
            "Some Unknown License"
        )));
        assert!(validate_license(&mut environment, "").is_err()); // TODO Rather check the details of the Err value!
                                                                  // todo!(); // TODO Add some more bad cases; Producing different errors
    }

    #[test]
    fn test_validate_repo_versioned_dir_prefix_url() -> std::result::Result<(), Error> {
        let mut environment = Environment::stub();
        // assert!(validate_repo_versioned_web_url(&mut environment, "https://github.com/hoijui/projvar/tree/525b3c9b8962dd02aab6ea867eebdee3719a6634")?.is_ok());
        validate_repo_versioned_dir_prefix_url(
            &mut environment,
            "https://github.com/hoijui/projvar/tree",
        )?;
        Ok(())
    }
}
