// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use askalono::{Store, TextData};
use std::{ffi::OsStr, fs};
use std::{fmt, sync::LazyLock};

const LICENSE_FILE_PREFIXES: [&str; 3] = ["LICENSE", "LICENCE", "COPYING"];

#[cfg(not(docsrs))]
static CACHE_DATA: &[u8] = include_bytes!(concat!(
    env!("OUT_DIR"),
    "/resources/licenses-cache.bin.zstd"
));
const MIN_THRESHOLD: f32 = 0.8;

/// An owned/no-lifetimes transcription of `Vec<&spdx::expression::ExpressionReq>`
#[derive(Debug, Clone)]
pub struct EvaluationError {
    // The original expression that the ranges of the expressions reffer to
    pub expression: String,
    /// The list of expressions that failed
    pub failed: Vec<spdx::expression::ExpressionReq>,
}

impl From<(String, Vec<&spdx::expression::ExpressionReq>)> for EvaluationError {
    fn from((expression, failures): (String, Vec<&spdx::expression::ExpressionReq>)) -> Self {
        Self {
            expression,
            failed: failures.iter().map(|req| req.to_owned().clone()).collect(),
        }
    }
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "evaluation failure(s) in SPDX expression \"{}\": [",
            self.expression
        ))?;
        for req in &self.failed {
            let expr_part = &self.expression.get((req.span.start as usize)..(req.span.end as usize))
                .expect("Looks like spdx::expression::ExpressionReq did something wrong, supplying us with an invalid string span for a license expression.");
            f.write_fmt(format_args!(
                "{{ '{}' - @({},{}) - \"{expr_part}\" }}, ",
                req.req, req.span.start, req.span.end
            ))?;
        }
        f.write_str("]")?;
        Ok(())
    }
}

impl std::error::Error for EvaluationError {}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("There is no license specified.")]
    NoLicense,

    #[error("The license expression is not in a valid SPDX format; see <>.")]
    ParsingFailed(#[from] spdx::ParseError),

    #[error("The license specifier is valid, but the licensing scheme is not approved.")]
    NotApproved(#[from] EvaluationError),
}

pub fn validate_spdx_expr(expr: &str) -> Result<(), Error> {
    if expr.is_empty() {
        return Err(Error::NoLicense);
    }
    let spdx_expr = spdx::Expression::parse(expr)?;
    spdx_expr
        // .evaluate_with_failures(|req| {
        .evaluate_with_failures(|req| {
            if let spdx::LicenseItem::Spdx { id, .. } = req.license {
                return id.is_osi_approved();
            }
            false
        })
        .map_err(|failures| EvaluationError::from((expr.to_owned(), failures)))?;
    Ok(())
}

pub fn get_licenses(dir: &str) -> Result<Vec<String>, std::io::Error> {
    static DIR_LICENSES_EXTRACTOR: LazyLock<Detector> = LazyLock::new(Detector::new);

    log::trace!("Fetching licenses from (REUSE-dir) '{}' OUTSIDE ...", dir);
    DIR_LICENSES_EXTRACTOR.get_licenses(dir)
}

/// A basic wrapper around the askalono library;
/// originally from here:
/// <https://github.com/o2sh/onefetch/blob/main/src/info/license.rs>
/// (MIT licensed)
struct Detector {
    store: Store,
}

impl Detector {
    #[cfg(not(docsrs))]
    pub fn new() -> Self {
        match Store::from_cache(CACHE_DATA) {
            Ok(store) => Self { store },
            Err(err) => {
                log::error!("Failed to load licenses info cache: {err}");
                panic!("Failed to load licenses info cache: {err}");
            }
        }
    }
    #[cfg(docsrs)]
    pub fn new() -> Self {
        // This will never be called; if it will anyway,
        // the dev of this code did something wrong
        log::error!("No licenses cache available if `cfg(docsrs)` is set");
        panic!("No licenses cache available if `cfg(docsrs)` is set");
    }

    /// Returns a list of SPDX license identifiers;
    /// one for each LICENSE file found in the given directory.
    pub fn get_licenses(&self, dir: &str) -> Result<Vec<String>, std::io::Error> {
        fn is_license_file<S: AsRef<str>>(file_name: S) -> bool {
            LICENSE_FILE_PREFIXES
                .iter()
                .any(|&lf_prefix| file_name.as_ref().starts_with(lf_prefix))
        }

        log::trace!("Looking for license files in '{}' ...", dir);
        let mut output = fs::read_dir(dir)?
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|entry| {
                entry.is_file()
                    && entry
                        .file_name()
                        .map(OsStr::to_string_lossy)
                        .is_some_and(is_license_file)
            })
            .filter_map(|entry| {
                let contents = fs::read_to_string(entry.as_path()).unwrap_or_default(); // TODO Not too clean; we should possibly fail the function instead of silently skipping the file on error
                let evaluated_license_opt = self.analyze(&contents);
                if let Some(evaluated_license) = &evaluated_license_opt {
                    log::trace!(
                        "Found (non-REUSE) license {evaluated_license} in file {}.",
                        entry.display()
                    );
                }
                evaluated_license_opt
            })
            .collect::<Vec<_>>();

        output.sort();
        output.dedup();
        log::trace!("Fetching licenses - found {}.", output.len());
        Ok(output)
    }

    fn analyze(&self, text: &str) -> Option<String> {
        let matched = self.store.analyze(&TextData::from(text));

        if matched.score >= MIN_THRESHOLD {
            Some(matched.name.into())
        } else {
            None
        }
    }
}
