use std::{collections::HashMap, env};

// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod common;

use common::StrMatcher;

const CI: &str = "true";
const CI_COMMIT_AUTHOR: &str = "Commit Author <commit.author@email.com>";
const CI_COMMIT_BRANCH: [Option<&str>; 5] = [
    Some("master"),
    Some("main"),
    Some("develop"),
    Some("something"),
    None,
];
const CI_COMMIT_REF_NAME: [Option<&str>; 5] = [
    Some("master"),
    Some("v0.1.0"),
    Some("0.1.0"),
    Some("-Some_Thing-"),
    None,
];
const CI_COMMIT_REF_SLUG: [Option<&str>; 5] = [
    Some("master"),
    Some("v0-1-0"),
    Some("0-1-0"),
    Some("some-thing"),
    None,
];
const CI_COMMIT_SHA: &str = "ffac537e6cbbf934b08745a378932722df287a53";
const CI_COMMIT_SHORT_SHA: &str = "ffac537e";
const CI_COMMIT_TAG: [Option<&str>; 4] =
    [Some("v0.1.0"), Some("0.1.0"), Some("-Some_Thing-"), None];
const CI_COMMIT_TIMESTAMP: &str = "2021-12-23T07:25:21+00:00";
const CI_DEBUG_TRACE: [&str; 2] = ["true", "false"];
const CI_DEFAULT_BRANCH: [&str; 4] = ["master", "main", "develop", "something"];
const CI_PAGES_DOMAIN: [Option<&str>; 3] = [Some("gitlab.io"), Some("our-own-domain.de"), None];
const CI_PAGES_URL: [Option<&str>; 3] = [
    Some("https://my-org.gitlab.io/my-proj"),
    Some("https://my-org.our-own-domain.de/my-proj"),
    None,
];
const CI_PROJECT_DIR: &str = "/some/dir/my-proj";
const CI_PROJECT_ID: fn() -> String = common::random_uuid;
const CI_PROJECT_NAME: &str = "Project-1";
const CI_PROJECT_NAMESPACE: [&str; 3] = [
    "User-Name",
    "User-Name/group-name",
    "User-Name/group-name/sub-group-name",
];
const CI_PROJECT_PATH_SLUG: [&str; 3] = [
    "user-Name/project-1",
    "user-Name/group-name/project-1",
    "user-Name/group-name/sub-group-name/project-1",
];
const CI_PROJECT_PATH: [&str; 3] = [
    "User-Name/Project-1",
    "User-Name/group-name/Project-1",
    "User-Name/group-name/sub-group-name/Project-1",
];
const CI_PROJECT_REPOSITORY_LANGUAGES: &str = "ruby,javascript,html,css";
const CI_PROJECT_ROOT_NAMESPACE: &str = "user-name";
const CI_PROJECT_TITLE: &str = "Project One";
const CI_PROJECT_URL: [&str; 6] = [
    "https:constgitlab.com/User-Name/Project-1",
    "https:constgitlab.com/User-Name/group-name/Project-1",
    "https:constgitlab.com/User-Name/group-name/sub-group-name/Project-1",
    "https:constgitlab.our-domain.de/User-Name/Project-1",
    "https:constgitlab.our-domain.de/User-Name/group-name/Project-1",
    "https:constgitlab.our-domain.de/User-Name/group-name/sub-group-name/Project-1",
];
const CI_PROJECT_VISIBILITY: [&str; 3] = ["internal", "private", "public"];
const CI_SERVER_HOST: [&str; 3] = ["gitlab.com", "gitlab.example.com", "gitlab.our-domain.de"];
const CI_SERVER_PORT: [&str; 2] = ["80", "8080"];
const CI_SERVER_PROTOCOL: [&str; 2] = ["https", "http"];
const CI_SERVER_URL: [&str; 3] = [
    "https:constgitlab.com",
    "https:constgitlab.example.com:8080",
    "https:constgitlab.our-domain.de:80",
];
const CI_SERVER: &str = "yes";
const GITLAB_CI: &str = "true";
const GITLAB_USER_EMAIL: &str = "job.triggerer@email.com";
const GITLAB_USER_ID: fn() -> String = common::random_uuid;
const GITLAB_USER_LOGIN: &str = "jobtriggerer";
const GITLAB_USER_NAME: &str = "Job Triggerer";

pub fn setup() -> Result<(), Box<dyn std::error::Error>> {
    common::clear_env_vars();
    env::set_var("CI", CI);
    env::set_var("CI_COMMIT_AUTHOR", CI_COMMIT_AUTHOR);
    env::set_var("CI_COMMIT_BRANCH", CI_COMMIT_BRANCH[0].unwrap());
    env::set_var("CI_COMMIT_REF_NAME", CI_COMMIT_REF_NAME[0].unwrap());
    env::set_var("CI_COMMIT_REF_SLUG", CI_COMMIT_REF_SLUG[0].unwrap());
    env::set_var("CI_COMMIT_SHA", CI_COMMIT_SHA);
    env::set_var("CI_COMMIT_SHORT_SHA", CI_COMMIT_SHORT_SHA);
    env::set_var("CI_COMMIT_TAG", CI_COMMIT_TAG[0].unwrap());
    env::set_var("CI_COMMIT_TIMESTAMP", CI_COMMIT_TIMESTAMP);
    env::set_var("CI_DEBUG_TRACE", CI_DEBUG_TRACE[0]);
    env::set_var("CI_DEFAULT_BRANCH", CI_DEFAULT_BRANCH[0]);
    env::set_var("CI_PAGES_DOMAIN", CI_PAGES_DOMAIN[0].unwrap());
    env::set_var("CI_PAGES_URL", CI_PAGES_URL[0].unwrap());
    env::set_var("CI_PROJECT_DIR", CI_PROJECT_DIR);
    env::set_var("CI_PROJECT_ID", CI_PROJECT_ID());
    env::set_var("CI_PROJECT_NAME", CI_PROJECT_NAME);
    env::set_var("CI_PROJECT_NAMESPACE", CI_PROJECT_NAMESPACE[0]);
    env::set_var("CI_PROJECT_PATH_SLUG", CI_PROJECT_PATH_SLUG[0]);
    env::set_var("CI_PROJECT_PATH", CI_PROJECT_PATH[0]);
    env::set_var(
        "CI_PROJECT_REPOSITORY_LANGUAGES",
        CI_PROJECT_REPOSITORY_LANGUAGES,
    );
    env::set_var("CI_PROJECT_ROOT_NAMESPACE", CI_PROJECT_ROOT_NAMESPACE);
    env::set_var("CI_PROJECT_TITLE", CI_PROJECT_TITLE);
    env::set_var("CI_PROJECT_URL", CI_PROJECT_URL[0]);
    env::set_var("CI_PROJECT_VISIBILITY", CI_PROJECT_VISIBILITY[0]);
    env::set_var("CI_SERVER_HOST", CI_SERVER_HOST[0]);
    env::set_var("CI_SERVER_PORT", CI_SERVER_PORT[0]);
    env::set_var("CI_SERVER_PROTOCOL", CI_SERVER_PROTOCOL[0]);
    env::set_var("CI_SERVER_URL", CI_SERVER_URL[0]);
    env::set_var("CI_SERVER", CI_SERVER);
    env::set_var("GITLAB_CI", GITLAB_CI);
    env::set_var("GITLAB_USER_EMAIL", GITLAB_USER_EMAIL);
    env::set_var("GITLAB_USER_ID", GITLAB_USER_ID());
    env::set_var("GITLAB_USER_LOGIN", GITLAB_USER_LOGIN);
    env::set_var("GITLAB_USER_NAME", GITLAB_USER_NAME);
    Ok(())
}

pub fn expected_pats(
) -> Result<HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>, Box<dyn std::error::Error>>
{
    Ok(vec![
        (
            "PROJECT_BUILD_BRANCH",
            (
                Box::new(&"master" as &'static dyn StrMatcher) as Box<&'static dyn StrMatcher>,
                true,
            ),
        ),
        (
            "PROJECT_BUILD_HOSTING_URL",
            (Box::new(&"https://my-org.gitlab.io/my-proj"), true),
        ),
        ("PROJECT_BUILD_TAG", (Box::new(&"v0.1.0"), true)),
        ("PROJECT_CI", (Box::new(&"true"), true)),
        ("PROJECT_NAME", (Box::new(&"Project-1"), true)),
        (
            "PROJECT_NAME_MACHINE_READABLE",
            (Box::new(&"Project-1"), true),
        ),
        (
            "PROJECT_REPO_WEB_URL",
            (Box::new(&"https:constgitlab.com/User-Name/Project-1"), true),
        ),
        ("PROJECT_VERSION", (Box::new(&"0.1.0"), true)),
        (
            "PROJECT_VERSION_DATE",
            (Box::new(&"2021-12-23 07:25:21"), true),
        ),
    ]
    .into_iter()
    .collect())
}

#[test]
fn gitlab_ci() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_proj_dir_empty = assert_fs::TempDir::new()?;
    env::set_current_dir(tmp_proj_dir_empty)?;
    setup()?;
    common::projvar_test_all(&expected_pats()?)
}
