// SPDX-FileCopyrightText: 2021-2023 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use assert_fs::fixture::FileTouch;
use cli_utils::BoxResult;
use fake::uuid::UUIDv5;
use fake::Fake;
use projvar::var;
use regex::Regex;
use uuid::Uuid;

use assert_cmd::prelude::*;
use lazy_static::lazy_static;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::{collections::HashMap, fmt::Display, process::Command};

lazy_static! {
    pub static ref R_DATE_TIME: Regex =
        Regex::new(r"^[12][0-9]{3}-[01]?[0-9]-[0-3]?[0-9] [012]?[0-9]:[0-5]?[0-9]:[0-5]?[0-9]$")
            .unwrap();
    pub static ref R_NON_EMPTY: Regex = Regex::new(r"^.+$").unwrap();
    pub static ref R_BOOL: Regex = Regex::new(r"^(true|false)$").unwrap();
}

pub fn random_uuid() -> String {
    UUIDv5.fake::<Uuid>().to_string()
}

pub trait StrMatcher: Display {
    fn matches(&self, text: &str) -> bool;
}

impl StrMatcher for Regex {
    fn matches(&self, text: &str) -> bool {
        self.is_match(text)
    }
}

impl StrMatcher for &Regex {
    fn matches(&self, text: &str) -> bool {
        self.is_match(text)
    }
}

impl StrMatcher for &str {
    fn matches(&self, text: &str) -> bool {
        self == &text
    }
}

/// This enumerates all possible errors returned by this module.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(
        "For key '{key}' we expected value '{expected_pat}', but actual value is '{actual_val}'."
    )]
    ValuesDiffer {
        key: &'static str,
        expected_pat: String,
        actual_val: String,
    },

    #[error("For key '{key}', value '{expected_pat}' was expected, but none was produced.")]
    MissingValue {
        key: &'static str,
        expected_pat: String,
    },

    #[error("For key '{key}', no value was expected, but '{actual_val}' was produced.")]
    Unexpected { key: String, actual_val: String },
}

/// A Container for multipel errors
/// that may happen during the comparison of two variables containers.
#[derive(thiserror::Error, Debug)]
#[error("{children:#?}")]
pub struct Errors {
    pub children: Vec<Error>,
}

pub fn compare(
    expected: &HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>,
    actual: &mut HashMap<String, String>,
) -> Result<(), Errors> {
    let mut errors = vec![];
    for (key, (expected_pat, required)) in expected.iter() {
        let actual_val = actual.remove(key.to_owned());
        match actual_val {
            Some(actual_val) => {
                if !expected_pat.matches(&actual_val) {
                    errors.push(Error::ValuesDiffer {
                        key,
                        expected_pat: expected_pat.to_string(),
                        actual_val,
                    });
                }
            }
            None => {
                if *required {
                    errors.push(Error::MissingValue {
                        key,
                        expected_pat: expected_pat.to_string(),
                    });
                }
            }
        }
    }

    for (key, actual_val) in actual.iter() {
        errors.push(Error::Unexpected {
            key: key.to_string(),
            actual_val: actual_val.to_string(),
        }); // TODO We should rather use a consuming iterator over `actual` (if such a thing exists... it should!)
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(Errors { children: errors })
    }
}

fn projvar_test_internal<I, K, V>(
    expected_pats: &HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>,
    args: &[&str],
    cwd: &Path,
    envs: I,
    debug: bool,
) -> BoxResult<()>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let tmp_out_file = assert_fs::NamedTempFile::new("projvar.out.env")?;
    tmp_out_file.touch()?;
    let out_file = if debug {
        // NOTE For debugging **A SINGLE TEST**!
        let out_file = PathBuf::from("/tmp/projvar-test-out.env");
        if out_file.exists() {
            fs::remove_file(&out_file)?;
        }
        out_file
    } else {
        tmp_out_file.path().to_path_buf()
    };
    let out_file_str = &out_file.display().to_string();

    let mut cmd = Command::cargo_bin("projvar")?;
    cmd.arg("-O").arg(&out_file_str);
    if debug {
        cmd.arg("-A").arg("/tmp/pv-dbg-out-all.md");
        cmd.arg("-F").arg("trace");
        cmd.arg("-F").arg("warnings");
    }
    cmd.current_dir(cwd);
    cmd.args(args);
    cmd.env_clear();
    cmd.envs(envs);

    if debug {
        let output = cmd.output()?;
        let stdout_utf8 = std::str::from_utf8(&output.stdout)?;
        println!("{stdout_utf8}");
    } else {
        cmd.assert().success();
    }

    assert!(out_file.exists());
    let mut output_reader = cli_utils::create_input_reader(Some(&out_file_str))?;
    let mut actual_vars = var::parse_vars_file_reader(&mut output_reader)?;

    compare(expected_pats, &mut actual_vars)?;

    Ok(())
}

pub fn projvar_test<I, K, V>(
    expected_pats: &HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>,
    args: &[&str],
    cwd: &Path,
    envs: I,
) -> BoxResult<()>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    projvar_test_internal(expected_pats, args, cwd, envs, false)
}

pub fn projvar_test_clean(
    expected_pats: &HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>,
    args: &[&str],
) -> BoxResult<()> {
    let tmp_proj_dir_empty = assert_fs::TempDir::new()?;
    projvar_test_internal(
        expected_pats,
        args,
        tmp_proj_dir_empty.path(),
        HashMap::<String, String>::new(),
        false,
    )
}
