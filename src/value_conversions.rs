// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::tools::git_hosting_provs::{HostingType, PublicSite};
use chrono::DateTime;
use thiserror::Error;

use clap::lazy_static::lazy_static;
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
        source: Box<dyn std::error::Error>,
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
    Other(#[from] Box<dyn std::error::Error>),
    // /// Represents all other errors, especially those not fitting any of the above,
    // /// and which do not derive from `std::error::Error`.
    // #[error("No info about the errror is available")]
    // Empty {
    //     key: Key,
    //     msg: String,
    //     input: String,
    // },
}

#[derive(Clone, Copy)]
pub enum Protocol {
    /// https://gitlab.com/hoijui/kicad-text-injector.git
    Https,
    /// git@gitlab.com/hoijui/kicad-text-injector.git
    Ssh,
    // /// ssh://gitlab.com/hoijui/kicad-text-injector.git
    // SshUrl,
}

macro_rules! let_named_cap {
    ($caps:ident,$name:ident) => {
        let $name = if let Some(rmatch) = $caps.name(stringify!($name)) {
            rmatch.as_str()
        } else {
            ""
        };
    };
}
//pub(crate) use let_named_cap;

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
/// # use projvar::value_conversions::{clone_url_conversion, Protocol};
/// # use projvar::environment::Environment;
/// # let environment = Environment::stub();
/// assert_eq!(
///     clone_url_conversion("git@github.com:hoijui/kicad-text-injector.git", Protocol::Https)?,
///     Some("https://github.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://github.com/hoijui/kicad-text-injector.git", Protocol::Https)?,
///     Some("https://github.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@gitlab.com:hoijui/kicad-text-injector.git", Protocol::Https)?,
///     Some("https://gitlab.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://gitlab.com/hoijui/kicad-text-injector.git", Protocol::Https)?,
///     Some("https://gitlab.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@bitbucket.org:Aouatef/master_arbeit.git", Protocol::Https)?,
///     Some("https://bitbucket.org/Aouatef/master_arbeit.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://hoijui@bitbucket.org/Aouatef/master_arbeit.git", Protocol::Https)?,
///     Some("https://bitbucket.org/Aouatef/master_arbeit.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@github.com:hoijui/kicad-text-injector.git", Protocol::Ssh)?,
///     Some("ssh://git@github.com:hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://github.com/hoijui/kicad-text-injector.git", Protocol::Ssh)?,
///     Some("ssh://git@github.com:hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@gitlab.com:hoijui/kicad-text-injector.git", Protocol::Ssh)?,
///     Some("ssh://git@gitlab.com:hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://gitlab.com/hoijui/kicad-text-injector.git", Protocol::Ssh)?,
///     Some("ssh://git@gitlab.com:hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("git@bitbucket.org:Aouatef/master_arbeit.git", Protocol::Ssh)?,
///     Some("ssh://git@bitbucket.org:Aouatef/master_arbeit.git".to_owned())
/// );
/// assert_eq!(
///     clone_url_conversion("https://hoijui@bitbucket.org/Aouatef/master_arbeit.git", Protocol::Ssh)?,
///     Some("ssh://git@bitbucket.org:Aouatef/master_arbeit.git".to_owned())
/// );
/// # Ok(())
/// # }
/// ```
pub fn clone_url_conversion(any_clone_url: &str, protocol: Protocol) -> Res {
    lazy_static! {
        // This matches all these 3 types of clone URLs:
        // * git@github.com:hoijui/rust-project-scripts.git
        // * ssh://github.com/hoijui/rust-project-scripts.git
        // * https://github.com/hoijui/rust-project-scripts.git
        static ref R_CLONE_URL: Regex = Regex::new(r"^((?P<protocol>[0-9a-zA-Z._-]+)://)?((?P<user>[0-9a-zA-Z._-]+)@)?(?P<host>[0-9a-zA-Z._-]+)([/:](?P<path_and_rest>.+)?)?$").unwrap();
    }
    let to_protocol_str = match protocol {
        Protocol::Https => "https",
        Protocol::Ssh => "ssh",
    };
    let clone_url_res = R_CLONE_URL.replace(any_clone_url, |caps: &regex::Captures| {
        // let_named_cap!(caps, protocol);
        // let_named_cap!(caps, host);
        // let_named_cap!(caps, user);
        // let_named_cap!(caps, path_and_rest);
        let host = caps.name("host").map(|m| m.as_str());
        // let user_at = if user.is_empty() || user == "git" {
        //     // use the default anonymous git user (NOTE This might get us in trouble in some scenarios)
        //     // user.to_owned() + "@"
        //     "git@"
        // } else {
        //     // anonymize the URL
        //     // String::new()
        //     ""
        // };
        let user_at = "git@";

        if let Some(host) = host {
            let path_and_rest = caps
                .name("path_and_rest")
                .map(|m| /*"/".to_owned() +*/ m.as_str());
            match protocol {
                Protocol::Https => format!(
                    "{protocol}://{host}/{path_and_rest}",
                    protocol = to_protocol_str,
                    // user = user.to_lowercase(),
                    host = host,
                    path_and_rest = path_and_rest.unwrap_or_default(),
                ),
                Protocol::Ssh => format!(
                    //"{user}{host}:{path_and_rest}", // TODO or this? ...
                    "{protocol}://{user}{host}:{path_and_rest}", // TODO This is the (URL spec compatible) alternative
                    // "{protocol}://{host}:{path_and_rest}", // TODO This is the (URL spec compatible) alternative, anonymised (without user)
                    protocol = to_protocol_str,
                    user = user_at.to_lowercase(),
                    host = host,
                    path_and_rest = path_and_rest.unwrap_or_default(),
                ),
            }
        } else {
            any_clone_url.to_owned()
        }
    });
    if clone_url_res.is_empty() {
        Err(Error::BadInputValue {
            key: match protocol {
                Protocol::Https => Key::RepoCloneUrl,
                Protocol::Ssh => Key::RepoCloneUrlSsh,
            },
            msg: format!(
                "Evaluated resulting clone URL is empty -> something went very wrong; Unable to convert clone URL to {} using regex '{}'",
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
pub fn clone_url_conversion_option(any_clone_url: Option<&String>, protocol: Protocol) -> Res {
    // TODO Can probably be removed by clever usage of map, ok, and*, or*, Into, From, ... something!
    Ok(match any_clone_url {
        Some(clone_url) => clone_url_conversion(clone_url, protocol)?,
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
        msg: format!("Invalid web hosting URL for {:?}", public_site),
        input: web_url.to_owned(),
    })
}

macro_rules! build_hostify_url {
    ($url:ident, $web_url:ident, $public_site:ident, $suffix:ident) => {{
        let old_path = $url.path().to_owned();
        let (site_user, site_project) =
            split_after_first_path_element($web_url, &old_path, $public_site)?;
        $url.set_host(Some(&format!("{}.{}", site_user, constants::$suffix)))
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
                PublicSite::BitBucketOrg // BB does not have pages hosting
                | _ => None, // TODO Implement the others!
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
/// # use projvar::value_conversions::{web_url_to_clone_url, Protocol};
/// # use projvar::environment::Environment;
/// # let environment = Environment::stub();
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://github.com/hoijui/kicad-text-injector/", Protocol::Ssh)?,
///     Some("ssh://git@github.com:hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://github.com/hoijui/kicad-text-injector", Protocol::Https)?,
///     Some("https://github.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://gitlab.com/hoijui/kicad-text-injector", Protocol::Ssh)?,
///     Some("ssh://git@gitlab.com:hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://gitlab.com/hoijui/kicad-text-injector/", Protocol::Https)?,
///     Some("https://gitlab.com/hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://gitlab.com/hoijui/sub-group/kicad-text-injector", Protocol::Ssh)?,
///     Some("ssh://git@gitlab.com:hoijui/sub-group/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://gitlab.com/hoijui/sub-group/kicad-text-injector/", Protocol::Https)?,
///     Some("https://gitlab.com/hoijui/sub-group/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://bitbucket.org/hoijui/kicad-text-injector", Protocol::Ssh)?,
///     Some("ssh://git@bitbucket.org:hoijui/kicad-text-injector.git".to_owned())
/// );
/// assert_eq!(
///     web_url_to_clone_url(&environment, "https://bitbucket.org/hoijui/kicad-text-injector/", Protocol::Https)?,
///     Some("https://bitbucket.org/hoijui/kicad-text-injector.git".to_owned())
/// );
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// If the conversion failed,
/// which usually happens if the `web_url` is not a github.com or gitlab.com.
pub fn web_url_to_clone_url(environment: &Environment, web_url: &str, protocol: Protocol) -> Res {
    lazy_static! {
        static ref R_SLASH_AT_END: Regex = Regex::new(r"^(.+?)/?$").unwrap();
    }
    let key = match protocol {
        Protocol::Https => Key::RepoCloneUrl,
        Protocol::Ssh => Key::RepoCloneUrlSsh,
    };
    let http_clone_url = web_url_match(environment, web_url, key, &|mut url| {
        Ok(match environment.settings.hosting_type(&url) {
            HostingType::GitHub | HostingType::GitLab | HostingType::BitBucket => {
                let path = R_SLASH_AT_END.replace(url.path(), "$1.git").into_owned();
                url.set_path(&path);
                Some(url.to_string())
            }
            _ => None, // TODO Implement the others!
        })
    })?;
    Ok(match protocol {
        Protocol::Https => http_clone_url,
        Protocol::Ssh => clone_url_conversion_option(http_clone_url.as_ref(), protocol)?,
    })
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
/// # Ok(())
/// # }
/// ```
pub fn clone_url_to_web_url(environment: &Environment, any_clone_url: &str) -> Res {
    lazy_static! {
        static ref R_DOT_GIT_SUFFIX: Regex = Regex::new(r"\.git$").unwrap();
    }

    let https_clone_url = clone_url_conversion(any_clone_url, Protocol::Https)?;
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
                        HostingType::GitHub | HostingType::GitLab | HostingType::BitBucket => {
                            let old_path = url.path().to_owned();
                            url.set_path(R_DOT_GIT_SUFFIX.replace(&old_path, "").as_ref());
                            url.set_username("").map_err(|_err| Error::BadInputValue {
                                key: Key::RepoWebUrl,
                                msg: "Failed to set username".to_owned(),
                                input: any_clone_url.to_owned(),
                            })?;
                            Some(url.to_string())
                        }
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
