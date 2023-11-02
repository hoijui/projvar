// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::ValueEnum;
use lazy_static::lazy_static;
use std::{collections::HashSet, path::PathBuf};
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString, EnumVariantNames, IntoStaticStr};
use url::Url;

use crate::{
    constants,
    tools::git_hosting_provs::{HostingType, PublicSite},
    var::Key,
};

#[derive(
    Debug,
    ValueEnum,
    EnumString,
    EnumVariantNames,
    EnumIter,
    IntoStaticStr,
    PartialEq,
    PartialOrd,
    Copy,
    Clone,
)]
pub enum Verbosity {
    None,
    Errors,
    Warnings,
    Info,
    Debug,
    Trace,
}

impl Default for Verbosity {
    fn default() -> Self {
        Self::Info
    }
}

lazy_static! {
    static ref VARIANTS_LIST: Vec<Verbosity> = Verbosity::iter().collect();
}

impl Verbosity {
    fn index(self) -> usize {
        self as usize
    }

    /// Increases the verbosity by `steps`,
    /// halting at the upper bound of the enum.
    #[must_use]
    pub fn up_max<S: Into<usize>>(self, steps: S) -> Self {
        let new_index = self.index().saturating_add(steps.into()) % VARIANTS_LIST.len();
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

impl From<bool> for Verbosity {
    fn from(verbose: bool) -> Self {
        if verbose {
            Verbosity::Info
        } else {
            Verbosity::Warnings
        }
    }
}

#[derive(Debug, ValueEnum, EnumString, EnumVariantNames, IntoStaticStr, Clone, Copy)]
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

impl Default for Overwrite {
    fn default() -> Self {
        Self::All
    }
}

/* impl strum::VariantNames for Overwrite { */
/*     const VARIANTS: &'static [&'static str]; */
/* } */

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Debug)]
pub enum ShowRetrieved {
    No,
    Primary(Option<PathBuf>),
    All(Option<PathBuf>),
}

#[derive(Clone, Debug)]
pub struct Settings /*<S: ::std::hash::BuildHasher>*/ {
    // pub repo_path: Option<Box<Path>>,
    pub repo_path: Option<PathBuf>,
    pub required_keys: HashSet<Key>,
    pub overwrite: Overwrite,
    pub date_format: String,
    pub fail_on: FailOn,
    // vars: Box<HashMap<String, String, S>>,
    // #[builder(default = false)]
    // fail_on_missing: bool,
    pub show_retrieved: ShowRetrieved,
    pub hosting_type: HostingType,
    pub only_required: bool,
    pub key_prefix: Option<String>,
    pub verbosity: Verbosity,
}

impl Settings {
    fn stub() -> Settings {
        let mut all_keys = HashSet::<Key>::new();
        all_keys.extend(Key::iter());
        Settings {
            repo_path: None,
            required_keys: all_keys,
            overwrite: Overwrite::All,
            date_format: crate::tools::git::DATE_FORMAT.to_string(),
            fail_on: FailOn::AnyMissingValue,
            show_retrieved: ShowRetrieved::No,
            hosting_type: HostingType::Unknown,
            only_required: false,
            key_prefix: Some(constants::DEFAULT_KEY_PREFIX.to_owned()),
            verbosity: Verbosity::None,
        }
    }

    #[must_use]
    pub fn hosting_type(&self, url: &Url) -> HostingType {
        if let HostingType::Unknown = self.hosting_type {
            HostingType::from(PublicSite::from(url.host()))
        } else {
            self.hosting_type
        }
    }

    #[must_use]
    pub fn hosting_type_from_host(&self, host: &str) -> HostingType {
        if let HostingType::Unknown = self.hosting_type {
            let host_assumed_domain = url::Host::Domain(host);
            HostingType::from(PublicSite::from(host_assumed_domain))
        } else {
            self.hosting_type
        }
    }

    #[must_use]
    pub fn hosting_type_from_hosting_suffix(&self, url: &Url) -> HostingType {
        if let HostingType::Unknown = self.hosting_type {
            HostingType::from(PublicSite::from_hosting_domain_option(url.host().as_ref()))
        } else {
            self.hosting_type
        }
    }
}

lazy_static! {
    pub static ref STUB: Settings = Settings::stub();
}
