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
use clap::lazy_static::lazy_static;
use std::path::{Path, PathBuf};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, EnumVariantNames, IntoStaticStr};

#[derive(
    Debug, EnumString, EnumVariantNames, EnumIter, IntoStaticStr, PartialEq, PartialOrd, Copy, Clone,
)]
pub enum Verbosity {
    None,
    Errors,
    Warnings,
    Info,
    Debug,
    Trace,
}

lazy_static! {
    static ref VARIANTS_LIST: Vec<Verbosity> = Verbosity::iter().collect();
}

impl Verbosity {
    fn index(self) -> usize {
        self as usize
    }
    // pub fn more(self) -> Self {
    //     let new_index = self.index().saturating_add(1) % VARIANTS_LIST.len();
    //     VARIANTS_LIST[new_index]
    // }
    // pub fn less(self) -> Self {
    //     let new_index = self.index().saturating_sub(1);
    //     VARIANTS_LIST[new_index]
    // }

    /// Increases the verbosity by `steps`,
    /// halting at the upper bound of the enum.
    #[must_use]
    pub fn up_max(self, steps: usize) -> Self {
        let new_index = self.index().saturating_add(steps) % VARIANTS_LIST.len();
        VARIANTS_LIST[new_index]
    }

    /// Decreases the verbosity by `steps`,
    /// halting at the lower bound of the enum.
    #[must_use]
    pub fn down_max(self, steps: usize) -> Self {
        let new_index = self.index().saturating_sub(steps);
        VARIANTS_LIST[new_index]
    }
}

// impl Add<usize> for Verbosity {
//     type Output = Self;

//     fn add(self, other: usize) -> Self {
//         let new_index = self.index().saturating_add(other) % VARIANTS_LIST.len();
//         VARIANTS_LIST[new_index]
//     }
// }

// impl Sub<usize> for Verbosity {
//     type Output = Self;

//     fn sub(self, other: usize) -> Self {
//         let new_index = self.index().saturating_sub(other) % VARIANTS_LIST.len();
//         VARIANTS_LIST[new_index]
//     }
// }

// impl Add<i16> for Verbosity {
//     type Output = Self;

//     fn add(self, other: i16) -> Self {
//         let new_index = ((self as i16 + other) % VARIANTS_LIST.len() as i16) as usize;
//         VARIANTS_LIST[new_index]
//     }
// }

// impl Sub<i16> for Verbosity {
//     type Output = Self;

//     fn sub(self, other: i16) -> Self {
//         self.add(-other)
//     }
// }

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

impl VarsToSet {
    #[must_use]
    pub fn main(&self) -> bool {
        match self {
            VarsToSet::All | VarsToSet::Primary => true,
        }
    }

    #[must_use]
    pub fn alt(&self) -> bool {
        match self {
            VarsToSet::All => true,
            VarsToSet::Primary => false,
        }
    }
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

#[derive(Debug, EnumString, EnumVariantNames, IntoStaticStr)]
pub enum Overwrite {
    All,
    None,
    Main,
    Alternative,
}

impl Overwrite {
    #[must_use]
    pub fn main(&self) -> bool {
        match self {
            Overwrite::All | Overwrite::Main => true,
            Overwrite::None | Overwrite::Alternative => false,
        }
    }

    #[must_use]
    pub fn alt(&self) -> bool {
        match self {
            Overwrite::All | Overwrite::Alternative => true,
            Overwrite::None | Overwrite::Main => false,
        }
    }
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
    pub verbosity: (Verbosity, Verbosity),
}
