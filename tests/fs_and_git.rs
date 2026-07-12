// SPDX-FileCopyrightText: 2021 - 2026 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod common;
mod repo_creation;

use cli_utils::BoxResult;
use common::R_BOOL;
use common::R_DATE_TIME;
use common::R_NON_EMPTY;
use common::StrMatcher;
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use crate::repo_creation::create_repo;

static R_CLONE_URL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(((https|ssh)://github\.com/hoijui/projvar(\.git)?)|((git@)github\.com:hoijui/projvar(\.git)?))$").unwrap()
});
static R_CLONE_URL_HTTP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^https://github\.com/hoijui/projvar(\.git)?$").unwrap());
static R_CLONE_URL_SSH: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^ssh://(git@)github\.com/hoijui/projvar(\.git)?$").unwrap());

fn setup() -> BoxResult<(PathBuf, HashMap<&'static str, &'static str>)> {
    let repo_dir = create_repo!(
        crate::repo_creation::default::create,
        "repo_creation/default.rs"
    )?;
    Ok((repo_dir, HashMap::<&'static str, &'static str>::new()))
}

fn expected_pats() -> BoxResult<HashMap<&'static str, (&'static dyn StrMatcher, bool)>> {
    Ok(vec![
        (
            "PROJECT_BUILD_DATE",
            (&*R_DATE_TIME as &'static dyn StrMatcher, true),
        ),
        ("PROJECT_BUILD_ARCH", (&*R_NON_EMPTY, true)),
        ("PROJECT_BUILD_BRANCH", (&*R_NON_EMPTY, false)),
        (
            "PROJECT_BUILD_HOSTING_URL",
            (&"https://hoijui.github.io/projvar", true),
        ),
        ("PROJECT_BUILD_OS", (&*R_NON_EMPTY, true)),
        ("PROJECT_BUILD_OS_FAMILY", (&*R_NON_EMPTY, true)),
        ("PROJECT_BUILD_TAG", (&*R_NON_EMPTY, false)),
        ("PROJECT_CI", (&*R_BOOL, true)),
        ("PROJECT_LICENSE", (&"AGPL-3.0-only", true)),
        (
            "PROJECT_LICENSES",
            (&"AGPL-3.0-or-later, CC0-1.0, Unlicense", true),
        ),
        ("PROJECT_NAME", (&"default.rs", true)),
        ("PROJECT_NAME_MACHINE_READABLE", (&"default_rs", true)),
        ("PROJECT_REPO_CLONE_URL", (&*R_CLONE_URL, true)),
        ("PROJECT_REPO_CLONE_URL_HTTP", (&*R_CLONE_URL_HTTP, true)),
        ("PROJECT_REPO_CLONE_URL_SSH", (&*R_CLONE_URL_SSH, true)),
        (
            "PROJECT_REPO_COMMIT_PREFIX_URL",
            (&"https://github.com/hoijui/projvar/commit", true),
        ),
        (
            "PROJECT_REPO_ISSUES_URL",
            (&"https://github.com/hoijui/projvar/issues", true),
        ),
        (
            "PROJECT_REPO_RAW_VERSIONED_PREFIX_URL",
            (&"https://raw.githubusercontent.com/hoijui/projvar", true),
        ),
        (
            "PROJECT_REPO_VERSIONED_DIR_PREFIX_URL",
            (&"https://github.com/hoijui/projvar/tree", true),
        ),
        (
            "PROJECT_REPO_VERSIONED_FILE_PREFIX_URL",
            (&"https://github.com/hoijui/projvar/blob", true),
        ),
        (
            "PROJECT_REPO_WEB_URL",
            (&"https://github.com/hoijui/projvar", true),
        ),
        ("PROJECT_VERSION", (&*R_NON_EMPTY, true)),
        ("PROJECT_VERSION_DATE", (&*R_DATE_TIME, true)),
    ]
    .into_iter()
    .collect())
}

#[test]
fn git() -> BoxResult<()> {
    let (cwd, envs) = setup()?;
    common::projvar_test(&expected_pats()?, &["--all"], &cwd, envs)
}
