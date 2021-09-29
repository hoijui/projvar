// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::lazy_static::lazy_static;
use enum_map::{Enum, EnumMap};
// use enumset::{EnumSet, EnumSetType};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    io::BufRead,
    iter::Iterator,
};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

use std::str::FromStr;

type BoxResult<T> = Result<T, Box<dyn Error>>;

// #[derive(Clone)]
// #[derive(Debug)]
#[derive(Default)]
pub struct Variable {
    pub key: &'static str,
    pub description: &'static str,
    pub default_required: bool,
    // pub alt_keys: Vec<&'static str>,
    pub alt_keys: &'static [&'static str],
}

impl Display for Variable {
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        write!(formatter, "{}", self.key)?;
        Ok(())
    }
}

lazy_static! {
    static ref D_VARIABLE: Variable = Variable::default();
}

impl<'a> Default for &'a Variable {
    fn default() -> &'a Variable {
        &D_VARIABLE
    }
}

// #[derive(Debug, EnumString, EnumIter, IntoStaticStr, PartialEq, Eq, Hash, Copy, Clone, Enum)]
// #[derive(Debug, EnumString, EnumIter, IntoStaticStr, Hash, Enum, EnumSetType)]
#[derive(Debug, EnumString, EnumIter, IntoStaticStr, Hash, Enum, PartialEq, Eq, Clone)]
pub enum Key {
    Version,
    License,
    RepoWebUrl,
    RepoFrozenWebUrl,
    RepoCloneUrl,
    RepoIssuesUrl,
    Name,
    VersionDate,
    BuildDate,
    BuildBranch,
    BuildTag,
    BuildIdent, // TODO This name is very bad, as it makes one think of BUILD_NUMBER; choose a different one!
    BuildOs,
    BuildOsFamily,
    BuildArch,
    BuildHostingUrl,
    BuildNumber,
    Ci,
}

/// Converts a `"CamelCase"` string into an `"UPPER_SNAKE_CASE"` one.
///
/// for example:
///
/// ```
/// //# fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use projvar::var::camel_to_upper_snake_case;
/// assert_eq!(
///     camel_to_upper_snake_case("someLowerCaseStartingTest"),
///     "SOME_LOWER_CASE_STARTING_TEST"
/// );
/// assert_eq!(
///     camel_to_upper_snake_case("SomeUpperCaseStartingTest"),
///     "SOME_UPPER_CASE_STARTING_TEST"
/// );
/// // NOTE From here on, we start seeing the limitation of this simple algorithm
/// assert_eq!(
///     camel_to_upper_snake_case("somethingWith123ANumber"),
///     "SOMETHING_WITH123_A_NUMBER"
/// );
/// assert_eq!(
///     camel_to_upper_snake_case("somethingWith123aNumber"),
///     "SOMETHING_WITH123A_NUMBER"
/// );
/// //# Ok(())
/// //# }
/// ```
#[must_use]
pub fn camel_to_upper_snake_case(id: &str) -> String {
    lazy_static! {
        static ref R_UPPER_SEL: Regex = Regex::new(r"(?P<after>[A-Z])").unwrap();
    }
    let res = R_UPPER_SEL.replace_all(id, "_$after").to_uppercase();
    res.strip_prefix('_').unwrap_or(&res).to_string()
}

impl Key {
    /// Tries to create a `Key` from a string identifier.
    /// This might be the exact name of the `Key` (like "Name"),
    /// or the associated variable key (like `"PROJECT_NAME"`).
    ///
    /// # Errors
    ///
    /// If the given identifier could not be mapped to any `Key` variant.
    pub fn from_name_or_var_key(id: &str) -> BoxResult<Key> {
        Ok(Self::from_str(id).or_else(|_| Self::from_str(&camel_to_upper_snake_case(id)))?)
    }
}

// pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
// where
//     P: AsRef<Path>,
// {
//     let file = File::open(filename)?;
//     Ok(io::BufReader::new(file).lines())
// }

/// Parses a file containing lines string with of the fomr "KEY=VALUE".
/// Empty lines and those starting wiht either "#" or "//" are ignored.
///
/// # Errors
///
/// If there is a problem with reading the file.
///
/// If any line has a bad form, missing key and/or value.
pub fn parse_vars_file_reader(mut reader: impl BufRead) -> BoxResult<HashMap<String, String>> {
    lazy_static! {
        // Ignore empty lines and those starting wiht '#' or "//"
        static ref R_IGNORE_LINE: Regex = Regex::new(r"^($|#|//)").unwrap();
    }
    let mut vars = HashMap::<String, String>::new();

    for line in repvar::tools::lines_iterator(&mut reader) {
        let line = line?;
        let line = line.trim();
        if !R_IGNORE_LINE.is_match(line) {
            let (key, value) = parse_key_value_str(line)?;
            vars.insert(key.to_owned(), value.to_owned());
        }
    }
    Ok(vars)
}

// pub fn parse_vars_file(var_file: &str) -> BoxResult<HashMap<String, String>> {
//     lazy_static! {
//         // Ignore empty lines or
//         static ref R_IGNORE_LINE: Regex = Regex::new(r"^($|#|//)").unwrap();
//     }
//     let mut vars = HashMap::<String, String>::new();
//     for line in read_lines(var_file)? {
//         let line = line?;
//         let line = line.trim();
//         if !R_IGNORE_LINE.is_match(line) {
//             let (key, value) = parse_key_value_str(line)?;
//             vars.insert(key.to_owned(), value.to_owned());
//         }
//     }
//     Ok(vars)
// }

/// Parses a string with the pattern "KEY=VALUE" into a (key, value) tuple.
///
/// # Errors
///
/// If the string has a bad form, missing key and/or value.
pub fn parse_key_value_str(key_value: &str) -> BoxResult<(&str, &str)> {
    let mut splitter = key_value.splitn(2, '=');
    let key = splitter
        .next()
        .ok_or("Failed to parse key; key-value pairs have to be of the form \"key=value\"")?;
    let value = splitter
        .next()
        .ok_or("Failed to parse value; key-value pairs have to be of the form \"key=value\"")?;
    Ok((key, value))
}

/// Checks if a string represents a valid key-value pair,
/// conforming to the pattern "KEY=VALUE".
///
/// # Errors
///
/// If the string does not represent a valid key-value pair.
pub fn is_key_value_str_valid(key_value: &str) -> Result<(), String> {
    parse_key_value_str(key_value)
        .map(|_kv| ())
        .map_err(|_err| String::from("Not a valid key=value pair"))
}

pub fn list_keys(alt_keys: bool) {
    if alt_keys {
        log::info!(
            "| {} | {} | {} | {} |",
            "D",
            "Key",
            "Alternative Keys",
            "Description"
        );
        log::info!("| - | --- | -------- | ------------ |");
    } else {
        log::info!("| {} | {} | {} |", "D", "Key", "Description");
        log::info!("| - | --- | ------------ |");
    }
    for key in Key::iter() {
        let var = get(key);
        let def = if var.default_required { "X" } else { " " };
        if alt_keys {
            log::info!(
                "| {} | {} | {} | {} |",
                def,
                var.key,
                var.alt_keys.join(" "),
                var.description
            );
        } else {
            log::info!("| {} | {} | {} |", def, var.key, var.description);
        }
    }
}

pub const KEY_VERSION: &str = "PROJECT_VERSION";
pub const KEY_LICENSE: &str = "PROJECT_LICENSE";
pub const KEY_REPO_WEB_URL: &str = "PROJECT_REPO_WEB_URL";
pub const KEY_REPO_FROZEN_WEB_URL: &str = "BUILD_REPO_FROZEN_WEB_URL";
pub const KEY_REPO_CLONE_URL: &str = "PROJECT_REPO_CLONE_URL";
pub const KEY_REPO_ISSUES_URL: &str = "PROJECT_REPO_ISSUES_URL";
pub const KEY_NAME: &str = "PROJECT_NAME";
pub const KEY_VERSION_DATE: &str = "PROJECT_VERSION_DATE";
pub const KEY_BUILD_DATE: &str = "BUILD_DATE";
pub const KEY_BUILD_BRANCH: &str = "BUILD_BRANCH";
pub const KEY_BUILD_TAG: &str = "BUILD_TAG";
pub const KEY_BUILD_IDENT: &str = "BUILD_IDENT"; // TODO This name is very bad, as it makes one think of BUILD_NUMBER; choose a different one!
pub const KEY_BUILD_OS: &str = "BUILD_OS";
pub const KEY_BUILD_OS_FAMILY: &str = "BUILD_OS_FAMILY";
pub const KEY_BUILD_ARCH: &str = "BUILD_ARCH";
pub const KEY_BUILD_HOSTING_URL: &str = "BUILD_HOSTING_URL";
pub const KEY_BUILD_NUMBER: &str = "BUILD_NUMBER";
pub const KEY_CI: &str = "CI";

// impl enum_map::Enum for Key {
//     type Array;

//     fn from_usize(value: usize) -> Self {
//         todo!()
//     }

//     fn into_usize(self) -> usize {
//         todo!()
//     }
// }

// impl Eq for Key {
//     fn assert_receiver_is_total_eq(&self) {}
// }

#[macro_export]
macro_rules! variable {
    ($key_str:ident, $description:expr, $alt_keys:expr) => {
        Variable {
            key: $key_str,
            description: $description,
            alt_keys: $alt_keys,
        }
    };
}

const VAR_VERSION: Variable = Variable {
    key: KEY_VERSION,
    description: "The project version.",
    default_required: true,
    alt_keys: &["VERSION", "CI_COMMIT_SHORT_SHA"],
};
const VAR_LICENSE: Variable = Variable {
    key: KEY_LICENSE,
    description: "Main License of the sources.",
    default_required: true,
    alt_keys: &["LICENSE"],
};
const VAR_REPO_WEB_URL: Variable = Variable {
    key: KEY_REPO_WEB_URL,
    description: "The Repo web UI URL.",
    default_required: true,
    alt_keys: &[
        "REPO_WEB_URL",
        "REPO",
        "CI_PROJECT_URL",
        "BITBUCKET_GIT_HTTP_ORIGIN",
    ],
};
const VAR_REPO_FROZEN_WEB_URL: Variable = Variable {
    key: KEY_REPO_FROZEN_WEB_URL,
    description: "The Repo web UI URL, pointing to the specific version of this build.",
    default_required: false,
    alt_keys: &["FROZEN_WEB_URL", "COMMIT_URL"],
};
const VAR_REPO_CLONE_URL: Variable = Variable {
    key: KEY_REPO_CLONE_URL,
    description: "The Repo clone URL.",
    default_required: true,
    alt_keys: &[
        "REPO_CLONE_URL",
        "CLONE_URL",
        "CI_REPOSITORY_URL",
        "BITBUCKET_GIT_SSH_ORIGIN",
    ],
};
const VAR_REPO_ISSUES_URL: Variable = Variable {
    key: KEY_REPO_ISSUES_URL,
    description: "The Repo issues URL.",
    default_required: true,
    alt_keys: &[], // TODO ... or maybe not, as we do not use this at all anymore
};
const VAR_NAME: Variable = Variable {
    key: KEY_NAME,
    description: "The name of the project.",
    default_required: true,
    alt_keys: &[
        "NAME",
        "CI_PROJECT_NAME",
        "APP_NAME",
        "BITBUCKET_PROJECT_KEY",
    ],
};
const VAR_VERSION_DATE: Variable = Variable {
    key: KEY_VERSION_DATE,
    description: "Date this version was committed to source control. ['%Y-%m-%d']",
    default_required: true,
    alt_keys: &[
        "VERSION_DATE",
        "DATE",
        "COMMIT_DATE",
        "PROJECT_COMMIT_DATE",
        "CI_COMMIT_TIMESTAMP",
    ],
};
const VAR_BUILD_DATE: Variable = Variable {
    key: KEY_BUILD_DATE,
    description: "Date of this build. ['%Y-%m-%d']",
    default_required: false,
    alt_keys: &[],
};
const VAR_BUILD_BRANCH: Variable = Variable {
    key: KEY_BUILD_BRANCH,
    description: "The development branch name.",
    default_required: false,
    alt_keys: &[
        "BRANCH",
        "GITHUB_REF",
        "CI_COMMIT_BRANCH",
        "BRANCH_NAME",
        "BITBUCKET_BRANCH",
        "TRAVIS_BRANCH",
    ],
};
const VAR_BUILD_TAG: Variable = Variable {
    key: KEY_BUILD_TAG,
    description: "The tag of a commit that kicked off the build. This value is only available on tags. Not available for builds against branches.",
    default_required: false,
    alt_keys: &[
        "TAG",
        "GITHUB_REF",
        "CI_COMMIT_TAG",
        "BITBUCKET_TAG",
        "TRAVIS_TAG",
    ],
};
const VAR_BUILD_IDENT: Variable = Variable {
    key: KEY_BUILD_IDENT,
    description:
        "Unique identifier of the state of the project that is being built (e.g. git commit SHA).",
    default_required: true,
    alt_keys: &[
        "GITHUB_SHA",
        "CI_COMMIT_SHA",
        "PULL_BASE_SHA",
        "BITBUCKET_COMMIT",
        "TRAVIS_COMMIT",
    ],
};
const VAR_BUILD_OS: Variable = Variable {
    key: KEY_BUILD_OS,
    description:
        "Operating system we are building on. (common values: 'linux', 'macos', 'windows')",
    default_required: false,
    alt_keys: &[
        "OS",
        "RUNNER_OS",
        "CI_RUNNER_EXECUTABLE_ARCH",
        "TRAVIS_OS_NAME",
    ],
};
const VAR_BUILD_OS_FAMILY: Variable = Variable {
    key: KEY_BUILD_OS_FAMILY,
    description:
        "Operating system family we are building on. (should be either 'unix' or 'windows')",
    default_required: false,
    alt_keys: &["OS_FAMILY", "FAMILY"],
};
const VAR_BUILD_ARCH: Variable = Variable {
    key: KEY_BUILD_ARCH,
    description:
        "Computer hardware architecture we are building on. (common values: 'x86', 'x86_64')",
    default_required: false,
    alt_keys: &["ARCH"],
};
const VAR_BUILD_HOSTING_URL: Variable = Variable {
    key: KEY_BUILD_HOSTING_URL,
    description: "Web URL under which the generated output will be available.",
    default_required: false,
    alt_keys: &["HOSTING_URL", "CI_PAGES_URL"],
};
const VAR_BUILD_NUMBER: Variable = Variable {
    key: KEY_BUILD_NUMBER,
    description: "The build number (1, 2, 3) starts at 1 for each repo and branch.",
    default_required: false,
    alt_keys: &[
        "NUMBER",
        "ID",
        "BUILD_ID",
        "BITBUCKET_BUILD_NUMBER",
        "TRAVIS_BUILD_NUMBER",
    ],
};
const VAR_CI: Variable = Variable {
    key: KEY_CI,
    description: "'true' if running on a CI/build-bot.",
    default_required: false,
    alt_keys: &[],
};

fn create(key: &Key) -> &'static Variable {
    match key {
        Key::Version => &VAR_VERSION,
        Key::License => &VAR_LICENSE,
        Key::RepoWebUrl => &VAR_REPO_WEB_URL,
        Key::RepoFrozenWebUrl => &VAR_REPO_FROZEN_WEB_URL,
        Key::RepoCloneUrl => &VAR_REPO_CLONE_URL,
        Key::RepoIssuesUrl => &VAR_REPO_ISSUES_URL,
        Key::Name => &VAR_NAME,
        Key::VersionDate => &VAR_VERSION_DATE,
        Key::BuildDate => &VAR_BUILD_DATE,
        Key::BuildBranch => &VAR_BUILD_BRANCH,
        Key::BuildTag => &VAR_BUILD_TAG,
        Key::BuildIdent => &VAR_BUILD_IDENT,
        Key::BuildOs => &VAR_BUILD_OS,
        Key::BuildOsFamily => &VAR_BUILD_OS_FAMILY,
        Key::BuildArch => &VAR_BUILD_ARCH,
        Key::BuildHostingUrl => &VAR_BUILD_HOSTING_URL,
        Key::BuildNumber => &VAR_BUILD_NUMBER,
        Key::Ci => &VAR_CI,
    }
}

fn create_vars() -> EnumMap<Key, &'static Variable> {
    Key::iter()
        .map(|key| {
            let var = create(&key);
            (key, var)
        })
        .into_iter()
        .collect()
}

// fn create_default_keys() -> EnumSet<Key> {
//     let mut def_keys = EnumSet::<Key>::empty();
fn create_default_keys() -> HashSet<Key> {
    let mut def_keys = HashSet::<Key>::new();
    for (key, variable) in VARS.iter() {
        if variable.default_required {
            def_keys.insert(key);
        }
    }
    def_keys
}

lazy_static! {
    static ref VARS: EnumMap<Key, &'static Variable> = create_vars();
}
lazy_static! {
    // static ref DEFAULT_KEYS: EnumSet<Key> = create_default_keys();
    static ref DEFAULT_KEYS: HashSet<Key> = create_default_keys();
}

/// Returns a reference to the variable settings associated with the given key.
///
/// # Panics
///
/// Never, as a match in the code ensures that all enum variants of `Key`
/// have a value assigned to them.
#[must_use]
pub fn get(key: Key) -> &'static Variable {
    // VARS.get(&key).unwrap()
    VARS[key]
}

#[must_use]
// pub fn default_keys() -> EnumSet<Key> {
pub fn default_keys() -> &'static HashSet<Key> {
    &DEFAULT_KEYS
}
