// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod common;

use common::StrMatcher;
use projvar::BoxResult;
use std::{collections::HashMap, env};

const CI: &str = "true";
const GITHUB_ACTOR: &str = "octocat";
const GITHUB_REPOSITORY: &str = "octocat/Hello-World";
const GITHUB_SHA: &str = "ffac537e6cbbf934b08745a378932722df287a53";
const GITHUB_SERVER_URL: &str = "https://github.com";
const GITHUB_API_URL: &str = "https://api.github.com";
const GITHUB_REF: [Option<&str>; 6] = [
    Some("refs/heads/feature-branch-1"),
    Some("refs/tags/0.1.0"),
    Some("refs/tags/hello-world-0.1.0"),
    Some("refs/tags/hello-world-v0.1.0"),
    Some("refs/tags/some-tag"),
    None,
];
const GITHUB_REF_NAME: [Option<&str>; 4] = [
    Some("feature-branch-1"),
    Some("v0.1.0"),
    Some("0.1.0"),
    None,
];
const GITHUB_REF_TYPE: [Option<&str>; 3] = [Some("branch"), Some("tag"), None];
const GITHUB_HEAD_REF: [Option<&str>; 2] = [Some("head-branch"), None];
const GITHUB_BASE_REF: [Option<&str>; 2] = [Some("base-branch"), None];

fn setup() -> BoxResult<()> {
    common::clear_env_vars();
    env::set_var("CI", CI);
    env::set_var("GITHUB_ACTOR", GITHUB_ACTOR);
    env::set_var("GITHUB_REPOSITORY", GITHUB_REPOSITORY);
    env::set_var("GITHUB_SHA", GITHUB_SHA);
    env::set_var("GITHUB_SERVER_URL", GITHUB_SERVER_URL);
    env::set_var("GITHUB_API_URL", GITHUB_API_URL);
    env::set_var("GITHUB_REF", GITHUB_REF[0].unwrap());
    env::set_var("GITHUB_REF_NAME", GITHUB_REF_NAME[0].unwrap());
    env::set_var("GITHUB_REF_TYPE", GITHUB_REF_TYPE[0].unwrap());
    env::set_var("GITHUB_HEAD_REF", GITHUB_HEAD_REF[0].unwrap());
    env::set_var("GITHUB_BASE_REF", GITHUB_BASE_REF[0].unwrap());
    Ok(())
}

fn expected_pats() -> BoxResult<HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>> {
    let vars = vec![
        (
            "PROJECT_BUILD_BRANCH",
            (
                Box::new(&"feature-branch-1" as &'static dyn StrMatcher)
                    as Box<&'static dyn StrMatcher>,
                true,
            ),
        ),
        (
            "PROJECT_BUILD_HOSTING_URL",
            (Box::new(&"https://octocat.github.io/Hello-World"), true),
        ),
        ("PROJECT_CI", (Box::new(&"true"), true)),
        ("PROJECT_NAME", (Box::new(&"Hello-World"), true)),
        (
            "PROJECT_NAME_MACHINE_READABLE",
            (Box::new(&"Hello-World"), true),
        ),
        (
            "PROJECT_REPO_CLONE_URL",
            (
                Box::new(&"https://github.com/octocat/Hello-World.git"),
                true,
            ),
        ),
        (
            "PROJECT_REPO_CLONE_URL_HTTP",
            (
                Box::new(&"https://github.com/octocat/Hello-World.git"),
                true,
            ),
        ),
        (
            "PROJECT_REPO_CLONE_URL_SSH",
            (
                Box::new(&"ssh://git@github.com/octocat/Hello-World.git"),
                true,
            ),
        ),
        (
            "PROJECT_REPO_COMMIT_PREFIX_URL",
            (
                Box::new(&"https://github.com/octocat/Hello-World/commit"),
                true,
            ),
        ),
        (
            "PROJECT_REPO_ISSUES_URL",
            (
                Box::new(&"https://github.com/octocat/Hello-World/issues"),
                true,
            ),
        ),
        (
            "PROJECT_REPO_RAW_VERSIONED_PREFIX_URL",
            (
                Box::new(&"https://raw.githubusercontent.com/octocat/Hello-World"),
                true,
            ),
        ),
        (
            "PROJECT_REPO_VERSIONED_DIR_PREFIX_URL",
            (
                Box::new(&"https://github.com/octocat/Hello-World/tree"),
                true,
            ),
        ),
        (
            "PROJECT_REPO_VERSIONED_FILE_PREFIX_URL",
            (
                Box::new(&"https://github.com/octocat/Hello-World/blob"),
                true,
            ),
        ),
        (
            "PROJECT_REPO_WEB_URL",
            (Box::new(&"https://github.com/octocat/Hello-World"), true),
        ),
        (
            "PROJECT_VERSION",
            (Box::new(&"ffac537e6cbbf934b08745a378932722df287a53"), true),
        ),
    ];
    Ok(vars.into_iter().collect())
}

#[test]
fn github_ci() -> BoxResult<()> {
    let tmp_proj_dir_empty = assert_fs::TempDir::new()?;
    env::set_current_dir(tmp_proj_dir_empty)?;
    setup()?;
    common::projvar_test_all(&expected_pats()?)
}
