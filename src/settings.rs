// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// use dict::Dict;
// use std::io;
// use std::io::BufRead;
// use std::io::Write;
// use git2;

// #[derive(Debug)]
// #[derive(TypedBuilder)]
use std::path::{Path, PathBuf};

pub enum Verbosity {
    Warnings,
    Info,
}

impl From<bool> for Verbosity {
    fn from(verbose: bool) -> Self {
        if verbose {
            Verbosity::Info
        } else {
            Verbosity::Warnings
        }
    }
}

pub enum VarsToSet {
    Primary,
    All,
}

impl From<bool> for VarsToSet {
    fn from(verbose: bool) -> Self {
        if verbose {
            VarsToSet::All
        } else {
            VarsToSet::Primary
        }
    }
}

// use strum::IntoEnumIterator;
use strum_macros::{EnumString, EnumVariantNames, IntoStaticStr};

#[derive(Debug, EnumString, EnumVariantNames, IntoStaticStr)]
pub enum Overwrite {
    All,
    None,
    Main,
    Alternative,
}

/* impl strum::VariantNames for Overwrite { */
/*     const VARIANTS: &'static [&'static str]; */
/* } */

pub enum FailOn {
    AnyMissingValue,
    Error,
}

impl From<bool> for FailOn {
    fn from(verbose: bool) -> Self {
        if verbose {
            FailOn::AnyMissingValue
        } else {
            FailOn::Error
        }
    }
}

pub enum StorageMode {
    Environment,
    ToFile(PathBuf),
    Dry,
}

pub struct Settings /*<S: ::std::hash::BuildHasher>*/ {
    // pub repo_path: Option<Box<Path>>,
    pub repo_path: Option<&'static Path>,
    pub to_set: VarsToSet,
    pub overwrite: Overwrite,
    pub date_format: String,
    pub fail_on: FailOn,
    // vars: Box<HashMap<String, String, S>>,
    // #[builder(default = false)]
    // fail_on_missing: bool,
    // #[builder(default = false)]
    pub storage: StorageMode,
    pub verbosity: Verbosity,
}
