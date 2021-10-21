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

use thiserror::Error;

use clap::lazy_static::lazy_static;

use crate::environment::Environment;
use crate::var::Key;
use crate::{std_error, value_conversions};

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

/// This enumerates all possible errors returned by this module.
#[derive(Error, Debug)]
pub enum Error {
    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    ConversionError(#[from] value_conversions::Error),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// Represents all other cases of `std_error::Error`.
    #[error(transparent)]
    Std(#[from] std_error::Error),

    /// Represents all other cases of `std::error::Error`.
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error>),
}

type RetrieveRes = Result<Option<String>, Error>;

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
}

pub fn var(environment: &Environment, key: &str) -> Option<String> {
    environment
        .vars
        .get(key)
        .map(std::borrow::ToOwned::to_owned)
}
