// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use fake::uuid::UUIDv5;
use fake::Fake;
use projvar::var;
use regex::Regex;
use uuid::Uuid;

use assert_cmd::prelude::*;
use lazy_static::lazy_static;
use std::{collections::HashMap, env, fmt::Display, path::PathBuf, process::Command};

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
#[error("{children:?}")]
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

pub fn projvar_test(
    expected_pats: &HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>,
    args: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    // let tmp_out_file = assert_fs::NamedTempFile::new("projvar.out.env")?;
    // tmp_out_file.touch()?;
    // let out_file = tmp_out_file.path();
    // NOTE Use this instead of the above for debugging
    let out_file = PathBuf::from("/tmp/projvar-test-out.env");

    let mut cmd = Command::cargo_bin("projvar")?;
    cmd.arg("-O").arg(&out_file.display().to_string());
    cmd.args(args);

    cmd.assert().success();

    let mut output_reader =
        repvar::tools::create_input_reader(Some(&out_file.display().to_string()))?;
    let mut actual_vars = var::parse_vars_file_reader(&mut output_reader)?;

    compare(expected_pats, &mut actual_vars)?;

    Ok(())
}

pub fn projvar_test_all(
    expected_pats: &HashMap<&'static str, (Box<&'static dyn StrMatcher>, bool)>,
) -> Result<(), Box<dyn std::error::Error>> {
    projvar_test(expected_pats, &["--all"])
}

pub fn clear_env_vars() {
    let vars: Vec<String> = env::vars().map(|(key, _val)| key).collect();
    for var in vars {
        env::remove_var(var);
    }
}
