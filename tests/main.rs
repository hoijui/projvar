// SPDX-FileCopyrightText: 2022 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod common;

use common::StrMatcher;
use projvar::BoxResult;
use std::collections::HashMap;

fn expected_pats() -> BoxResult<HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>> {
    Ok(vec![(
        "PROJECT_CI",
        (
            Box::new(&"false" as &'static dyn StrMatcher) as Box<&'static dyn StrMatcher>,
            true,
        ),
    )]
    .into_iter()
    .collect())
}

#[test]
fn cli_arg_all() -> BoxResult<()> {
    common::projvar_test_clean(&expected_pats()?, &["--all"])
}

#[test]
fn cli_arg_variable() -> BoxResult<()> {
    common::projvar_test_clean(&expected_pats()?, &["--variable", "A=a"])?;
    common::projvar_test_clean(&expected_pats()?, &["--variable=B=b"])?;
    common::projvar_test_clean(&expected_pats()?, &["-D C=c"])?;
    common::projvar_test_clean(&expected_pats()?, &["-D=D=d"])?;
    common::projvar_test_clean(&expected_pats()?, &["-DE=e"])
}

fn expected_pats_tuetue() -> BoxResult<HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>>
{
    Ok(vec![(
        "TUETUE_CI",
        (
            Box::new(&"false" as &'static dyn StrMatcher) as Box<&'static dyn StrMatcher>,
            true,
        ),
    )]
    .into_iter()
    .collect())
}

#[test]
fn cli_arg_key_prefix() -> BoxResult<()> {
    common::projvar_test_clean(&expected_pats_tuetue()?, &["--key-prefix", "TUETUE_"])
}

#[test]
fn cli_arg_require() -> BoxResult<()> {
    common::projvar_test_clean(
        &expected_pats()?,
        &["--none", "--fail", "--require", "PROJECT_CI"],
    )?;
    common::projvar_test_clean(&expected_pats()?, &["-n", "-f", "-R", "Ci"])
}

#[test]
fn cli_arg_date_format() -> BoxResult<()> {
    common::projvar_test_clean(&expected_pats()?, &["--date-format", "%y"])?;
    common::projvar_test_clean(&expected_pats()?, &["-T", "%m"])
}

#[test]
fn cli_arg_no_env_in() -> BoxResult<()> {
    common::projvar_test_clean(&expected_pats()?, &["--no-env-in"])?;
    common::projvar_test_clean(&expected_pats()?, &["-x"])
}

#[test]
fn cli_arg_hosting_type() -> BoxResult<()> {
    common::projvar_test_clean(&expected_pats()?, &["--hosting-type", "git-hub"])?;
    common::projvar_test_clean(&expected_pats()?, &["-t", "source-hut"])
}

#[test]
fn cli_arg_only_required() -> BoxResult<()> {
    common::projvar_test_clean(&HashMap::new(), &["--only-required"])
}

#[test]
fn cli_arg_overwrite() -> BoxResult<()> {
    common::projvar_test_clean(&expected_pats()?, &["--overwrite", "all"])?;
    common::projvar_test_clean(&expected_pats()?, &["-o", "all"])?;
    common::projvar_test_clean(&expected_pats()?, &["-o", "none"])?;
    common::projvar_test_clean(&expected_pats()?, &["-o", "main"])?;
    common::projvar_test_clean(&expected_pats()?, &["-o", "alternative"])
}

#[test]
fn cli_arg_log_level() -> BoxResult<()> {
    common::projvar_test_clean(&expected_pats()?, &["--log-level", "trace"])
}

#[test]
fn cli_arg_verbose() -> BoxResult<()> {
    common::projvar_test_clean(&expected_pats()?, &["--verbose"])?;
    common::projvar_test_clean(&expected_pats()?, &["-v"])?;
    common::projvar_test_clean(&expected_pats()?, &["-v", "-v"])?;
    common::projvar_test_clean(&expected_pats()?, &["-v", "-v", "-v"])?;
    common::projvar_test_clean(&expected_pats()?, &["-v", "-v", "-v", "-v"])
}

#[test]
fn cli_arg_defaults() -> BoxResult<()> {
    common::projvar_test_clean(&expected_pats()?, &[])
}
