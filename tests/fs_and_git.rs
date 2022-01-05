// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod common;
mod repo_creation;

use common::StrMatcher;
use common::R_BOOL;
use common::R_DATE_TIME;
use common::R_NON_EMPTY;
use std::{collections::HashMap, env};

use crate::repo_creation::create_repo;

pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
    let repo_dir = create_repo!(
        crate::repo_creation::default::create,
        "repo_creation/default.rs"
    )?;
    env::set_current_dir(repo_dir)?;
    Ok(())
}

pub fn expected_patterns(
) -> Result<HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>, Box<dyn std::error::Error>>
{
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
        (
            "PROJECT_REPO_CLONE_URL",
            (Box::new(&"https://github.com/hoijui/projvar.git"), true),
        ),
        (
            "PROJECT_REPO_CLONE_URL_SSH",
            (Box::new(&"ssh://git@github.com:hoijui/projvar.git"), true),
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
fn git() -> Result<(), Box<dyn std::error::Error>> {
    setup()?;
    common::projvar_test_all(&expected_patterns()?)
}
