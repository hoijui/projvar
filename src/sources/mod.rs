// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod bitbucket_ci;
pub mod deriver;
pub mod env;
pub mod fs;
pub mod git;
pub mod github_ci;
pub mod gitlab_ci;
pub mod jenkins_ci;
pub mod selector;
pub mod travis_ci;

use std::path::Path;

use cli_utils::BoxResult;
use thiserror::Error;

use lazy_static::lazy_static;

use crate::environment::Environment;
use crate::var::{Confidence, Key, C_HIGH};
use crate::{cleanup, std_error, tools, validator, value_conversions};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Hierarchy {
    Low,
    Middle,
    High,
    Higher,
    EvenHigher,
    Top,
}

lazy_static! {
    static ref NO_PROPS: Vec::<String> = Vec::<String>::new();
}

/// This enumerates all possible errors returned by this module.
#[derive(Error, Debug)]
pub enum Error {
    #[error("The value '{low_level_value}' - fetched from the underlying source - was bad: {msg}")]
    BadLowLevelValue {
        msg: String,
        low_level_value: String,
    },

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    ConversionError(#[from] value_conversions::Error),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    Git(#[from] tools::git::Error),

    /// Represents all other cases of `std_error::Error`.
    #[error(transparent)]
    Std(#[from] std_error::Error),

    /// Represents all other cases of `std::error::Error`.
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

pub type ConfVal = (Confidence, String);
pub type RetrieveRes = BoxResult<Option<ConfVal>>;

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
    fn retrieve(&self, environment: &mut Environment, key: Key) -> RetrieveRes;

    /// Uses an already found build-tag as the version field,
    /// if available.
    ///
    /// # Errors
    ///
    /// See [`Self::retrieve`].
    fn version_from_build_tag(&self, environment: &mut Environment, key: Key) -> RetrieveRes {
        assert!(matches!(key, Key::Version));
        Ok(self
            .retrieve(environment, Key::BuildTag)?
            .map(|conf_val| cleanup::conf_version(environment, conf_val))
            .filter(|conf_val| {
                if let Ok(validity) = validator::get(key)(environment, &conf_val.1) {
                    validity.is_good()
                } else {
                    false
                }
            }))
    }
}

#[must_use]
pub fn var(
    environment: &Environment,
    key: &str,
    confidence: Confidence,
) -> Option<(Confidence, String)> {
    environment
        .vars
        .get(key)
        .map(|val| (confidence, val.clone()))
}

fn ref_ok_or_err<'t>(refr: &str, part: Option<&'t str>) -> Result<&'t str, Error> {
    part.ok_or_else(|| Error::BadLowLevelValue {
        msg: "Invalid git reference, should be 'refs/<TYPE>/<NAME>'".to_owned(),
        low_level_value: refr.to_owned(),
    })
}

fn ref_extract_name_if_type_matches(refr: &str, required_ref_type: &str) -> RetrieveRes {
    let mut parts = refr.split('/');
    let extracted_ref_type = ref_ok_or_err(refr, parts.nth(1))?;
    Ok(if extracted_ref_type == required_ref_type {
        // it *is* a branch
        let branch_name = ref_ok_or_err(refr, parts.next())?;
        Some((C_HIGH, branch_name.to_owned()))
    } else {
        None
    })
}

/// Given a git reference, returns the branch name,
/// if `refr` reffers to a branch; None otherwise.
/// `refr` references should look like:
/// * "refs/tags/v1.2.3"
/// * "refs/heads/master"
/// * "refs/pull/:prNumber/merge"
///
/// # Errors
///
/// If the given ref is ill-formatted, meaning it does not split
/// into at least 3 parts with the '/' separator)
pub fn ref_extract_branch(refr: &str) -> RetrieveRes {
    ref_extract_name_if_type_matches(refr, "heads")
}

/// Given a git reference, returns the tag name,
/// if it reffers to a tag; None otherwise.
/// `refr` references should look like:
/// * "refs/tags/v1.2.3"
/// * "refs/heads/master"
/// * "refs/pull/:prNumber/merge"
///
/// # Errors
///
/// If the given ref is ill-formatted, meaning it does not split
/// into at least 3 parts with the '/' separator)
pub fn ref_extract_tag(refr: &str) -> RetrieveRes {
    ref_extract_name_if_type_matches(refr, "tags")
}

fn is_git_repo_root(repo_path: Option<&Path>) -> bool {
    tools::git::Repo::try_from(repo_path).is_ok()
}

#[must_use]
pub fn default_list(repo_path: &Path) -> Vec<Box<dyn VarSource>> {
    let mut sources: Vec<Box<dyn VarSource>> = vec![];
    if is_git_repo_root(Some(repo_path)) {
        sources.push(Box::new(git::VarSource {}));
    }
    sources.push(Box::new(fs::VarSource {}));
    sources.push(Box::new(bitbucket_ci::VarSource {}));
    sources.push(Box::new(github_ci::VarSource {}));
    sources.push(Box::new(gitlab_ci::VarSource {}));
    sources.push(Box::new(jenkins_ci::VarSource {}));
    sources.push(Box::new(travis_ci::VarSource {}));
    sources.push(Box::new(env::VarSource {}));
    sources.push(Box::new(selector::VarSource {}));
    sources.push(Box::new(deriver::VarSource {}));
    // NOTE We add the deriver a second time,
    //      so it may derive from values created in the first run.
    sources.push(Box::new(deriver::VarSource {}));
    if log::log_enabled!(log::Level::Trace) {
        for source in &sources {
            log::trace!("Registered source {}.", source.display());
        }
    }
    sources
}
