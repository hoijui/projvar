// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::lazy_static::lazy_static;
use enum_map::Enum;
// use enumset::{EnumSet, EnumSetType};
use regex::Regex;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    io::BufRead,
    iter::Iterator,
};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, IntoStaticStr};

use std::str::FromStr;

use crate::environment::Environment;

type BoxResult<T> = Result<T, Box<dyn Error>>;

pub type Confidence = u8;
pub const C_HIGH: Confidence = 75;
pub const C_MIDDLE: Confidence = 50;
pub const C_LOW: Confidence = 25;

// #[derive(Clone)]
// #[derive(Debug)]
#[derive(Default)]
pub struct Variable {
    key: &'static str,
    pub description: &'static str,
    pub default_required: bool,
    // pub alt_keys: &'static [&'static str], // This data was once present for all variables; see the commit that commented out this line with `git blame`
}

impl Variable {
    #[must_use]
    pub fn key(&self, environment: &Environment) -> Cow<str> {
        match &environment.settings.key_prefix {
            Some(prefix) => Cow::Owned(prefix.clone() + self.key),
            None => Cow::Borrowed(self.key),
        }
    }
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

#[remain::sorted]
// #[derive(Debug, EnumString, EnumIter, IntoStaticStr, PartialEq, Eq, Hash, Copy, Clone, Enum)]
// #[derive(Debug, EnumString, EnumIter, IntoStaticStr, Hash, Enum, EnumSetType)]
#[derive(Debug, EnumString, EnumIter, IntoStaticStr, Hash, Enum, PartialEq, Eq, Clone, Copy)]
pub enum Key {
    BuildArch,
    BuildBranch,
    BuildDate,
    BuildHostingUrl,
    // BuildIdent, // TODO This name is very bad, as it makes one think of BUILD_NUMBER; choose a different one! Maybe refunction it as well(?) -> `HumanVersion` (vs a machine-readable one like from git describe, which goes to `Version`), for example "Ubuntu 10.04 - UbsiDubsi"
    BuildNumber,
    BuildOs,
    BuildOsFamily,
    BuildTag,
    Ci,
    License,
    Licenses,
    Name,
    NameMachineReadable,
    RepoCloneUrl,
    RepoCloneUrlSsh,
    RepoCommitPrefixUrl,
    RepoIssuesUrl,
    RepoRawVersionedPrefixUrl,
    RepoVersionedDirPrefixUrl,
    RepoVersionedFilePrefixUrl,
    RepoWebUrl,
    Version,
    VersionDate,
}

/// Converts an `"UPPER_SNAKE_CASE"` string into an `"CamelCase"` one.
///
/// for example:
///
/// ```
/// //# fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # use projvar::var::upper_snake_to_camel_case;
/// assert_eq!(
///     upper_snake_to_camel_case("SOME_UPPER_CASE_STARTING_TEST"),
///     "SomeUpperCaseStartingTest"
/// );
/// // NOTE From here on, we start seeing the limitation of this simple algorithm
/// assert_eq!(
///     upper_snake_to_camel_case("SOMETHING_WITH123A_NUMBER"),
///     "SomethingWith123aNumber"
/// );
/// //# Ok(())
/// //# }
/// ```
#[must_use]
pub fn upper_snake_to_camel_case(id: &str) -> String {
    lazy_static! {
        // static ref R_PREF: Regex = Regex::new(r"^_").unwrap();
        // static ref R_SUFF: Regex = Regex::new(r"_$").unwrap();
        static ref R_FIRST: Regex = Regex::new(r"^(.)").unwrap();
        static ref R_UPPER_SEL: Regex = Regex::new(r"(.)_(.)").unwrap();
    }
    let id = id.to_lowercase();
    // let id= R_PREF.replace(&id, "");
    // let id= R_SUFF.replace(&id, "");
    let id = R_FIRST.replace_all(&id, |captures: &regex::Captures| captures[1].to_uppercase());
    let id = R_UPPER_SEL.replace_all(&id, |captures: &regex::Captures| {
        captures[1].to_owned() + &captures[2].to_uppercase()
    });
    id.strip_prefix('_')
        .unwrap_or(&id)
        .strip_suffix('_')
        .unwrap_or(&id)
        .to_string()
}

impl Key {
    /// Tries to create a `Key` from a string identifier.
    /// This might be the exact name of the `Key` (like "Name"),
    /// or the associated variable key (like `"PROJECT_NAME"`).
    ///
    /// # Errors
    ///
    /// If the given identifier could not be mapped to any `Key` variant.
    pub fn from_name_or_var_key(key_prefix: &Regex, id: &str) -> BoxResult<Key> {
        Ok(Self::from_str(id).or_else(|_| {
            Self::from_str(&upper_snake_to_camel_case(
                key_prefix.replace(id, "").as_ref(),
            ))
        })?)
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

pub fn list_keys(environment: &Environment) {
    log::info!("| {} | {} | {} |", "D", "Key", "Description");
    log::info!("| - | --- | ------------ |");
    for key in Key::iter() {
        let var = get(key);
        let def = if var.default_required { "[x]" } else { "[ ]" };
        log::info!(
            "| {} | {} | {} |",
            def,
            var.key(environment),
            var.description
        );
    }
}

pub const KEY_BUILD_ARCH: &str = "BUILD_ARCH";
pub const KEY_BUILD_BRANCH: &str = "BUILD_BRANCH";
pub const KEY_BUILD_DATE: &str = "BUILD_DATE";
pub const KEY_BUILD_HOSTING_URL: &str = "BUILD_HOSTING_URL";
pub const KEY_BUILD_NUMBER: &str = "BUILD_NUMBER";
pub const KEY_BUILD_OS: &str = "BUILD_OS";
pub const KEY_BUILD_OS_FAMILY: &str = "BUILD_OS_FAMILY";
pub const KEY_BUILD_TAG: &str = "BUILD_TAG";
pub const KEY_CI: &str = "CI";
pub const KEY_LICENSE: &str = "LICENSE";
pub const KEY_LICENSES: &str = "LICENSES";
pub const KEY_NAME: &str = "NAME";
pub const KEY_NAME_MACHINE_READABLE: &str = "NAME_MACHINE_READABLE";
pub const KEY_REPO_CLONE_URL: &str = "REPO_CLONE_URL";
pub const KEY_REPO_CLONE_URL_SSH: &str = "REPO_CLONE_URL_SSH";
pub const KEY_REPO_COMMIT_PREFIX_URL: &str = "REPO_COMMIT_PREFIX_URL";
pub const KEY_REPO_ISSUES_URL: &str = "REPO_ISSUES_URL";
pub const KEY_REPO_RAW_VERSIONED_PREFIX_URL: &str = "REPO_RAW_VERSIONED_PREFIX_URL";
pub const KEY_REPO_VERSIONED_DIR_PREFIX_URL: &str = "REPO_VERSIONED_DIR_PREFIX_URL";
pub const KEY_REPO_VERSIONED_FILE_PREFIX_URL: &str = "REPO_VERSIONED_FILE_PREFIX_URL";
pub const KEY_REPO_WEB_URL: &str = "REPO_WEB_URL";
pub const KEY_VERSION: &str = "VERSION";
pub const KEY_VERSION_DATE: &str = "VERSION_DATE";

const VAR_BUILD_ARCH: Variable = Variable {
    key: KEY_BUILD_ARCH,
    description:
        "The computer hardware architecture we are building on. (common values: 'x86', 'x86_64')",
    default_required: false,
};
const VAR_BUILD_BRANCH: Variable = Variable {
    key: KEY_BUILD_BRANCH,
    description: r#"The development branch name, for example: "master", "develop""#,
    default_required: false,
};
const VAR_BUILD_DATE: Variable = Variable {
    key: KEY_BUILD_DATE,
    description: r#"Date of this build, for example: "2021-12-31 23:59:59" (see --date-format)"#,
    default_required: false,
};
const VAR_BUILD_HOSTING_URL: Variable = Variable {
    key: KEY_BUILD_HOSTING_URL,
    description: "Web URL under which the generated output will be available, for example: https://osegermany.gitlab.io/OHS-3105",
    default_required: false,
};
const VAR_BUILD_NUMBER: Variable = Variable {
    key: KEY_BUILD_NUMBER,
    description: "The build number (1, 2, 3) starts at 1 for each repo and branch.",
    default_required: false,
};
const VAR_BUILD_OS: Variable = Variable {
    key: KEY_BUILD_OS,
    description:
        "The operating system we are building on. (common values: 'linux', 'macos', 'windows')",
    default_required: false,
};
const VAR_BUILD_OS_FAMILY: Variable = Variable {
    key: KEY_BUILD_OS_FAMILY,
    description:
        "The operating system family we are building on. (should be either 'unix' or 'windows')",
    default_required: false,
};
const VAR_BUILD_TAG: Variable = Variable {
    key: KEY_BUILD_TAG,
    description: "The tag of a commit that kicked off the build. This value is only available on tags. Not available for builds against branches.",
    default_required: false,
};
const VAR_CI: Variable = Variable {
    key: KEY_CI,
    description: "'true' if running on a CI/build-bot; unset otherwise.",
    default_required: false,
};
const VAR_LICENSE: Variable = Variable {
    key: KEY_LICENSE,
    description: r#"The main License identifier of the sources, prefferably from the SPDX specs, for example: "AGPL-3.0-or-later", "CC-BY-SA-4.0""#,
    default_required: true,
};
const VAR_LICENSES: Variable = Variable {
    key: KEY_LICENSES,
    description: r#"The identifiers of all the licenses of this project, prefferably from the SPDX specs, comma separated, for example: "AGPL-3.0-or-later, CC0-1.0, Unlicense""#,
    default_required: true,
};
const VAR_NAME: Variable = Variable {
    key: KEY_NAME,
    description: "The human focused name of the project.",
    default_required: true,
};
const VAR_NAME_MACHINE_READABLE: Variable = Variable {
    key: KEY_NAME_MACHINE_READABLE,
    description: "The machine readable name of the project.",
    default_required: true,
};
const VAR_REPO_CLONE_URL: Variable = Variable {
    key: KEY_REPO_CLONE_URL,
    description: "The repo clone URL, HTTP(S) version. This is commonly used for anonymous fetch-only access.",
    default_required: true,
};
const VAR_REPO_CLONE_URL_SSH: Variable = Variable {
    key: KEY_REPO_CLONE_URL_SSH,
    description: "The repo clone URL, SSH version. This is commonly used for authenticated, fetch and push access.",
    default_required: false,
};
const VAR_REPO_COMMIT_PREFIX_URL: Variable = Variable {
    key: KEY_REPO_COMMIT_PREFIX_URL,
    description: "The repo commit prefix URL. Add commit SHA. The part in []: [https://github.com/hoijui/nim-ci/commit]/23f84b91]",
    default_required: true,
};
const VAR_REPO_ISSUES_URL: Variable = Variable {
    key: KEY_REPO_ISSUES_URL,
    description: "The repo issues URL, for example: https://gitlab.com/openflexure/openflexure-microscope/issues",
    default_required: true,
};
const VAR_REPO_RAW_VERSIONED_PREFIX_URL: Variable = Variable {
    key: KEY_REPO_RAW_VERSIONED_PREFIX_URL,
    description: "The repo raw prefix URL. Add version (tag, branch, SHA) and file path. The part in []: [https://raw.githubusercontent.com/hoijui/nim-ci]/master/.github/workflows/docker.yml]",
    default_required: true,
};
const VAR_REPO_VERSIONED_DIR_PREFIX_URL: Variable = Variable {
    key: KEY_REPO_VERSIONED_DIR_PREFIX_URL,
    description: "The repo directory prefix URL. Add version (tag, branch, SHA) and directory path. The part in []: [https://github.com/hoijui/nim-ci]/master/.github/workflows/docker.yml]",
    default_required: true,
};
const VAR_REPO_VERSIONED_FILE_PREFIX_URL: Variable = Variable {
    key: KEY_REPO_VERSIONED_FILE_PREFIX_URL,
    description: "The repo file prefix URL. Add version (tag, branch, SHA) and file path. The part in []: [https://github.com/hoijui/nim-ci]/master/.github/workflows/docker.yml]",
    default_required: true,
};
const VAR_REPO_WEB_URL: Variable = Variable {
    key: KEY_REPO_WEB_URL,
    description: "The repo web UI URL, for example: https://gitlab.com/OSEGermany/OHS-3105",
    default_required: true,
};
const VAR_VERSION: Variable = Variable {
    key: KEY_VERSION,
    description: r#"The project version, for example: "1.10.3", "0.2.0-1-ga5387ac-dirty""#,
    default_required: true,
};
const VAR_VERSION_DATE: Variable = Variable {
    key: KEY_VERSION_DATE,
    description: r#"Date this version was committed to source control, for example: "2021-12-31 23:59:59" (see --date-format)"#,
    default_required: true,
};

/// Returns a reference to the variable settings associated with the given key.
#[must_use]
#[remain::check]
pub fn get(key: Key) -> &'static Variable {
    #[remain::sorted]
    match key {
        Key::BuildArch => &VAR_BUILD_ARCH,
        Key::BuildBranch => &VAR_BUILD_BRANCH,
        Key::BuildDate => &VAR_BUILD_DATE,
        Key::BuildHostingUrl => &VAR_BUILD_HOSTING_URL,
        Key::BuildNumber => &VAR_BUILD_NUMBER,
        Key::BuildOs => &VAR_BUILD_OS,
        Key::BuildOsFamily => &VAR_BUILD_OS_FAMILY,
        Key::BuildTag => &VAR_BUILD_TAG,
        Key::Ci => &VAR_CI,
        Key::License => &VAR_LICENSE,
        Key::Licenses => &VAR_LICENSES,
        Key::Name => &VAR_NAME,
        Key::NameMachineReadable => &VAR_NAME_MACHINE_READABLE,
        Key::RepoCloneUrl => &VAR_REPO_CLONE_URL,
        Key::RepoCloneUrlSsh => &VAR_REPO_CLONE_URL_SSH,
        Key::RepoCommitPrefixUrl => &VAR_REPO_COMMIT_PREFIX_URL,
        Key::RepoIssuesUrl => &VAR_REPO_ISSUES_URL,
        Key::RepoRawVersionedPrefixUrl => &VAR_REPO_RAW_VERSIONED_PREFIX_URL,
        Key::RepoVersionedDirPrefixUrl => &VAR_REPO_VERSIONED_DIR_PREFIX_URL,
        Key::RepoVersionedFilePrefixUrl => &VAR_REPO_VERSIONED_FILE_PREFIX_URL,
        Key::RepoWebUrl => &VAR_REPO_WEB_URL,
        Key::Version => &VAR_VERSION,
        Key::VersionDate => &VAR_VERSION_DATE,
    }
}

fn create_default_keys() -> HashSet<Key> {
    let mut def_keys = HashSet::<Key>::new();
    for key in Key::iter() {
        let variable = get(key);
        if variable.default_required {
            def_keys.insert(key);
        }
    }
    def_keys
}

lazy_static! {
    // static ref DEFAULT_KEYS: EnumSet<Key> = create_default_keys();
    static ref DEFAULT_KEYS: HashSet<Key> = create_default_keys();
}

#[must_use]
// pub fn default_keys() -> EnumSet<Key> {
pub fn default_keys() -> &'static HashSet<Key> {
    &DEFAULT_KEYS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_name_or_var_key() -> std::result::Result<(), Box<dyn Error>> {
        let r_prefix_none = Regex::new("^").unwrap();
        let r_prefix_project = Regex::new("^PROJECT_").unwrap();

        assert_eq!(
            Key::from_name_or_var_key(&r_prefix_none, "VERSION")?,
            Key::Version
        );
        assert_eq!(
            Key::from_name_or_var_key(&r_prefix_project, "VERSION")?,
            Key::Version
        );
        assert_eq!(
            Key::from_name_or_var_key(&r_prefix_project, "PROJECT_VERSION")?,
            Key::Version
        );
        assert_eq!(
            Key::from_name_or_var_key(&r_prefix_none, "REPO_WEB_URL")?,
            Key::RepoWebUrl
        );
        assert_eq!(
            Key::from_name_or_var_key(&r_prefix_project, "REPO_WEB_URL")?,
            Key::RepoWebUrl
        );
        assert_eq!(
            Key::from_name_or_var_key(&r_prefix_project, "PROJECT_REPO_WEB_URL")?,
            Key::RepoWebUrl
        );

        assert!(Key::from_name_or_var_key(&r_prefix_none, "PROJECT_VERSION").is_err());
        assert!(Key::from_name_or_var_key(&r_prefix_none, "PROJECT_REPO_WEB_URL").is_err());

        Ok(())
    }
}
