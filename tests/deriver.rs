// SPDX-FileCopyrightText: 2021 - 2026 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod common;
mod repo_creation;

use std::{collections::HashMap, path::PathBuf};

use cli_utils::BoxResult;
use common::{R_DATE_TIME, R_NON_EMPTY, StrMatcher};

use crate::repo_creation::create_repo;

fn setup() -> BoxResult<(PathBuf, HashMap<&'static str, &'static str>)> {
    let repo_dir = create_repo!(
        crate::repo_creation::default::create,
        "repo_creation/default.rs"
    )?;
    Ok((repo_dir, HashMap::<&'static str, &'static str>::new()))
}

fn expected_pats() -> HashMap<&'static str, (&'static dyn StrMatcher, bool)> {
    vec![
        (
            "PROJECT_BUILD_DATE",
            (&*R_DATE_TIME as &'static dyn StrMatcher, true),
        ),
        (
            "PROJECT_LICENSES",
            (&"AGPL-3.0-or-later, CC0-1.0, Unlicense", true),
        ),
        ("PROJECT_NAME", (&"default.rs", true)),
        (
            "PROJECT_REPO_WEB_URL",
            (&"https://github.com/hoijui/projvar", true),
        ),
        ("PROJECT_VERSION", (&*R_NON_EMPTY, true)),
        ("PROJECT_VERSION_DATE", (&*R_DATE_TIME, true)),
    ]
    .into_iter()
    .collect()
}

#[test]
fn deriver() -> BoxResult<()> {
    let (cwd, envs) = setup()?;
    common::projvar_test(
        &expected_pats(),
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
        &cwd,
        envs,
    )
}
