// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;

mod common;

use cli_utils::BoxResult;
use common::StrMatcher;

const CI: &str = "true";
const BITBUCKET_COMMIT: &str = "ffac537e6cbbf934b08745a378932722df287a53";
const BITBUCKET_WORKSPACE: [&str; 2] = ["my-org", "my-user"];
const BITBUCKET_REPO_SLUG: &str = "my-user/my-proj";
const BITBUCKET_REPO_UUID: &str = "123e4567-e89b-12d3-a456-426614174000";
const BITBUCKET_REPO_FULL_NAME: &str = "my-user/my-proj";
const BITBUCKET_BRANCH: [Option<&str>; 5] = [
    Some("master"),
    Some("main"),
    Some("develop"),
    Some("something"),
    None,
];
const BITBUCKET_TAG: [Option<&str>; 4] =
    [Some("v0.1.0"), Some("0.1.0"), Some("-Some_Thing-"), None];
const BITBUCKET_PR_ID: fn() -> String = common::random_uuid;
const BITBUCKET_PR_DESTINATION_BRANCH: [Option<&str>; 5] = [
    Some("master"),
    Some("main"),
    Some("develop"),
    Some("something"),
    None,
];
const BITBUCKET_GIT_HTTP_ORIGIN: &str = "https://bitbucket.org/my-user/my-proj";
const BITBUCKET_GIT_SSH_ORIGIN: &str = "git@bitbucket.org:my-user/my-proj.git";
const BITBUCKET_PROJECT_KEY: [Option<&str>; 2] = [Some("my-project-group"), None];
const BITBUCKET_PROJECT_UUID: &str = "123e4567-e89b-12d3-a456-426614174001";

fn setup() -> BoxResult<HashMap<&'static str, String>> {
    Ok(HashMap::from([
        ("CI", CI.to_owned()),
        ("BITBUCKET_COMMIT", BITBUCKET_COMMIT.to_owned()),
        ("BITBUCKET_WORKSPACE", BITBUCKET_WORKSPACE[0].to_owned()),
        ("BITBUCKET_REPO_SLUG", BITBUCKET_REPO_SLUG.to_owned()),
        ("BITBUCKET_REPO_UUID", BITBUCKET_REPO_UUID.to_owned()),
        (
            "BITBUCKET_REPO_FULL_NAME",
            BITBUCKET_REPO_FULL_NAME.to_owned(),
        ),
        ("BITBUCKET_BRANCH", BITBUCKET_BRANCH[0].unwrap().to_owned()),
        ("BITBUCKET_TAG", BITBUCKET_TAG[0].unwrap().to_owned()),
        ("BITBUCKET_PR_ID", BITBUCKET_PR_ID()),
        (
            "BITBUCKET_PR_DESTINATION_BRANCH",
            BITBUCKET_PR_DESTINATION_BRANCH[0].unwrap().to_owned(),
        ),
        (
            "BITBUCKET_GIT_HTTP_ORIGIN",
            BITBUCKET_GIT_HTTP_ORIGIN.to_owned(),
        ),
        (
            "BITBUCKET_GIT_SSH_ORIGIN",
            BITBUCKET_GIT_SSH_ORIGIN.to_owned(),
        ),
        (
            "BITBUCKET_PROJECT_KEY",
            BITBUCKET_PROJECT_KEY[0].unwrap().to_owned(),
        ),
        ("BITBUCKET_PROJECT_UUID", BITBUCKET_PROJECT_UUID.to_owned()),
    ]))
}

fn expected_pats() -> BoxResult<HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>> {
    Ok(vec![
        (
            "PROJECT_BUILD_BRANCH",
            (
                Box::new(&"master" as &'static dyn StrMatcher) as Box<&'static dyn StrMatcher>,
                true,
            ),
        ),
        ("PROJECT_BUILD_TAG", (Box::new(&"v0.1.0"), true)),
        ("PROJECT_CI", (Box::new(&"true"), true)),
        ("PROJECT_NAME", (Box::new(&"my-project-group"), true)),
        (
            "PROJECT_NAME_MACHINE_READABLE",
            (Box::new(&"my-project-group"), true),
        ),
        (
            "PROJECT_REPO_CLONE_URL",
            (Box::new(&"git@bitbucket.org:my-user/my-proj.git"), true),
        ),
        (
            "PROJECT_REPO_CLONE_URL_HTTP",
            (Box::new(&"https://bitbucket.org/my-user/my-proj"), true),
        ),
        (
            "PROJECT_REPO_CLONE_URL_SSH",
            (
                Box::new(&"ssh://git@bitbucket.org/my-user/my-proj.git"),
                true,
            ),
        ),
        (
            "PROJECT_REPO_COMMIT_PREFIX_URL",
            (
                Box::new(&"https://bitbucket.org/my-user/my-proj/commits"),
                true,
            ),
        ),
        (
            "PROJECT_REPO_ISSUES_URL",
            (
                Box::new(&"https://bitbucket.org/my-user/my-proj/issues"),
                true,
            ),
        ),
        (
            "PROJECT_REPO_RAW_VERSIONED_PREFIX_URL",
            (Box::new(&"https://bitbucket.org/my-user/my-proj/raw"), true),
        ),
        (
            "PROJECT_REPO_VERSIONED_DIR_PREFIX_URL",
            (Box::new(&"https://bitbucket.org/my-user/my-proj/src"), true),
        ),
        (
            "PROJECT_REPO_VERSIONED_FILE_PREFIX_URL",
            (Box::new(&"https://bitbucket.org/my-user/my-proj/src"), true),
        ),
        (
            "PROJECT_REPO_WEB_URL",
            (Box::new(&"https://bitbucket.org/my-user/my-proj"), true),
        ),
        ("PROJECT_VERSION", (Box::new(&"0.1.0"), true)),
    ]
    .into_iter()
    .collect())
}

#[test]
fn bitbucket_ci() -> BoxResult<()> {
    let tmp_proj_dir_empty = assert_fs::TempDir::new()?;
    let envs = setup()?;
    common::projvar_test(&expected_pats()?, &["--all"], &tmp_proj_dir_empty, envs)
}
