// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// use std::process::Command;
use crate::environment::Environment;
use crate::sinks::VarSink;
use crate::sources::VarSource;
// use crate::settings;
// use crate::storage;
// use chrono::DateTime;
// use chrono::Local;
use strum::IntoEnumIterator;
// use chrono::NaiveDateTime;
// use chrono::Utc;
// use clap::lazy_static::lazy_static;
// use git2::{self, Repository};
// use regex::Regex;
// use crate::tools::git;
// use crate::props::version;
use crate::var::{self, Key, Variable};
// use std::collections::HashMap;
// use std::convert::TryFrom;
// use std::env;
use std::error::Error;
// use std::fmt;
// use std::path::Path;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/* fn alt_version(environment: &mut Environment) -> BoxResult<Option<String>> { */
/*     Ok(if let Some(repo) = environment.repo() { */
/*         let sc_version = repo.version()?; */

/*         if git::is_git_dirty_version(&sc_version) { */
/*             log::warn!( */
/*                 "Dirty project version ('{}')! (you have uncommitted changes in your project)", */
/*                 sc_version */
/*             ); */
/*         } */
/*         Some(sc_version) */
/*     } else { */
/*         None */
/*     }) */
/* } */

/* fn alt_name(environment: &mut Environment) -> BoxResult<Option<String>> { */
/*     Ok( */
/*         if let Some(val) = environment.vars.get("GITHUB_REPOSITORY") { */
/*             // `val` should usually be "USER_NAME/PROJECT_NAME", */
/*             // on GitLab it can also have more then these two parts, */
/*             // but there should always be at least two parts, */
/*             // therefore the call to `skip()`. */
/*             Some( */
/*                 val.split('/') */
/*                     .skip(1) */
/*                     .last() */
/*                     .ok_or_else(|| { */
/*                         format!( */
/*                             "Failed to parse project-name from env(GITHUB_REPOSITORY) ('{}')", */
/*                             val */
/*                         ) */
/*                     })? */
/*                     .to_owned(), */
/*             ) */
/*         } else { */
/*             let repo_path = environment */
/*                 .settings */
/*                 .repo_path */
/*                 // .map_or(Ok(None), |r| r.map(Some)); */
/*                 .ok_or("No repo path provided")?; */
/*             let dir_name = dir_name(repo_path)?; */
/*             match dir_name.as_str() { */
/*                 // Filter out some common names that are not likely to be the projects name */
/*                 "src" | "target" | "build" | "master" | "main" => None, */
/*                 _other => Some(dir_name), */
/*             } */
/*         }, */
/*     ) */
/* } */

/* /// This uses an alternative method to fetch certain specific variable keys values. */
/* /// Altenraitve meaning here: */
/* /// Not directly fetching it from any environment variable. */
/* fn fetch_alt(environment: &mut Environment, variable: &var::Variable) -> BoxResult<Option<String>> { */
/*     Ok(match variable.key { */
/*         var::KEY_VERSION => alt_version(environment)?, */
/*         var::KEY_NAME => alt_name(environment)?, */
/*         var::KEY_REPO_WEB_URL => { */
/*             match ( */
/*                 environment.vars.get("GITHUB_SERVER_URL")?, */
/*                 environment.vars.get("GITHUB_REPOSITORY")?, */
/*             ) { */
/*                 (Some(server), Some(repo)) => { */
/*                     // "${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/" */
/*                     // usually: */
/*                     // GITHUB_SERVER_URL="https://github.com/" */
/*                     // GITHUB_REPOSITORY="user/project" */
/*                     Some(format!("{}/{}/", server, repo)) */
/*                 } */
/*                 (_, _) => { */
/*                     if let Some(repo) = environment.repo() { */
/*                         Some(repo.remote_web_url()?) */
/*                     } else { */
/*                         None */
/*                     } */
/*                 } */
/*             } */
/*         } */
/*         var::KEY_REPO_FROZEN_WEB_URL => { */
/*             let base_repo_web_url = fetch_var_or_key_any_key_or_alt(environment, &Key::RepoWebUrl)?; */
/*             let version = fetch_var_or_key_any_key_or_alt(environment, &Key::Version)?; */
/*             let commit_sha = fetch_var_or_key_any_key_or_alt(environment, &Key::BuildIdent)?; */

/*             if let (Some(base_repo_web_url), Some(version_or_sha)) = */
/*                 (base_repo_web_url, version.or(commit_sha)) */
/*             { */
/*                 Some(format!("{}/tree/{}", base_repo_web_url, version_or_sha)) */
/*             } else { */
/*                 None */
/*             } */
/*             // https://gitlab.com/OSEGermany/okhmanifest */
/*             // https://gitlab.com/OSEGermany/okhmanifest/-/commit/9e1df32c42a85253af95ea2dc9311128bd8f775a */
/*             // https://gitlab.com/OSEGermany/okhmanifest/-/tree/oldCombinedDeprecated */
/*             // https://gitlab.com/OSEGermany/OHS-3105/-/tree/din-3105-0.10.0 */
/*             // https://gitlab.com/OSEGermany/OHS-3105/-/tree/din-spec-3105-0.10.0-179-g60c46fc */

/*             // https://github.com/hoijui/repvar */
/*             // https://github.com/hoijui/repvar/tree/4939bd538643bfb445167ea72b825e605f120318 */
/*         } */
/*         var::KEY_BUILD_DATE => { */
/*             let now = Local::now(); */
/*             Some(now.format(&environment.settings.date_format).to_string()) */
/*         } */
/*         // var::KEY_CI => { */
/*         //     // If env("CI") is not defined/set, */
/*         //     // we want this variable to remain undefined/unset */
/*         // }, */
/*         var::KEY_BUILD_BRANCH => { */
/*             if let Some(repo) = environment.repo() { */
/*                 repo.branch().unwrap_or_else(|err| { */
/*                     log::warn!("Failed fetching git branch - {}", err); */
/*                     None */
/*                 }) */
/*             } else { */
/*                 None */
/*             } */
/*         } */
/*         var::KEY_BUILD_TAG => { */
/*             if let Some(repo) = environment.repo() { */
/*                 repo.tag()? */
/*             } else { */
/*                 None */
/*             } */
/*         } */
/*         var::KEY_REPO_CLONE_URL => { */
/*             if let Some(repo) = environment.repo() { */
/*                 Some(repo.remote_clone_url()?) */
/*                 // repo.remote_clone_url().or_else(|err| { */
/*                 //     log::warn!("Failed fetching git repo clone URL - {}", err); */
/*                 //     None */
/*                 // }) */
/*             } else { */
/*                 None */
/*             } */
/*         } */
/*         var::KEY_VERSION_DATE => { */
/*             let date_format = &environment.settings.date_format; */
/*             if let Some(repo) = environment.repo() { */
/*                 Some(repo.commit_date(date_format)?) */
/*             } else { */
/*                 None */
/*             } */
/*         } */
/*         var::KEY_BUILD_HOSTING_URL => { */
/*             if let Some(repo) = environment.repo() { */
/*                 Some(repo.build_hosting_url()?) */
/*             } else { */
/*                 None */
/*             } */
/*         } */
/*         var::KEY_BUILD_OS => { */
/*             // See here for possible values: */
/*             // <https://doc.rust-lang.org/std/env/consts/constant.OS.html> */
/*             // Most common values: "linux", "macos", "windows" */
/*             Some(env::consts::OS.to_owned()) */
/*         } */
/*         var::KEY_BUILD_OS_FAMILY => { */
/*             // Possible values: "unix", "windows" */
/*             // <https://doc.rust-lang.org/std/env/consts/constant.FAMILY.html> */
/*             // format!("{}", env::consts::FAMILY) */
/*             Some(env::consts::FAMILY.to_owned()) */
/*         } */
/*         var::KEY_BUILD_ARCH => { */
/*             // See here for possible values: */
/*             // <https://doc.rust-lang.org/std/env/consts/constant.ARCH.html> */
/*             // Most common values: "x86", "x86_64" */
/*             Some(env::consts::ARCH.to_owned()) */
/*         } */

/*         &_ => { */
/*             // Err(Box::new(Err("")))? */
/*             // return Err(NoAltMethodError { */
/*             //     key: variable.key.to_owned(), */
/*             // } */
/*             // .into()); */
/*             if let settings::Verbosity::Info = environment.settings.verbosity { */
/*                 log::warn!( */
/*                     "No alternative method to fetch value for key '{}'", */
/*                     variable.key */
/*                 ); */
/*             } */
/*             None */
/*         } */
/*     }) */
/* } */

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
    sources: Vec<Box<dyn VarSource>>,
    sinks: Vec<Box<dyn VarSink>>,
) -> BoxResult<()> {
    for source in sources {
        if source.is_usable(environment) {
            log::trace!("Trying to fetch from source {} ...", source);
            for key in Key::iter() {
                // log::trace!("\tTrying to fetch {:?} ...", source);
                let value = source.retrieve(environment, key.clone())?;
                if let Some(value) = value {
                    log::trace!("\tFetched {:?}='{}'", key, value);
                    environment.output.insert(key, value);
                }
            }
        }
    }

    // TODO Add validators here

    log::trace!("Evaluated variables ...");
    let values: Vec<(Key, &'static Variable, String)> = {
        environment
            .output
            .iter()
            .map(|key_value| {
                let key = key_value.0.clone();
                let variable = var::get(key_value.0.clone());
                let value = key_value.1.clone();
                log::trace!("\t{:?}:{}='{}'", key, variable.key, &value);
                (key, variable, value)
            })
            .collect()
    };

    for ref sink in sinks {
        log::trace!("Checking if sink {} is usable ...", sink);
        if sink.is_usable(environment) {
            log::trace!("Storing to sink {} ...", sink);
            sink.store(environment, &values)?;
        }
    }

    log::trace!("Done.");

    Ok(())
}
