// SPDX-FileCopyrightText: 2021 - 2023 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, PartialEq, Eq)]
pub struct PartsRef<'a> {
    pub protocol: Option<&'a str>,
    pub user: Option<&'a str>,
    pub host: &'a str,
    pub path_and_rest: &'a str,
}

macro_rules! let_named_cap_opt {
    ($caps:ident,$name:ident) => {
        let $name = $caps.name(stringify!($name)).map(|mtch| mtch.as_str());
    };
}
macro_rules! let_named_cap {
    ($caps:ident,$name:ident) => {
        let $name = $caps
            .name(stringify!($name))
            .map(|mtch| mtch.as_str())
            .expect(concat!(
                "Required regex capture not matched: ",
                stringify!($name)
            ));
    };
}
//pub(crate) use let_named_cap;

impl<'a> PartsRef<'a> {
    /// Parses a git clone URL of any type -
    /// including non URL spec compliant ones -
    /// into a set of basic parts.
    ///
    /// # Errors
    ///
    /// If our internal regex to parse a git clone URL
    /// does not match the supplied string.
    pub fn parse<'b>(any_clone_url: &'b str) -> Result<Self, String>
    where
        'b: 'a,
    {
        lazy_static! {
            // This matches all these 3 types of clone URLs:
            // * git@github.com:hoijui/rust-project-scripts.git
            // * ssh://github.com/hoijui/rust-project-scripts.git
            // * https://github.com/hoijui/rust-project-scripts.git
            static ref R_CLONE_URL: Regex = Regex::new(r"^((?P<protocol>[0-9a-zA-Z._-]+)://)?((?P<user>[0-9a-zA-Z._-]+)@)?(?P<host>[0-9a-zA-Z._-]+)([/:](?P<path_and_rest>.+)?)?$").unwrap();
        }

        R_CLONE_URL
            .captures(any_clone_url.as_ref())
            .map(|caps| {
                let_named_cap_opt!(caps, protocol);
                let_named_cap_opt!(caps, user);
                let_named_cap!(caps, host);
                let_named_cap!(caps, path_and_rest);
                Self {
                    protocol,
                    user,
                    host,
                    path_and_rest,
                }
            })
            .ok_or_else(|| {
                format!("Failed to parse as (any type of) git clone URL: '{any_clone_url}'")
            })
    }
}
