// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// use std::process::Command;
use crate::environment::Environment;
use crate::settings;
// use crate::storage;
// use chrono::DateTime;
use chrono::Local;
// use chrono::NaiveDateTime;
// use chrono::Utc;
// use clap::lazy_static::lazy_static;
// use git2::{self, Repository};
// use regex::Regex;
use crate::git;
// use crate::props::version;
use crate::var;
// use std::collections::HashMap;
// use std::convert::TryFrom;
use std::env;
use std::error::Error;
// use std::fmt;
use std::path::Path;

type BoxResult<T> = Result<T, Box<dyn Error>>;

// @click.argument("additional_replacements", type=replace_vars.KEY_VALLUE_PAIR, nargs=-1)
// @click.option('--src-file-path', '-p',
//         type=click.Path(dir_okay=False, file_okay=True),
//         envvar='PROJECT_SRC_FILE_PATH',
//         default=None,
//         help='The path to the source file, relative to the repo root. '
//             + 'This is only used in variable substitution; '
//             + 'no reading from that path will be attempted. (default: SRC)')
// @click.option('--repo-path', '-r',
//         type=click.Path(dir_okay=True, file_okay=False),
//         envvar='PROJECT_REPO_PATH',
//         default='.',
//         help='The path to the local git repo')
// @click.option('--repo-url', '-u',
//         type=click.STRING,
//         envvar='PROJECT_REPO_URL',
//         default=None,
//         help='Public project repo URL')
// @click.option('-n', '--name',
//         type=click.STRING,
//         envvar='PROJECT_NAME',
//         default=None,
//         help='Project name (prefferably without spaces)')
// @click.option('--vers',
//         type=click.STRING,
//         envvar='PROJECT_VERSION',
//         default=None,
//         help='Project version (prefferably without spaces)')
// @click.option('-d', '--version-date',
//         type=click.STRING,
//         envvar='PROJECT_VERSION_DATE',
//         default=None,
//         help='Date at which this version of the project was committed/released')
// @click.option('--build-date',
//         type=click.STRING,
//         envvar='PROJECT_BUILD_DATE',
//         default=None,
//         help=('Date at which the currently being-made build of '
//             + 'the project is made. This should basically always be left on the '
//             + 'default, which is the current date.'))

//     '''
//     Using a KiCad PCB file as input,
//     replaces variables of the type `${VAR_NAME}` in text-fields with actual values,
//     writing the result to an other KiCad PCB file.

//     Key-value pairs to be used for the replacement are collected from 3 sources:

//     * read from common environment variables like `PROJECT_REPO_PATH` and `PROJECT_REPO_URL`

//     * specified through command-line switches like `--repo-url "https://github.com/user/repo/"`

//     * directly specified through `ADDITIONAL_REPLACEMENTS`, for example `"PROJECT_BATCH_ID=john-1"`

//     SRC - The source KiCad PCB file (this will be used as input,
//     and potentially for the replacement variable `${PROJECT_SRC_FILE_PATH}`).

//     DST - The destination KiCad PCB file (this will be used for the generated output).

//     ADDITIONAL_REPLACEMENTS - Each one of these is a ful key-value pair,
//     using '=' as the delimiter, for example `"PROJECT_BATCH_ID=john-1"`.
//     '''

// fn insert_if_missing<K: std::cmp::Eq + std::hash::Hash, V>(
//     map: &mut HashMap<K, V>,
//     key: K,
//     val: V,
// ) {
//     map.entry(key).or_insert(val);
// }

/// Returns the name of the given path (same as `basename` on UNIX systems)
fn dir_name(path: &Path) -> BoxResult<String> {
    Ok(path
        .canonicalize()?
        // .parent()
        // .ok_or_else(|| git2::Error::from_str("Unable to get containing directory's name"))?
        .file_name()
        .ok_or_else(|| git2::Error::from_str("File ends in .."))?
        .to_str()
        .ok_or_else(|| git2::Error::from_str("File name is not UTF-8 compatible"))?
        .to_owned())
}

// #[derive(Debug, Clone)]
// enum VarErrorType {
//     Get,
//     Set,
// }

// #[derive(Debug, Clone)]
// pub struct NoAltMethodError {
//     key: String,
//     // kind: VarErrorType,
//     // source: Option<env::VarError>,
// }

// impl Error for NoAltMethodError {
//     //     fn source(&self) -> Option<&(dyn Error + 'static)> {
//     //         self.source.map_or_else(|| None, |val| Some(val))
//     //     }
// }

// impl fmt::Display for NoAltMethodError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "No alternative mthod to fetch value for key '{}'",
//             self.key
//         )
//     }
// }

fn alt_version(environment: &mut Environment) -> BoxResult<Option<String>> {
    Ok(if let Some(repo) = environment.repo() {
        let sc_version = repo.version()?;

        if git::is_git_dirty_version(&sc_version) {
            log::warn!(
                "Dirty project version ('{}')! (you have uncommitted changes in your project)",
                sc_version
            );
        }
        Some(sc_version)
    } else {
        None
    })
}

fn alt_name(environment: &mut Environment) -> BoxResult<Option<String>> {
    Ok(
        if let Some(val) = environment.vars.get("GITHUB_REPOSITORY")? {
            // `val` should usually be "USER_NAME/PROJECT_NAME",
            // on GitLab it can also have more then these two parts,
            // but there should always be at least two parts,
            // therefore the call to `skip()`.
            Some(
                val.split('/')
                    .skip(1)
                    .last()
                    .ok_or_else(|| {
                        format!(
                            "Failed to parse project-name from env(GITHUB_REPOSITORY) ('{}')",
                            val
                        )
                    })?
                    .to_owned(),
            )
        } else {
            let repo_path = environment
                .settings
                .repo_path
                // .map_or(Ok(None), |r| r.map(Some));
                .ok_or("No repo path provided")?;
            let dir_name = dir_name(repo_path)?;
            match dir_name.as_str() {
                // Filter out some common names that are not likely to be the projects name
                "src" | "target" | "build" | "master" | "main" => None,
                _other => Some(dir_name),
            }
        },
    )
}

fn fetch_key_or_alt(environment: &mut Environment, key: &str) -> BoxResult<Option<String>> {
    Ok(environment.vars.get(key).unwrap_or(fetch_alt(
        environment,
        var::VARS
            .get(key)
            .ok_or_else(|| format!("Key '{}' is not part of the main core variables", key))?,
    )?))
}

fn fetch_any_key_or_alt(
    environment: &mut Environment,
    var: &var::Variable,
) -> BoxResult<Option<String>> {
    // Ok(var.fetch_any_var(&*environment.vars)?.or_else(|| fetch_alt(environment, var)?))
    let mut value = var.fetch_any_var(&*environment.vars)?;
    if value.is_none() {
        value = fetch_alt(environment, var)?;
    }
    Ok(value)
}

fn fetch_var_or_key_any_key_or_alt(
    environment: &mut Environment,
    key: &str,
) -> BoxResult<Option<String>> {
    match var::VARS.get(key) {
        Some(var) => fetch_any_key_or_alt(environment, var),
        None => fetch_key_or_alt(environment, key),
    }
}

/// This uses an alternative method to fetch certain specific variable keys values.
/// Altenraitve meaning here:
/// Not directly fetching it from any environment variable.
fn fetch_alt(environment: &mut Environment, variable: &var::Variable) -> BoxResult<Option<String>> {
    Ok(match variable.key {
        var::KEY_VERSION => alt_version(environment)?,
        var::KEY_NAME => alt_name(environment)?,
        var::KEY_REPO_WEB_URL => {
            match (
                environment.vars.get("GITHUB_SERVER_URL")?,
                environment.vars.get("GITHUB_REPOSITORY")?,
            ) {
                (Some(server), Some(repo)) => {
                    // "${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/"
                    // usually:
                    // GITHUB_SERVER_URL="https://github.com/"
                    // GITHUB_REPOSITORY="user/project"
                    Some(format!("{}/{}/", server, repo))
                }
                (_, _) => {
                    if let Some(repo) = environment.repo() {
                        Some(repo.remote_web_url()?)
                    } else {
                        None
                    }
                }
            }
        }
        var::KEY_REPO_FROZEN_WEB_URL => {
            let base_repo_web_url =
                fetch_var_or_key_any_key_or_alt(environment, var::KEY_REPO_WEB_URL)?;
            let version = fetch_var_or_key_any_key_or_alt(environment, var::KEY_VERSION)?;
            let commit_sha = fetch_var_or_key_any_key_or_alt(environment, var::KEY_BUILD_IDENT)?;

            if let (Some(base_repo_web_url), Some(version_or_sha)) =
                (base_repo_web_url, version.or(commit_sha))
            {
                Some(format!("{}/tree/{}", base_repo_web_url, version_or_sha))
            } else {
                None
            }
            // https://gitlab.com/OSEGermany/okhmanifest
            // https://gitlab.com/OSEGermany/okhmanifest/-/commit/9e1df32c42a85253af95ea2dc9311128bd8f775a
            // https://gitlab.com/OSEGermany/okhmanifest/-/tree/oldCombinedDeprecated
            // https://gitlab.com/OSEGermany/OHS-3105/-/tree/din-3105-0.10.0
            // https://gitlab.com/OSEGermany/OHS-3105/-/tree/din-spec-3105-0.10.0-179-g60c46fc

            // https://github.com/hoijui/repvar
            // https://github.com/hoijui/repvar/tree/4939bd538643bfb445167ea72b825e605f120318
        }
        var::KEY_BUILD_DATE => {
            let now = Local::now();
            Some(now.format(&environment.settings.date_format).to_string())
        }
        // var::KEY_CI => {
        //     // If env("CI") is not defined/set,
        //     // we want this variable to remain undefined/unset
        // },
        var::KEY_BUILD_BRANCH => {
            if let Some(repo) = environment.repo() {
                repo.branch().unwrap_or_else(|err| {
                    log::warn!("Failed fetching git branch - {}", err);
                    None
                })
            } else {
                None
            }
        }
        var::KEY_BUILD_TAG => {
            if let Some(repo) = environment.repo() {
                repo.tag()?
            } else {
                None
            }
        }
        var::KEY_REPO_CLONE_URL => {
            if let Some(repo) = environment.repo() {
                Some(repo.remote_clone_url()?)
                // repo.remote_clone_url().or_else(|err| {
                //     log::warn!("Failed fetching git repo clone URL - {}", err);
                //     None
                // })
            } else {
                None
            }
        }
        var::KEY_VERSION_DATE => {
            let date_format = &environment.settings.date_format;
            if let Some(repo) = environment.repo() {
                Some(repo.commit_date(date_format)?)
            } else {
                None
            }
        }
        var::KEY_BUILD_HOSTING_URL => {
            if let Some(repo) = environment.repo() {
                Some(repo.build_hosting_url()?)
            } else {
                None
            }
        }
        var::KEY_BUILD_OS => {
            // See here for possible values:
            // <https://doc.rust-lang.org/std/env/consts/constant.OS.html>
            // Most common values: "linux", "macos", "windows"
            Some(env::consts::OS.to_owned())
        }
        var::KEY_BUILD_OS_FAMILY => {
            // Possible values: "unix", "windows"
            // <https://doc.rust-lang.org/std/env/consts/constant.FAMILY.html>
            // format!("{}", env::consts::FAMILY)
            Some(env::consts::FAMILY.to_owned())
        }
        var::KEY_BUILD_ARCH => {
            // See here for possible values:
            // <https://doc.rust-lang.org/std/env/consts/constant.ARCH.html>
            // Most common values: "x86", "x86_64"
            Some(env::consts::ARCH.to_owned())
        }

        &_ => {
            // Err(Box::new(Err("")))?
            // return Err(NoAltMethodError {
            //     key: variable.key.to_owned(),
            // }
            // .into());
            if let settings::Verbosity::Info = environment.settings.verbosity {
                log::warn!(
                    "No alternative method to fetch value for key '{}'",
                    variable.key
                );
            }
            None
        }
    })
}

/// The main function of this crate,
/// gathering data as good as it can,
/// and making sure it is stored in the appropriate environment variables.
///
/// # Errors
///
/// Reading from the environment fails.
///
/// Any of the alternative methods to come up with a value
/// for a specific key fails.
///
/// Writing to the environment fails.
pub fn prepare_project_vars(
    environment: &mut Environment,
    // vars: &mut HashMap<String, String>,
    // repo_path: Option<&str>,
    // repo_web_url: Option<&str>,
    // name: Option<&str>,
    // version: Option<&str>,
    // version_date: Option<&str>,
    // build_date: Option<&str>,
    // date_format: Option<&str>,
) -> BoxResult<()> {
    // ) {
    environment.vars.init();

    for var in &*var::VARS {
        log::debug!("Looking for {} ...", &var.1);
        let val = fetch_any_key_or_alt(environment, var.1)?;
        match val {
            Some(val) => {
                if let settings::Verbosity::Info = environment.settings.verbosity {
                    log::info!("Value found for {}: '{}'", &var.1, val);
                }
                match environment.settings.to_set {
                    settings::VarsToSet::All => var.1.set_all(&mut *environment.vars, &val),
                    settings::VarsToSet::Primary => var.1.set_main(&mut *environment.vars, &val),
                }?;
            }
            None => {
                log::warn!("No value found for {}", &var.1);
            }
        }
    }

    environment.vars.finalize();

    Ok(())
}
