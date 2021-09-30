// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use chrono::Local;

use crate::environment::Environment;
use crate::var::Key;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::{env, fmt};

pub struct VarSource;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Returns the name of the given path (same as `basename` on UNIX systems)
fn dir_name(path: &Path) -> BoxResult<String> {
    Ok(path
        .canonicalize()?
        .file_name()
        .ok_or_else(|| git2::Error::from_str("File ends in \"..\""))?
        .to_str()
        .ok_or_else(|| git2::Error::from_str("File name is not UTF-8 compatible"))?
        .to_owned())
}

/// Get the title of a Markdown file.
///
/// Reads the first line of a Markdown file, strips any hashes and
/// leading/trailing whitespace, and returns the title.
fn title_string<R>(mut rdr: R) -> BoxResult<String>
where
    R: BufRead,
{
    let mut first_line = String::new();

    rdr.read_line(&mut first_line)?;

    // Where do the leading hashes stop?
    let last_hash = first_line
        .char_indices()
        .find(|&(_, c)| c != '#')
        .map_or(0, |(idx, _)| idx);

    // Trim the leading hashes and any whitespace
    Ok(first_line[last_hash..].trim().into())
}

/// Read the first line of the file into `title`.
fn file_title(path: &Path) -> BoxResult<Option<String>> {
    Ok(if path.exists() && path.is_file() {
        let file = File::open(path)?;
        let buffer = BufReader::new(file);
        Some(title_string(buffer)?)
    } else {
        None
    })
}

fn version(environment: &mut Environment) -> BoxResult<Option<String>> {
    Ok(match &environment.settings.repo_path {
        Some(repo_path) => {
            let version_file = repo_path.join("VERSION");
            file_title(&version_file)?
        }
        _ => None,
    })
}

fn name(environment: &mut Environment) -> BoxResult<Option<String>> {
    let repo_path = environment
        .settings
        .repo_path
        .as_ref()
        .ok_or("No repo path provided")?;
    let dir_name = dir_name(repo_path)?;
    Ok(match dir_name.to_lowercase().as_str() {
        // Filter out some common directory names that are not likely to be the projects name
        "src" | "target" | "build" | "master" | "main" | "develop" | "git" | "repo" | "repos"
        | "scm" | "trunk" => None,
        _other => Some(dir_name),
    })
}

fn build_date(environment: &mut Environment) -> String {
    let now = Local::now();
    now.format(&environment.settings.date_format).to_string()
}

fn build_os(_environment: &mut Environment) -> String {
    // See here for possible values:
    // <https://doc.rust-lang.org/std/env/consts/constant.OS.html>
    // Most common values: "linux", "macos", "windows"
    env::consts::OS.to_owned() // TODO Maybe move to a new source "env.rs"?
}

fn build_os_family(_environment: &mut Environment) -> String {
    // Possible values: "unix", "windows"
    // <https://doc.rust-lang.org/std/env/consts/constant.FAMILY.html>
    // format!("{}", env::consts::FAMILY)
    env::consts::FAMILY.to_owned() // TODO Maybe move to a new source "env.rs"?
}

fn build_arch(_environment: &mut Environment) -> String {
    // See here for possible values:
    // <https://doc.rust-lang.org/std/env/consts/constant.ARCH.html>
    // Most common values: "x86", "x86_64"
    env::consts::ARCH.to_owned() // TODO Maybe move to a new source "env.rs"?
}

/// This uses an alternative method to fetch certain specific variable keys values.
/// Alternative meaning here:
/// Not directly fetching it from any environment variable.
impl super::VarSource for VarSource {
    fn is_usable(&self, environment: &mut Environment) -> bool {
        environment.repo().is_some()
    }

    fn retrieve(&self, environment: &mut Environment, key: Key) -> BoxResult<Option<String>> {
        Ok(match key {
            Key::Version | Key::BuildIdent => version(environment)?,
            Key::Name => name(environment)?,
            Key::BuildDate => Some(build_date(environment)),
            Key::BuildOs => Some(build_os(environment)),
            Key::BuildOsFamily => Some(build_os_family(environment)),
            Key::BuildArch => Some(build_arch(environment)),
            Key::RepoWebUrl
            | Key::RepoVersionedWebUrl
            | Key::RepoIssuesUrl
            | Key::BuildBranch
            | Key::BuildTag
            | Key::RepoCloneUrl
            | Key::VersionDate
            | Key::BuildHostingUrl
            | Key::Ci
            | Key::License
            | Key::BuildNumber => None,
        })
    }
}

impl fmt::Display for VarSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::any::type_name::<VarSource>())
    }
}
