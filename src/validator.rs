// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::environment::Environment;
use crate::var::Key;
use chrono::{DateTime, NaiveDateTime};
use clap::lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;
use url::{Host, Url};

pub type Validator =
    fn(&mut Environment, &str) -> Result<Option<ValidationWarning>, ValidationError>;

// See these resources for implement our own, custom errors
// accoridng to rust best practises for errors (and error handling):
// * good, simple intro:
//   <https://nick.groenen.me/posts/rust-error-handling/>
// * very nice, extensive, detailed example:
//   https://www.lpalmieri.com/posts/error-handling-rust/#removing-the-boilerplate-with-thiserror

/// This enumerates all possible errors returned by this module.
#[derive(Error, Debug)]
pub enum ValidationWarning {
    /// A non-required properties value could not be evaluated;
    /// no source returned a (valid) value for it.
    #[error("No value found for a property")]
    Missing,

    /// The evaluated value is usable, but not optimal.
    #[error("The value '{value}' is usable, but not optimal - {msg}")]
    SuboptimalValue { msg: String, value: String },
}

/// This enumerates all possible errors returned by this module.
#[derive(Error, Debug)]
pub enum ValidationError {
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
    IOError(#[from] std::io::Error),
}

pub fn validate_version(
    _environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    lazy_static! {
        // The official SemVer regex as of September 2021, taken from
        // https://semver.org/#is-there-a-suggested-regular-expression-regex-to-check-a-semver-string
        // TODO Think of what to do if we have a "v" prefix, as in "v1.2.3" -> best: remove it, but where.. a kind of pre-validator function?
        static ref R_SEM_VERS: Regex = Regex::new(r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$").unwrap();
        static ref R_GIT_VERS: Regex = Regex::new(r"^((g[0-9a-f]{7})|((0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)))(-(0|[1-9]\d*)-(g[0-9a-f]{7}))?((-dirty(-broken)?)|-broken(-dirty)?)?$").unwrap();
        static ref R_GIT_SHA: Regex = Regex::new(r"^[0-9a-f]{7,40}$").unwrap();
        static ref R_UNKNOWN_VERS: Regex = Regex::new(r"^($|#|//)").unwrap();
    }
    if R_SEM_VERS.is_match(value) {
        Ok(None)
    } else if R_GIT_VERS.is_match(value) {
        Ok(Some(ValidationWarning::SuboptimalValue {
            msg: "This version is technically good, but not a release-version (i.e., does not look so nice)".to_owned(),
            value: value.to_owned(),
        }))
    } else if R_GIT_SHA.is_match(value) {
        Ok(Some(ValidationWarning::SuboptimalValue {
            msg:
                "This version is technically ok, but not a release-version, and not human-readable"
                    .to_owned(),
            value: value.to_owned(),
        }))
    } else if R_UNKNOWN_VERS.is_match(value) {
        // Ok(Some(ValidationError::Missing))
        Err(ValidationError::Missing)
    } else {
        Err(ValidationError::BadValue {
            msg: "Not a valid version".to_owned(),
            value: value.to_owned(),
        })
    }
}

lazy_static! {
    static ref SPDX_IDENTS: Vec<&'static str> = ["CC0-1.0", "GPL-3.0-or-later"].to_vec(); // TODO HACK ...
    // TODO use an SPDX repo as submodule that contains the list of supported license idenfiers and compare against them
    // TODO see: https://github.com/spdx/license-list-XML/issues/1335
}

pub fn validate_license(
    _environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    if SPDX_IDENTS.contains(&value) {
        Ok(None)
    } else {
        Ok(Some(ValidationWarning::SuboptimalValue {
            msg: "Not a recognized SPDX license identifier".to_owned(),
            value: value.to_owned(),
        }))
    }
}

fn check_public_url(
    _environment: &mut Environment,
    value: &str,
    allow_ssh: bool,
) -> Result<Url, ValidationError> {
    match Url::parse(value) {
        Err(_err) => Err(ValidationError::BadValue {
            msg: "Not a valid URL".to_owned(),
            value: value.to_owned(),
        }),
        Ok(url) => {
            if !["http", "https"].contains(&url.scheme()) && !(allow_ssh && url.scheme() == "ssh") {
                Err(ValidationError::AlmostUsableValue {
                    msg: format!(
                        "Should use one of these as protocol(scheme): [http, https{}]",
                        if allow_ssh { ", ssh" } else { "" }
                    ),
                    value: value.to_owned(),
                })
            } else if url.username() != "" {
                Err(ValidationError::AlmostUsableValue {
                    msg: format!(
                        "Should be anonymous access, but specifies a user-name: {}",
                        url.username()
                    ),
                    value: value.to_owned(),
                })
            } else if let Some(_pw) = url.password() {
                Err(ValidationError::AlmostUsableValue {
                    msg: "Should be anonymous access, but contains a password".to_owned(),
                    value: value.to_owned(),
                })
            } else if let Some(query) = url.query() {
                Err(ValidationError::AlmostUsableValue {
                    msg: format!(
                        "Should be a simple URL, but uses query arguments: {}",
                        query
                    ),
                    value: value.to_owned(),
                })
            } else if let Some(fragment) = url.fragment() {
                Err(ValidationError::AlmostUsableValue {
                    msg: format!("Should be a simple URL, but uses a fragment: {}", fragment),
                    value: value.to_owned(),
                })
            } else {
                Ok(url)
            }
        }
    }
}

pub fn validate_repo_web_url(
    environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    let url = check_public_url(environment, value, false)?;
    if url.host() == Some(Host::Domain("github.com"))
        && url
            .path_segments()
            .ok_or_else(|| ValidationError::AlmostUsableValue {
                msg: "Missing a path for GitHub.com".to_owned(),
                value: value.to_owned(),
            })?
            .count()
            != 2
    {
        Err(ValidationError::AlmostUsableValue {
            msg: format!(
                r#"On GitHub.com, the path part of the web URL of a repo should be of the form "user/repo", but it is "{}""#,
                url.path()
            ),
            value: value.to_owned(),
        })
    } else if url.host() == Some(Host::Domain("gitlab.com"))
        && url
            .path_segments()
            .ok_or_else(|| ValidationError::AlmostUsableValue {
                msg: "Missing a path for GitLab.com".to_owned(),
                value: value.to_owned(),
            })?
            .count()
            < 2
    {
        Err(ValidationError::AlmostUsableValue {
            msg: format!(
                r#"On GitLab.com, the path part of the web URL of a repo should be of the form "user/repo" or "user/.../repo", but it is "{}""#,
                url.path()
            ),
            value: value.to_owned(),
        })
    } else {
        Ok(None)
    }
}

pub fn validate_repo_frozen_web_url(
    environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    lazy_static! {
        // TODO Check if really both of these providers support the '-' part in ".../user/repo/-/tree/34f8e45".
        static ref R_GIT_HUB_PATH: Regex = Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)/(-/)?(tree)/(?P<commit>[^/]+)$").unwrap();
        static ref R_GIT_LAB_PATH: Regex = Regex::new(r"^/(?P<user>[^/]+)/((?P<structure>[^/]+)/)*(?P<repo>[^/]+)/(-/)?(tree)/(?P<commit>[^/]+)$").unwrap();
    }

    let url = check_public_url(environment, value, false)?;
    if url.host() == Some(Host::Domain("github.com")) && !R_GIT_HUB_PATH.is_match(url.path()) {
        Err(ValidationError::AlmostUsableValue {
            msg: format!(
                r#"For GitHub.com, this path part of the frozen web URL is invalid: "{}"; it should match "{}""#,
                url.path(),
                R_GIT_HUB_PATH.as_str()
            ),
            value: value.to_owned(),
        })
    } else if url.host() == Some(Host::Domain("gitlab.com")) && !R_GIT_LAB_PATH.is_match(url.path())
    {
        Err(ValidationError::AlmostUsableValue {
            msg: format!(
                r#"For GitLab.com, this path part of the frozen web URL is invalid: "{}"; it should match "{}""#,
                url.path(),
                R_GIT_LAB_PATH.as_str()
            ),
            value: value.to_owned(),
        })
    } else {
        Ok(None)
    }
}

pub fn validate_repo_clone_url(
    environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    lazy_static! {
        // NOTE We only accept the user "git", as it stands for anonymous access
        static ref R_SSH_CLONE_TO_URL: Regex = Regex::new(r"^(?P<user>git@)?(?P<host>[^/:]+)((:|/)(?P<path>.+))?$").unwrap();
    }

    let url = match check_public_url(environment, value, true) {
        Ok(url) => url,
        Err(err_orig) => {
            let ssh_value = R_SSH_CLONE_TO_URL.replace(value, "ssh://$host/$path");
            match check_public_url(environment, &ssh_value, true) {
                Ok(url) => url,
                // If also the ssh_value failed to parse,
                // return the error concerning the failed parsing of the original value.
                Err(_err_ssh) => Err(err_orig)?, // Err(_err_ssh) => Err(_err_ssh)?
            }
        }
    };
    if url.host() == Some(Host::Domain("github.com"))
        && url
            .path_segments()
            .ok_or_else(|| ValidationError::AlmostUsableValue {
                msg: "Missing a path for GitHub.com".to_owned(),
                value: value.to_owned(),
            })?
            .count()
            != 2
        && !url.path().ends_with(".git")
    {
        Err(ValidationError::AlmostUsableValue {
            msg: format!(
                r#"On GitHub.com, the path part of the web URL of a repo should be of the form "user/repo", but it is "{}""#,
                url.path()
            ),
            value: value.to_owned(),
        })
    } else if url.host() == Some(Host::Domain("gitlab.com"))
        && url
            .path_segments()
            .ok_or_else(|| ValidationError::AlmostUsableValue {
                msg: "Missing a path for GitLab.com".to_owned(),
                value: value.to_owned(),
            })?
            .count()
            < 2
        && !url.path().ends_with(".git")
    {
        Err(ValidationError::AlmostUsableValue {
            msg: format!(
                r#"On GitLab.com, the path part of the web URL of a repo should be of the form "user/repo" or "user/.../repo", but it is "{}""#,
                url.path()
            ),
            value: value.to_owned(),
        })
    } else {
        Ok(None)
    }
}

pub fn validate_repo_issues_url(
    environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    lazy_static! {
        // TODO Check if really both of these providers support the '-' part in ".../user/repo/-/issues".
        static ref R_GIT_HUB_PATH: Regex = Regex::new(r"^/(?P<user>[^/]+)/(?P<repo>[^/]+)/(-/)?(issues)$").unwrap();
        static ref R_GIT_LAB_PATH: Regex = Regex::new(r"^/(?P<user>[^/]+)/((?P<structure>[^/]+)/)*(?P<repo>[^/]+)/(-/)?(issues)$").unwrap();
    }

    let url = check_public_url(environment, value, false)?;
    if url.host() == Some(Host::Domain("github.com")) && !R_GIT_HUB_PATH.is_match(url.path()) {
        Err(ValidationError::AlmostUsableValue {
            msg: format!(
                r#"For GitHub.com, this path part of the issues URL is invalid: "{}"; it should match "{}""#,
                url.path(),
                R_GIT_HUB_PATH.as_str()
            ),
            value: value.to_owned(),
        })
    } else if url.host() == Some(Host::Domain("gitlab.com")) && !R_GIT_LAB_PATH.is_match(url.path())
    {
        Err(ValidationError::AlmostUsableValue {
            msg: format!(
                r#"For GitLab.com, this path part of the issues URL is invalid: "{}"; it should match "{}""#,
                url.path(),
                R_GIT_LAB_PATH.as_str()
            ),
            value: value.to_owned(),
        })
    } else {
        Ok(None)
    }
}

pub fn validate_build_hosting_url(
    environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    lazy_static! {
        static ref R_GIT_HUB_HOST: Regex = Regex::new(r"^(?P<user>[^/.]+)\.github\.io$").unwrap();
        static ref R_GIT_LAB_HOST: Regex = Regex::new(r"^(?P<user>[^/.]+)\.gitlab\.io$").unwrap();
    }

    let url = check_public_url(environment, value, false)?;
    if let Some(host) = url.host() {
        let host_str = host.to_string();
        if host_str.ends_with(".github.io") && !R_GIT_HUB_HOST.is_match(&host_str) {
            Err(ValidationError::AlmostUsableValue {
                msg: format!(
                    r#"For GitHub.com, this host for the build hosting URL is invalid: "{}"; it should match "{}""#,
                    host,
                    R_GIT_HUB_HOST.as_str()
                ),
                value: value.to_owned(),
            })
        } else if host_str.ends_with(".gitlab.io") && !R_GIT_LAB_HOST.is_match(&host_str) {
            Err(ValidationError::AlmostUsableValue {
                msg: format!(
                    r#"For GitLab.com, this path part of the frozen web URL is invalid: "{}"; it should match "{}""#,
                    host,
                    R_GIT_LAB_HOST.as_str()
                ),
                value: value.to_owned(),
            })
        } else {
            Ok(None)
        }
    } else {
        Err(ValidationError::BadValue {
            msg: "No host; seems not to be a commonly accessible URL".to_owned(),
            value: value.to_owned(),
        })
    }
}

pub fn validate_name(
    _environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    if value.is_empty() {
        Err(ValidationError::BadValue {
            msg: "Project name can not be empty".to_owned(),
            value: value.to_owned(),
        })
    } else {
        Ok(None)
    }
}

pub fn validate_version_date(
    environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    if value.is_empty() {
        Err(ValidationError::BadValue {
            msg: "Date can not be empty".to_owned(),
            value: value.to_owned(),
        })
    } else if NaiveDateTime::parse_from_str(value, &environment.settings.date_format).is_err()
        && DateTime::parse_from_str(value, &environment.settings.date_format).is_err()
    {
        // log::error!("XXX {}", NaiveDateTime::parse_from_str(value, &environment.settings.date_format).unwrap_err());
        Err(ValidationError::BadValue {
            msg: format!(
                r#"Not a date according to the date-format "{}""#,
                environment.settings.date_format
            ),
            value: value.to_owned(),
        })
    } else {
        Ok(None)
    }
}

pub fn validate_build_date(
    environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    validate_version_date(environment, value)
}

pub fn validate_build_branch(
    _environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    if value.is_empty() {
        Err(ValidationError::BadValue {
            msg: "Branch can not be empty".to_owned(),
            value: value.to_owned(),
        })
    } else {
        Ok(None)
    }
}

pub fn validate_build_tag(
    _environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    if value.is_empty() {
        Err(ValidationError::BadValue {
            msg: "Tag can not be empty".to_owned(),
            value: value.to_owned(),
        })
    } else {
        Ok(None)
    }
}

pub fn validate_build_ident(
    _environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    if value.is_empty() {
        Err(ValidationError::BadValue {
            msg: "Build identifier can not be empty".to_owned(),
            value: value.to_owned(),
        })
    } else {
        Ok(None)
    }
}

pub fn validate_build_os(
    _environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    if value.is_empty() {
        Err(ValidationError::BadValue {
            msg: "Build OS can not be empty".to_owned(),
            value: value.to_owned(),
        })
    } else {
        Ok(None)
    }
}

const VALID_OS_FAMILIES: &[&str] = &["linux", "unix", "bsd", "osx", "windows"]; // TODO

pub fn validate_build_os_family(
    _environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    if value.is_empty() {
        Err(ValidationError::BadValue {
            msg: "Build OS Family can not be empty".to_owned(),
            value: value.to_owned(),
        })
    } else if VALID_OS_FAMILIES.contains(&value) {
        Ok(None)
    } else {
        // todo!();
        // Err(ValidationError::SuboptimalValue {
        //     msg: "TODO".to_owned(), // TODO
        //     value: value.to_owned(),
        // })
        Err(ValidationError::BadValue {
            msg: format!(
                "Only these values are valid: {}",
                VALID_OS_FAMILIES.join(", ")
            ),
            value: value.to_owned(),
        })
    }
}

const VALID_ARCHS: &[&str] = &["x86", "x86_64", "arm", "arm64"]; // TODO

pub fn validate_build_arch(
    _environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    if VALID_ARCHS.contains(&value) {
        Ok(None)
    } else if value.is_empty() {
        Err(ValidationError::BadValue {
            msg: "Build arch can not be empty".to_owned(),
            value: value.to_owned(),
        })
    } else {
        // todo!();
        // Err(ValidationError::SuboptimalValue {
        //     msg: "TODO".to_owned(), // TODO
        //     value: value.to_owned(),
        // })
        Err(ValidationError::BadValue {
            msg: format!("Only these values are valid: {}", VALID_ARCHS.join(", ")),
            value: value.to_owned(),
        })
    }
}

pub fn validate_build_number(
    _environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    if value.is_empty() {
        Err(ValidationError::BadValue {
            msg: "Build number can not be empty".to_owned(),
            value: value.to_owned(),
        })
    } else {
        match value.parse::<i32>() {
            Err(_err) => Ok(Some(ValidationWarning::SuboptimalValue {
                msg: "It is generally recommended and assumed that the build number is an integer (a positive, whole number)".to_owned(),
                value: value.to_owned(),
             })),
            Ok(_int_value) => Ok(None)
        }
    }
}

pub fn validate_ci(
    _environment: &mut Environment,
    value: &str,
) -> Result<Option<ValidationWarning>, ValidationError> {
    if value.is_empty() {
        Err(ValidationError::BadValue {
            msg: "CI can not be empty".to_owned(),
            value: value.to_owned(),
        })
    } else {
        match value {
            "true" => Ok(None),
            &_ => Ok(Some(ValidationWarning::SuboptimalValue {
                msg: r#"CI should either be unset or set to "true""#.to_owned(),
                value: value.to_owned(),
            })),
        }
    }
}

#[must_use]
pub fn get(key: &Key) -> Validator {
    // TODO This match could be written by a macro
    match key {
        Key::Version => validate_version,
        Key::License => validate_license,
        Key::RepoWebUrl => validate_repo_web_url,
        Key::RepoFrozenWebUrl => validate_repo_frozen_web_url,
        Key::RepoCloneUrl => validate_repo_clone_url,
        Key::RepoIssuesUrl => validate_repo_issues_url,
        Key::Name => validate_name,
        Key::VersionDate => validate_version_date,
        Key::BuildDate => validate_build_date,
        Key::BuildBranch => validate_build_branch,
        Key::BuildTag => validate_build_tag,
        Key::BuildIdent => validate_build_ident,
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
        static ref VE_ALMOST_USABLE_VALUE: ValidationError = ValidationError::AlmostUsableValue {
            msg: String::default(),
            value: String::default()
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

    #[test]
    fn test_validate_version() {
        let mut environment = Environment::stub();
        assert!(validate_version(&mut environment, "gad8f844").is_ok());
        assert!(validate_version(&mut environment, "gad8f844-dirty").is_ok());
        assert!(validate_version(&mut environment, "gad8f844-broken").is_ok());
        assert!(validate_version(&mut environment, "gad8f844-dirty-broken").is_ok());
        assert!(validate_version(&mut environment, "gad8f844-broken-dirty").is_ok());
        assert!(validate_version(&mut environment, "0.1.19-12-gad8f844").is_ok());
        assert!(validate_version(&mut environment, "0.1.19-12-gad8f844-dirty").is_ok());
        assert!(validate_version(&mut environment, "0.1.19-12-gad8f844-broken").is_ok());
        assert!(validate_version(&mut environment, "0.1.19-12-gad8f844-dirty-broken").is_ok());
        assert!(validate_version(&mut environment, "0.1.19-12-gad8f844-broken-dirty").is_ok());
        assert!(validate_version(&mut environment, "0.1.19").is_ok());
        assert!(validate_version(&mut environment, "0.1.19-dirty").is_ok());
        assert!(validate_version(&mut environment, "0.1.19-broken").is_ok());
        assert!(validate_version(&mut environment, "0.1.19-dirty-broken").is_ok());
        assert!(validate_version(&mut environment, "0.1.19-broken-dirty").is_ok());
        todo!(); // TODO Add some bad cases too; Producing various different errors
    }

    #[test]
    fn test_validate_license() {
        let mut environment = Environment::stub();
        assert!(validate_version(&mut environment, "GPL-3.0").is_ok());
        assert!(validate_version(&mut environment, "GPL-3.0-or-later").is_ok());
        assert!(validate_version(&mut environment, "GPL-2.0").is_ok());
        assert!(validate_version(&mut environment, "GPL-2.0-or-later").is_ok());
        assert!(validate_version(&mut environment, "AGPL-3.0").is_ok());
        assert!(validate_version(&mut environment, "AGPL-3.0-or-later").is_ok());
        assert!(validate_version(&mut environment, "CC0-1.0").is_ok());
        assert!(variant_eq(
            &validate_version(&mut environment, "CC0-2.0").unwrap_err(),
            &VE_ALMOST_USABLE_VALUE
        ));
        assert!(variant_eq(
            &validate_version(&mut environment, "CC0").unwrap_err(),
            &VE_ALMOST_USABLE_VALUE
        ));
        assert!(variant_eq(
            &validate_version(&mut environment, "GPL").unwrap_err(),
            &VE_ALMOST_USABLE_VALUE
        ));
        assert!(variant_eq(
            &validate_version(&mut environment, "AGPL").unwrap_err(),
            &VE_ALMOST_USABLE_VALUE
        ));
        assert!(variant_eq(
            &validate_version(&mut environment, "GPL").unwrap_err(),
            &VE_ALMOST_USABLE_VALUE
        ));
        assert!(variant_eq(
            &validate_version(&mut environment, "GPL").unwrap_err(),
            &VE_ALMOST_USABLE_VALUE
        ));
        // let result = validate_version(&mut environment, "GPL");
        // assert!(is_error_type!(result, ValidationError::AlmostUsableValue));

        todo!(); // TODO Add some more bad cases; Producing different errors
    }

    #[test]
    fn test_validate_repo_frozen_web_url() -> Result<(), ValidationError> {
        let mut environment = Environment::stub();
        // assert!(validate_repo_frozen_web_url(&mut environment, "https://github.com/hoijui/projvar/tree/525b3c9b8962dd02aab6ea867eebdee3719a6634")?.is_ok());
        validate_repo_frozen_web_url(
            &mut environment,
            "https://github.com/hoijui/projvar/tree/525b3c9b8962dd02aab6ea867eebdee3719a6634",
        )?;
        Ok(())
    }
}
