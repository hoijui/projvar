// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::str::FromStr;

use crate::tools::git::TransferProtocol;
use crate::tools::git_clone_url;
use crate::tools::git_hosting_provs::{HostingType, PublicSite};
use chrono::DateTime;
use thiserror::Error;

use lazy_static::lazy_static;
use regex::Regex;
use url::Url;

use crate::environment::Environment;
use crate::var::Key;
use crate::{constants, std_error};

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
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// Represents all other cases of `std_error::Error`.
    #[error(transparent)]
    Std(#[from] std_error::Error),

    /// Represents time parsing errors
    #[error(transparent)]
    DateTime(#[from] chrono::ParseError),

    /// Represents all other cases of `std::error::Error`.
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
    // /// Represents all other errors, especially those not fitting any of the above,
    // /// and which do not derive from `std::error::Error`.
    // #[error("No info about the errror is available")]
    // Empty {
    //     key: Key,
    //     msg: String,
    //     input: String,
    // },
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
            .last()
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

fn web_url_match(
    _environment: &Environment,
    web_url: &str,
    key: Key,
    matcher: &dyn Fn(Url) -> Res,
) -> Res {
    match Url::parse(web_url) {
        Err(err) => Err(Error::BadInputValueErr {
            key,
            msg: "Not a valid web URL".to_owned(),
            input: web_url.to_owned(),
            source: Box::new(err),
        }),
        Ok(url) => matcher(url),
    }
}

/// Tries to construct the issues URL
/// from the repo web URL property of a variable source.
/// See also [`crate::validator::validate_repo_issues_url`].
///
/// NOTE: This currently only works for github.com and gitlab.com!
///
/// for example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use projvar::value_conversions::web_url_to_issues_url;
/// # use projvar::environment::Environment;
/// # let environment = Environment::stub();
/// assert_eq!(
///     web_url_to_issues_url(&environment, "https://github.com/hoijui/kicad-text-injector/")?,
///     Some("https://github.com/hoijui/kicad-text-injector/issues".to_owned())
/// );
/// assert_eq!(
///     web_url_to_issues_url(&environment, "https://gitlab.com/hoijui/kicad-text-injector")?,
///     Some("https://gitlab.com/hoijui/kicad-text-injector/-/issues".to_owned())
/// );
/// assert_eq!(
///     web_url_to_issues_url(&environment, "https://gitlab.com/hoijui/some-group/kicad-text-injector/")?,
///     Some("https://gitlab.com/hoijui/some-group/kicad-text-injector/-/issues".to_owned())
/// );
/// assert_eq!(
///     web_url_to_issues_url(&environment, "https://gitlab.com/hoijui/some-group/kicad-text-injector")?,
///     Some("https://gitlab.com/hoijui/some-group/kicad-text-injector/-/issues".to_owned())
/// );
/// # Ok(())
/// # }
/// ```
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
    web_url_match(environment, web_url, Key::RepoIssuesUrl, &|mut url| {
        Ok(match environment.settings.hosting_type(&url) {
            HostingType::BitBucket | HostingType::GitHub => {
                url.set_path(&format!("/{}/issues", trim_char(url.path(), '/')));
                Some(url.to_string())
            }
            HostingType::GitLab => {
                url.set_path(&format!("/{}/-/issues", trim_char(url.path(), '/')));
                Some(url.to_string())
            }
            _ => None, // TODO Implement the others!
        })
    })
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
    web_url_match(
        environment,
        web_url,
        Key::RepoRawVersionedPrefixUrl,
        &|mut url| {
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
        },
    )
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
    web_url_match(
        environment,
        web_url,
        Key::RepoVersionedFilePrefixUrl,
        &|mut url| {
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
        },
    )
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
    web_url_match(
        environment,
        web_url,
        Key::RepoVersionedDirPrefixUrl,
        &|mut url| {
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
        },
    )
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
    web_url_match(
        environment,
        web_url,
        Key::RepoCommitPrefixUrl,
        &|mut url| {
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
        },
    )
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
///
/// for example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use projvar::tools::git::TransferProtocol;
/// # use projvar::value_conversions::clone_url_conversion;
/// # use projvar::environment::Environment;
/// # let environment = Environment::stub();
/// assert_eq!(
///     clone_url_conversion("git@github.com:hoijui/kicad-text-injector.git", &environment, TransferProtocol::Https)?,
///     Some("https://github.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@github.com:hoijui/kicad-text-injector", &environment, TransferProtocol::Https)?,
///     Some("https://github.com/hoijui/kicad-text-injector".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://github.com/hoijui/kicad-text-injector.git", &environment, TransferProtocol::Https)?,
///     Some("https://github.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://github.com/hoijui/kicad-text-injector", &environment, TransferProtocol::Https)?,
///     Some("https://github.com/hoijui/kicad-text-injector".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@gitlab.com:hoijui/kicad-text-injector.git", &environment, TransferProtocol::Https)?,
///     Some("https://gitlab.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://gitlab.com/hoijui/kicad-text-injector.git", &environment, TransferProtocol::Https)?,
///     Some("https://gitlab.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@bitbucket.org:Aouatef/master_arbeit.git", &environment, TransferProtocol::Https)?,
///     Some("https://bitbucket.org/Aouatef/master_arbeit.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://hoijui@bitbucket.org/Aouatef/master_arbeit.git", &environment, TransferProtocol::Https)?,
///     Some("https://bitbucket.org/Aouatef/master_arbeit.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://git.sr.ht/~sircmpwn/sr.ht-docs", &environment, TransferProtocol::Https)?,
///     Some("https://git.sr.ht/~sircmpwn/sr.ht-docs".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@git.sr.ht:~sircmpwn/sr.ht-docs", &environment, TransferProtocol::Https)?,
///     Some("https://git.sr.ht/~sircmpwn/sr.ht-docs".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("ssh://git@git.sr.ht:~sircmpwn/sr.ht-docs", &environment, TransferProtocol::Https)?,
///     Some("https://git.sr.ht/~sircmpwn/sr.ht-docs".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://rocketgit.com/user/hoijui/rs-test", &environment, TransferProtocol::Https)?,
///     Some("https://rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("ssh://rocketgit@ssh.rocketgit.com/user/hoijui/rs-test", &environment, TransferProtocol::Https)?,
///     Some("https://rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git://git.rocketgit.com/user/hoijui/rs-test", &environment, TransferProtocol::Https)?,
///     Some("https://rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://repo.or.cz/girocco.git", &environment, TransferProtocol::Https)?,
///     Some("https://repo.or.cz/girocco.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("ssh://repo.or.cz/girocco.git", &environment, TransferProtocol::Https)?,
///     Some("https://repo.or.cz/girocco.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git://repo.or.cz/girocco.git", &environment, TransferProtocol::Https)?,
///     Some("https://repo.or.cz/girocco.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@github.com:hoijui/kicad-text-injector.git", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://git@github.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@github.com:hoijui/kicad-text-injector", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://git@github.com/hoijui/kicad-text-injector".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://github.com/hoijui/kicad-text-injector.git", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://git@github.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://github.com/hoijui/kicad-text-injector", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://git@github.com/hoijui/kicad-text-injector".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@gitlab.com:hoijui/kicad-text-injector.git", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://git@gitlab.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://gitlab.com/hoijui/kicad-text-injector.git", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://git@gitlab.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@bitbucket.org:Aouatef/master_arbeit.git", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://git@bitbucket.org/Aouatef/master_arbeit.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://hoijui@bitbucket.org/Aouatef/master_arbeit.git", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://git@bitbucket.org/Aouatef/master_arbeit.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://git.sr.ht/~sircmpwn/sr.ht-docs", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://git@git.sr.ht:~sircmpwn/sr.ht-docs".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@git.sr.ht:~sircmpwn/sr.ht-docs", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://git@git.sr.ht:~sircmpwn/sr.ht-docs".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("ssh://git@git.sr.ht:~sircmpwn/sr.ht-docs", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://git@git.sr.ht:~sircmpwn/sr.ht-docs".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://rocketgit.com/user/hoijui/rs-test", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://rocketgit@ssh.rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("ssh://rocketgit@ssh.rocketgit.com/user/hoijui/rs-test", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://rocketgit@ssh.rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git://git.rocketgit.com/user/hoijui/rs-test", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://rocketgit@ssh.rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://repo.or.cz/girocco.git", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://repo.or.cz/girocco.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("ssh://repo.or.cz/girocco.git", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://repo.or.cz/girocco.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git://repo.or.cz/girocco.git", &environment, TransferProtocol::Ssh)?,
///     Some("ssh://repo.or.cz/girocco.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://rocketgit.com/user/hoijui/rs-test", &environment, TransferProtocol::Git)?,
///     Some("git://git.rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("ssh://rocketgit@ssh.rocketgit.com/user/hoijui/rs-test", &environment, TransferProtocol::Git)?,
///     Some("git://git.rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git://git.rocketgit.com/user/hoijui/rs-test", &environment, TransferProtocol::Git)?,
///     Some("git://git.rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://repo.or.cz/girocco.git", &environment, TransferProtocol::Git)?,
///     Some("git://repo.or.cz/girocco.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("ssh://repo.or.cz/girocco.git", &environment, TransferProtocol::Git)?,
///     Some("git://repo.or.cz/girocco.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git://repo.or.cz/girocco.git", &environment, TransferProtocol::Git)?,
///     Some("git://repo.or.cz/girocco.git".to_owned())
/// );
/// # Ok(())
/// # }
/// ```
pub fn clone_url_conversion(
    any_clone_url: &str,
    environment: &Environment,
    protocol: TransferProtocol,
) -> Res {
    lazy_static! {
        static ref R_HOST_PREFIX: Regex = Regex::new(r"^(git|ssh)\.").unwrap(); // TODO This is RocketGit specific -> ranme and move to constants?
    }

    let clone_url_parts = git_clone_url::PartsRef::parse(any_clone_url).map_err(|err_str| {
        let scheme = protocol.scheme_str();
        Error::BadInputValue {
            key: protocol.to_clone_url_key(),
            msg: format!(
                "Evaluated resulting clone URL is empty -> something went very wrong; Unable to convert clone URL to {scheme} using regex '{err_str}'",
            ),
            input: any_clone_url.to_owned(),
        }
    })?;
    let hosting_type = environment
        .settings
        .hosting_type_from_host(clone_url_parts.host);

    let host = if matches!(hosting_type, HostingType::RocketGit) {
        let prefix = match protocol {
            TransferProtocol::Git => "git.",
            TransferProtocol::Https => "",
            TransferProtocol::Ssh => "ssh.",
        };
        Cow::Owned(format!(
            "{prefix}{}",
            R_HOST_PREFIX.replace(clone_url_parts.host, "")
        ))
    } else {
        Cow::Borrowed(clone_url_parts.host)
    };
    let user_opt = clone_url_parts.user;
    let user_at = if matches!(protocol, TransferProtocol::Ssh) {
        // use the default user for the given hosting-type
        Cow::Borrowed(hosting_type.def_ssh_user())
    } else if let Some(user) = user_opt {
        if user == "git" {
            Cow::Borrowed("git@")
        } else if user.is_empty() {
            // no user
            Cow::Borrowed("")
        } else {
            // same user
            Cow::Owned(format!("{user}@"))
        }
    } else {
        // no user
        Cow::Borrowed("")
    };

    let path_and_rest = clone_url_parts.path_and_rest;
    let scheme = protocol.scheme_str();
    Ok(Some(match protocol {
        TransferProtocol::Https | TransferProtocol::Git => {
            format!("{scheme}://{host}/{path_and_rest}",)
        }
        TransferProtocol::Ssh => {
            let host_path_sep = if host == constants::D_GIT_SOURCE_HUT {
                // This is **not** URL spec compatible,
                // but some/most hosters support this.
                ':'
            } else {
                // This is the (URL spec) compatible way
                '/'
            };
            format!(
                "{scheme}://{user}{host}{host_path_sep}{path_and_rest}",
                // "{scheme}://{host}/{path_and_rest}", // anonymized (without user)
                user = user_at.to_lowercase(),
                path_and_rest = path_and_rest,
            )
        }
    }))
}

/// Converts any kind of clone URL (wrapped in an `Option`) to an HTTP(S) or SSH one.
/// See [`clone_url_conversion`].
///
/// # Errors
///
/// If conversion failed, usually due to an invalid input URL.
///
/// If the user in the URL suggests a non-public access URL.
pub fn clone_url_conversion_option(
    any_clone_url: Option<&String>,
    environment: &Environment,
    protocol: TransferProtocol,
) -> Res {
    // TODO Can probably be removed by clever usage of map, ok, and*, or*, Into, From, ... something!
    Ok(match any_clone_url {
        Some(clone_url) => clone_url_conversion(clone_url, environment, protocol)?,
        None => None,
    })
}

/// Removes one a given `char` from both the first and last position
/// of the `input` string, if it is present there.
///
/// for example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use projvar::value_conversions::trim_char;
/// assert_eq!(
///     trim_char("/hoijui/kicad-text-injector/", '/'),
///     "hoijui/kicad-text-injector"
/// );
/// assert_eq!(
///     trim_char("hoijui/kicad-text-injector/", '/'),
///     "hoijui/kicad-text-injector"
/// );
/// assert_eq!(
///     trim_char("/hoijui/kicad-text-injector", '/'),
///     "hoijui/kicad-text-injector"
/// );
/// assert_eq!(
///     trim_char("hoijui/kicad-text-injector", '/'),
///     "hoijui/kicad-text-injector"
/// );
/// assert_eq!(
///     trim_char("*hoijui/kicad-text-injector/", '/'),
///     "*hoijui/kicad-text-injector"
/// );
/// assert_eq!(
///     trim_char("*hoijui/kicad-text-injector/", '*'),
///     "hoijui/kicad-text-injector/"
/// );
/// # Ok(())
/// # }
/// ```
#[must_use]
pub fn trim_char(input: &'_ str, char: char) -> &'_ str {
    let output = if input.starts_with(char) {
        &input[1..]
    } else {
        input
    };
    if output.ends_with(char) {
        &output[..output.len() - 1]
    } else {
        output
    }
}

/// Converts a common git repo web-host URL
/// into the URL of where to find hosted CI output
/// (commonly known as "pages" URL).
///
/// NOTE: This will likely only work for github.com and gitlab.com!
///
/// for example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use projvar::value_conversions::split_after_first_path_element;
/// # use projvar::tools::git_hosting_provs::PublicSite;
/// assert_eq!(
///     split_after_first_path_element("", "/hoijui/kicad-text-injector/", PublicSite::Unknown)?,
///     ("hoijui", "kicad-text-injector")
/// );
/// assert_eq!(
///     split_after_first_path_element("", "hoijui/kicad-text-injector/", PublicSite::Unknown)?,
///     ("hoijui", "kicad-text-injector")
/// );
/// assert_eq!(
///     split_after_first_path_element("", "/hoijui/kicad-text-injector", PublicSite::Unknown)?,
///     ("hoijui", "kicad-text-injector")
/// );
/// assert_eq!(
///     split_after_first_path_element("", "hoijui/kicad-text-injector", PublicSite::Unknown)?,
///     ("hoijui", "kicad-text-injector")
/// );
/// assert_eq!(
///     split_after_first_path_element("", "/hoijui/sub-path/kicad-text-injector/", PublicSite::Unknown)?,
///     ("hoijui", "sub-path/kicad-text-injector")
/// );
/// assert_eq!(
///     split_after_first_path_element("", "hoijui/sub-path/kicad-text-injector/", PublicSite::Unknown)?,
///     ("hoijui", "sub-path/kicad-text-injector")
/// );
/// assert_eq!(
///     split_after_first_path_element("", "/hoijui/sub-path/kicad-text-injector", PublicSite::Unknown)?,
///     ("hoijui", "sub-path/kicad-text-injector")
/// );
/// assert_eq!(
///     split_after_first_path_element("", "hoijui/sub-path/kicad-text-injector", PublicSite::Unknown)?,
///     ("hoijui", "sub-path/kicad-text-injector")
/// );
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Failed to split.
pub fn split_after_first_path_element<'t>(
    web_url: &str,
    path: &'t str,
    public_site: PublicSite,
) -> Result<(&'t str, &'t str), Error> {
    let path = trim_char(path, '/');
    path.split_once('/').ok_or_else(|| Error::BadInputValue {
        key: Key::BuildHostingUrl,
        msg: format!("Invalid web hosting URL for {public_site:?}"),
        input: web_url.to_owned(),
    })
}

macro_rules! build_hostify_url {
    ($url:ident, $web_url:ident, $public_site:ident, $suffix:ident) => {{
        let old_path = $url.path().to_owned();
        let (site_user, site_project) =
            split_after_first_path_element($web_url, &old_path, $public_site)?;
        $url.set_host(Some(&format!("{site_user}.{}", constants::$suffix)))
            .map_err(std_error::Error::from)?;
        $url.set_path(site_project);
        Some($url.to_string())
    }};
}

/// Converts a common git repo web-host URL
/// into the URL of where to find hosted CI output
/// (commonly known as "pages" URL).
///
/// NOTE: This will likely only work for github.com and gitlab.com!
///
/// for example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use projvar::value_conversions::web_url_to_build_hosting_url;
/// # use projvar::environment::Environment;
/// # let environment = Environment::stub();
/// assert_eq!(
///     web_url_to_build_hosting_url(&environment, "https://github.com/hoijui/kicad-text-injector/")?,
///     Some("https://hoijui.github.io/kicad-text-injector".to_owned())
/// );
/// assert_eq!(
///     web_url_to_build_hosting_url(&environment, "https://gitlab.com/hoijui/kicad-text-injector")?,
///     Some("https://hoijui.gitlab.io/kicad-text-injector".to_owned())
/// );
/// assert_eq!(
///     web_url_to_build_hosting_url(&environment, "https://gitlab.com/hoijui/sub-group/kicad-text-injector")?,
///     Some("https://hoijui.gitlab.io/sub-group/kicad-text-injector".to_owned())
/// );
/// assert_eq!(
///     web_url_to_build_hosting_url(&environment, "https://codeberg.org/hoijui/kicad-text-injector/")?,
///     Some("https://hoijui.codeberg.page/kicad-text-injector".to_owned())
/// );
/// assert_eq!(
///     web_url_to_build_hosting_url(&environment, "https://sourceforge.net/projects/xampp/")?,
///     Some("https://xampp.sourceforge.io".to_owned())
/// );
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Failed fetching/generating the Web URL.
///
/// Failed generating the "pages" URL,
/// likely because the remote is neither "github.com" nor "gitlab.com".
// <https://osegermany.gitlab.io/OHS-3105/>
// <https://hoijui.github.io/escher/>
pub fn web_url_to_build_hosting_url(environment: &Environment, web_url: &str) -> Res {
    web_url_match(
        environment,
        web_url,
        Key::RepoCommitPrefixUrl,
        &|mut url| {
            let public_site = PublicSite::from(url.host());
            Ok(match public_site {
                PublicSite::GitHubCom => {
                    build_hostify_url!(url, web_url, public_site, DS_GIT_HUB_IO_SUFIX)
                }
                PublicSite::GitLabCom => {
                    build_hostify_url!(url, web_url, public_site, DS_GIT_LAB_IO_SUFIX)
                }
                PublicSite::CodeBergOrg => {
                    build_hostify_url!(url, web_url, public_site, DS_CODE_BERG_PAGE)
                }
                PublicSite::SourceForgeNet => {
                    let url_path = PathBuf::from_str(url.path()).expect("Impossible");
                    let proj_name_opt = url_path.file_name().map(OsStr::to_string_lossy);
                    proj_name_opt.map(|proj_name| format!("https://{proj_name}.{}", constants::DS_SOURCE_FORGE_IO))
                }
                PublicSite::BitBucketOrg // has no pages hosting
                | PublicSite::SourceHut // has pages support (<https://srht.site/>), but only per-user, not per repo. One could try to emulate per repo pages there, but it would be cumbersome and is not standardized.
                | PublicSite::RepoOrCz // has no pages hosting
                | PublicSite::RocketGitCom // has no pages hosting
                | PublicSite::Unknown => None,
            })
        },
    )
}

/// Converts a common web hosting URL (HTTPS)
/// into a git remote URL (HTTPS or SSH).
///
/// for example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use projvar::tools::git::TransferProtocol;
/// # use projvar::value_conversions::web_url_to_clone_url;
/// # use projvar::environment::Environment;
/// # let environment = Environment::stub();
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://github.com/hoijui/kicad-text-injector/", TransferProtocol::Ssh)?,
///     Some("ssh://git@github.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://github.com/hoijui/kicad-text-injector", TransferProtocol::Https)?,
///     Some("https://github.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://gitlab.com/hoijui/kicad-text-injector", TransferProtocol::Ssh)?,
///     Some("ssh://git@gitlab.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://gitlab.com/hoijui/kicad-text-injector/", TransferProtocol::Https)?,
///     Some("https://gitlab.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://gitlab.com/hoijui/sub-group/kicad-text-injector", TransferProtocol::Ssh)?,
///     Some("ssh://git@gitlab.com/hoijui/sub-group/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://gitlab.com/hoijui/sub-group/kicad-text-injector/", TransferProtocol::Https)?,
///     Some("https://gitlab.com/hoijui/sub-group/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://bitbucket.org/hoijui/kicad-text-injector", TransferProtocol::Ssh)?,
///     Some("ssh://git@bitbucket.org/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://bitbucket.org/hoijui/kicad-text-injector/", TransferProtocol::Https)?,
///     Some("https://bitbucket.org/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://git.sr.ht/~sircmpwn/sr.ht-docs", TransferProtocol::Ssh)?,
///     Some("ssh://git@git.sr.ht:~sircmpwn/sr.ht-docs".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://git.sr.ht/~sircmpwn/sr.ht-docs", TransferProtocol::Https)?,
///     Some("https://git.sr.ht/~sircmpwn/sr.ht-docs".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://rocketgit.com/user/hoijui/rs-test", TransferProtocol::Ssh)?,
///     Some("ssh://rocketgit@ssh.rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://rocketgit.com/user/hoijui/rs-test", TransferProtocol::Https)?,
///     Some("https://rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://rocketgit.com/user/hoijui/rs-test", TransferProtocol::Git)?,
///     Some("git://git.rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://repo.or.cz/girocco.git", TransferProtocol::Ssh)?,
///     Some("ssh://repo.or.cz/girocco.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://repo.or.cz/girocco.git", TransferProtocol::Https)?,
///     Some("https://repo.or.cz/girocco.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://repo.or.cz/girocco.git", TransferProtocol::Git)?,
///     Some("git://repo.or.cz/girocco.git".to_owned())
/// );
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// If the conversion failed,
/// which usually happens if the `web_url` is not a github.com or gitlab.com.
pub fn web_url_to_clone_url(
    environment: &Environment,
    web_url: &str,
    protocol: TransferProtocol,
) -> Res {
    lazy_static! {
        static ref R_SLASH_AT_END: Regex = Regex::new(r"^(.+?)/?$").unwrap();
    }
    let key = protocol.to_clone_url_key();
    let http_clone_url = web_url_match(environment, web_url, key, &|mut url| {
        Ok(match environment.settings.hosting_type(&url) {
            HostingType::GitHub | HostingType::GitLab | HostingType::BitBucket => {
                let path = R_SLASH_AT_END.replace(url.path(), "$1.git").into_owned();
                url.set_path(&path);
                Some(url.to_string())
            }
            HostingType::SourceHut | HostingType::RocketGit | HostingType::Girocco => {
                Some(url.to_string())
            }
            _ => None, // TODO Implement the others!
        })
    })?;
    clone_url_conversion_option(http_clone_url.as_ref(), environment, protocol)
}

/// Converts a common git remote URL (HTTP(S) or SSH)
/// into a web-ready (HTTPS) URL of the project.
///
/// # Errors
///
/// If `any_clone_url` failed to parse as a URL.
///
/// for example:
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use projvar::value_conversions::clone_url_to_web_url;
/// # use projvar::environment::Environment;
/// # let environment = Environment::stub();
/// assert_eq!(
///     clone_url_to_web_url(&environment, "git@github.com:hoijui/kicad-text-injector.git")?,
///     Some("https://github.com/hoijui/kicad-text-injector".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "https://github.com/hoijui/kicad-text-injector.git")?,
///     Some("https://github.com/hoijui/kicad-text-injector".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "git@gitlab.com:hoijui/kicad-text-injector.git")?,
///     Some("https://gitlab.com/hoijui/kicad-text-injector".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "https://gitlab.com/hoijui/kicad-text-injector.git")?,
///     Some("https://gitlab.com/hoijui/kicad-text-injector".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "git@bitbucket.org:Aouatef/master_arbeit.git")?,
///     Some("https://bitbucket.org/Aouatef/master_arbeit".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "https://hoijui@bitbucket.org/Aouatef/master_arbeit.git")?,
///     Some("https://bitbucket.org/Aouatef/master_arbeit".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "https://git.sr.ht/~sircmpwn/sr.ht-docs")?,
///     Some("https://git.sr.ht/~sircmpwn/sr.ht-docs".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "git@git.sr.ht:~sircmpwn/sr.ht-docs")?,
///     Some("https://git.sr.ht/~sircmpwn/sr.ht-docs".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "https://rocketgit.com/user/hoijui/rs-test")?,
///     Some("https://rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "git://rocketgit@git.rocketgit.com/user/hoijui/rs-test")?,
///     Some("https://rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "ssh://rocketgit@ssh.rocketgit.com/user/hoijui/rs-test")?,
///     Some("https://rocketgit.com/user/hoijui/rs-test".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "https://repo.or.cz/girocco.git")?,
///     Some("https://repo.or.cz/girocco.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "git://repo.or.cz/girocco.git")?,
///     Some("https://repo.or.cz/girocco.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_to_web_url(&environment, "ssh://repo.or.cz/girocco.git")?,
///     Some("https://repo.or.cz/girocco.git".to_owned())
/// );
/// # Ok(())
/// # }
/// ```
pub fn clone_url_to_web_url(environment: &Environment, any_clone_url: &str) -> Res {
    lazy_static! {
        static ref R_DOT_GIT_SUFFIX: Regex = Regex::new(r"\.git$").unwrap();
    }

    let https_clone_url =
        clone_url_conversion(any_clone_url, environment, TransferProtocol::Https)?;
    match https_clone_url {
        Some(https_clone_url) => {
            match Url::parse(&https_clone_url) {
                Err(err) => Err(Error::BadInputValueErr {
                    key: Key::RepoWebUrl,
                    msg: "Not a valid URL".to_owned(),
                    input: https_clone_url.clone(),
                    source: Box::new(err),
                }),
                Ok(mut url) => {
                    Ok(match environment.settings.hosting_type(&url) {
                        HostingType::GitHub
                        | HostingType::GitLab
                        | HostingType::BitBucket
                        | HostingType::Gitea => {
                            let old_path = url.path().to_owned();
                            url.set_path(R_DOT_GIT_SUFFIX.replace(&old_path, "").as_ref());
                            url.set_username("").map_err(|_err| Error::BadInputValue {
                                key: Key::RepoWebUrl,
                                msg: "Failed to set username".to_owned(),
                                input: any_clone_url.to_owned(),
                            })?;
                            Some(url.to_string())
                        }
                        HostingType::Girocco | HostingType::SourceHut | HostingType::RocketGit => {
                            Some(https_clone_url)
                        } // Web-hosting and HTTP clone URL are exactly identical
                        _ => None, // TODO Implement the others!
                    })
                }
            }
        }
        None => Ok(None),
    }
}

/// Converts an ISO 8601 formatted date string
/// into the date frmat in our settings.
///
/// # Errors
///
/// If `in_date` is not a valid ISO 8601 date,
/// or the date format in our settings is invalid.
pub fn date_iso8601_to_our_format(environment: &Environment, in_date: &str) -> Res {
    let parsed = DateTime::parse_from_rfc3339(in_date)?;
    Ok(Some(
        parsed.format(&environment.settings.date_format).to_string(),
    ))
}
