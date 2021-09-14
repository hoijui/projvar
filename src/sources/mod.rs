// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod bitbucket_ci;
pub mod git;
pub mod github_ci;
pub mod gitlab_ci;
pub mod jenkins_ci;
pub mod travis_ci;

use std::error::Error;
use std::fmt;

use crate::environment::Environment;
use crate::var::Key;

type BoxResult<T> = Result<T, Box<dyn Error>>;

pub trait VarSource: fmt::Display {
    /// Indicates whether this source of variables is usable.
    /// It might not be usable if the underlying data-source (e.g. a file) does not exist,
    /// or is not reachable (e.g. a web URL).
    fn is_usable(&self, environment: &mut Environment) -> bool;

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

/// Tries to construct the frozen web URL
/// from other properties of a variable source.
///
/// # Errors
///
/// If an attempt to try fetching any required property returned an error.
// https://gitlab.com/OSEGermany/okhmanifest
// https://gitlab.com/OSEGermany/okhmanifest/-/commit/9e1df32c42a85253af95ea2dc9311128bd8f775a
// https://gitlab.com/OSEGermany/okhmanifest/-/tree/oldCombinedDeprecated
// https://gitlab.com/OSEGermany/OHS-3105/-/tree/din-3105-0.10.0
// https://gitlab.com/OSEGermany/OHS-3105/-/tree/din-spec-3105-0.10.0-179-g60c46fc
// https://github.com/hoijui/repvar
// https://github.com/hoijui/repvar/tree/4939bd538643bfb445167ea72b825e605f120318
pub fn try_construct_frozen<S: VarSource>(
    var_source: &S,
    environment: &mut Environment,
) -> BoxResult<Option<String>> {
    let base_repo_web_url = var_source.retrieve(environment, Key::RepoWebUrl)?;
    let version = var_source.retrieve(environment, Key::Version)?;
    let commit_sha = var_source.retrieve(environment, Key::BuildIdent)?;

    Ok(
        if let (Some(base_repo_web_url), Some(version_or_sha)) =
            (base_repo_web_url, version.or(commit_sha))
        {
            Some(format!("{}/tree/{}", base_repo_web_url, version_or_sha))
        } else {
            None
        },
    )
}
