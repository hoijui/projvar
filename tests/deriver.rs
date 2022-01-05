use std::{collections::HashMap, env};

// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod common;
mod repo_creation;

use common::{StrMatcher, R_DATE_TIME, R_NON_EMPTY};

use crate::repo_creation::{create_repo, create_repo_common};

pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
    let repo_dir = create_repo!(
        crate::repo_creation::default::create,
        "repo_creation/default.rs"
    )?;
    // let repo_dir = create_repo_common!(default)?;
    env::set_current_dir(repo_dir)?;
    Ok(())
}

pub fn expected_pats(
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
        (
            "PROJECT_LICENSES",
            (Box::new(&"AGPL-3.0-or-later, CC0-1.0, Unlicense"), true),
        ),
        ("PROJECT_NAME", (Box::new(&"default.rs"), true)),
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
fn deriver() -> Result<(), Box<dyn std::error::Error>> {
    setup()?;
    common::projvar_test(
        &expected_pats()?,
        &[
            "--fail",
            "--only-required",
            "--none",
            "-RPROJECT_BUILD_DATE",
            "-RPROJECT_LICENSES",
            "-RPROJECT_REPO_WEB_URL",
            "-RPROJECT_NAME",
            "-RPROJECT_VERSION",
            "-RPROJECT_VERSION_DATE",
        ],
    )
}
