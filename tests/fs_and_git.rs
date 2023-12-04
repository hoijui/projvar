// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod common;
mod repo_creation;

use cli_utils::BoxResult;
use common::StrMatcher;
use common::R_BOOL;
use common::R_DATE_TIME;
use common::R_NON_EMPTY;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::repo_creation::create_repo;

lazy_static! {
    pub static ref R_CLONE_URL: Regex = Regex::new(r"^(((https|ssh)://github\.com/hoijui/projvar(\.git)?)|((git@)github\.com:hoijui/projvar(\.git)?))$").unwrap();
    pub static ref R_CLONE_URL_HTTP: Regex = Regex::new(r"^https://github\.com/hoijui/projvar(\.git)?$").unwrap();
    pub static ref R_CLONE_URL_SSH: Regex = Regex::new(r"^ssh://(git@)github\.com/hoijui/projvar(\.git)?$").unwrap();
}

fn setup() -> BoxResult<(PathBuf, HashMap<&'static str, &'static str>)> {
    let repo_dir = create_repo!(
        crate::repo_creation::default::create,
        "repo_creation/default.rs"
    )?;
    Ok((repo_dir, HashMap::<&'static str, &'static str>::new()))
}

fn expected_pats() -> BoxResult<HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>> {
    Ok(vec![
        (
            "PROJECT_BUILD_DATE",
            (
                Box::new(&*R_DATE_TIME as &'static dyn StrMatcher) as Box<&'static dyn StrMatcher>,
                true,
            ),
        ),
        ("PROJECT_BUILD_ARCH", (Box::new(&*R_NON_EMPTY), true)),
        ("PROJECT_BUILD_BRANCH", (Box::new(&*R_NON_EMPTY), false)),
        (
            "PROJECT_BUILD_HOSTING_URL",
            (Box::new(&"https://hoijui.github.io/projvar"), true),
        ),
        ("PROJECT_BUILD_OS", (Box::new(&*R_NON_EMPTY), true)),
        ("PROJECT_BUILD_OS_FAMILY", (Box::new(&*R_NON_EMPTY), true)),
        ("PROJECT_BUILD_TAG", (Box::new(&*R_NON_EMPTY), false)),
        ("PROJECT_CI", (Box::new(&*R_BOOL), true)),
        ("PROJECT_LICENSE", (Box::new(&"AGPL-3.0-only"), true)),
        (
            "PROJECT_LICENSES",
            (Box::new(&"AGPL-3.0-or-later, CC0-1.0, Unlicense"), true),
        ),
        ("PROJECT_NAME", (Box::new(&"default.rs"), true)),
        (
            "PROJECT_NAME_MACHINE_READABLE",
            (Box::new(&"default_rs"), true),
        ),
        ("PROJECT_REPO_CLONE_URL", (Box::new(&*R_CLONE_URL), true)),
        (
            "PROJECT_REPO_CLONE_URL_HTTP",
            (Box::new(&*R_CLONE_URL_HTTP), true),
        ),
        (
            "PROJECT_REPO_CLONE_URL_SSH",
            (Box::new(&*R_CLONE_URL_SSH), true),
        ),
        (
            "PROJECT_REPO_COMMIT_PREFIX_URL",
            (Box::new(&"https://github.com/hoijui/projvar/commit"), true),
        ),
        (
            "PROJECT_REPO_ISSUES_URL",
            (Box::new(&"https://github.com/hoijui/projvar/issues"), true),
        ),
        (
            "PROJECT_REPO_RAW_VERSIONED_PREFIX_URL",
            (
                Box::new(&"https://raw.githubusercontent.com/hoijui/projvar"),
                true,
            ),
        ),
        (
            "PROJECT_REPO_VERSIONED_DIR_PREFIX_URL",
            (Box::new(&"https://github.com/hoijui/projvar/tree"), true),
        ),
        (
            "PROJECT_REPO_VERSIONED_FILE_PREFIX_URL",
            (Box::new(&"https://github.com/hoijui/projvar/blob"), true),
        ),
        (
            "PROJECT_REPO_WEB_URL",
            (Box::new(&"https://github.com/hoijui/projvar"), true),
        ),
        ("PROJECT_VERSION", (Box::new(&*R_NON_EMPTY), true)),
        ("PROJECT_VERSION_DATE", (Box::new(&*R_DATE_TIME), true)),
    ]
    .into_iter()
    .collect())
}

#[test]
fn git() -> BoxResult<()> {
    let (cwd, envs) = setup()?;
    common::projvar_test(&expected_pats()?, &["--all"], &cwd, envs)
}
