// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;

/// An owned/no-lifetimes transcription of [`spdx::error::ParseError`].
#[derive(thiserror::Error, Debug)]
pub struct SpdxParseError {
    /// The string that was attempting to be parsed
    pub original: String,
    /// The range of characters in the original string that result
    /// in this error
    pub span: std::ops::Range<usize>,
    // /// The specific reason for the error
    // pub reason: spdx::error::Reason, // TODO Can't use this, as Reason does not impl Clone
}

impl From<spdx::error::ParseError<'_>> for SpdxParseError {
    fn from(err: spdx::error::ParseError<'_>) -> Self {
        SpdxParseError {
            original: err.original.to_owned(),
            span: err.span,
            // reason: err.reason,
        }
    }
}

impl<'a> From<&'a SpdxParseError> for spdx::error::ParseError<'a> {
    fn from(err: &'a SpdxParseError) -> Self {
        spdx::error::ParseError {
            original: &err.original,
            span: err.span.clone(),
            reason: spdx::error::Reason::Empty,
        }
    }
}

impl fmt::Display for SpdxParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err: spdx::error::ParseError = self.into();
        err.fmt(f)
    }
}

/// An owned/no-lifetimes transcription of `Vec<&spdx::expression::ExpressionReq>`
#[derive(thiserror::Error, Debug)]
pub struct ApprovementFailure {
    /// The list of expresions that were not approved
    pub failed: Vec<spdx::expression::ExpressionReq>,
}

impl From<Vec<&spdx::expression::ExpressionReq>> for ApprovementFailure {
    fn from(err: Vec<&spdx::expression::ExpressionReq>) -> Self {
        Self {
            failed: err.iter().map(|req| req.to_owned().clone()).collect(),
        }
    }
}

impl fmt::Display for ApprovementFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:?}", self.failed))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("There is no license specified.")]
    NoLicense,

    #[error("The license expression is not in a valid SPDX format; see <>.")]
    ParsingFailed(#[from] SpdxParseError),

    #[error("The license specifier is valid, but the licensing scheme is not approved.")]
    NotApproved(#[from] ApprovementFailure),
}

pub fn validate_spdx_expr(expr: &str) -> Result<(), Error> {
    if expr.is_empty() {
        return Err(Error::NoLicense);
    }
    let spdx_expr = spdx::Expression::parse(expr).map_err(SpdxParseError::from)?;
    spdx_expr
        .evaluate_with_failures(|req| {
            if let spdx::LicenseItem::Spdx { id, .. } = req.license {
                return id.is_osi_approved();
            }
            false
        })
        .map_err(ApprovementFailure::from)?;
    Ok(())
}
