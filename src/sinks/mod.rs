// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod env;
pub mod file;

use std::fmt;

use crate::BoxResult;
use crate::environment::Environment;
use crate::var::{Confidence, Key, Variable};

pub trait VarSink: fmt::Display {
    /// Indicates whether this sink of variables is usable.
    /// It might not be usable if the underlying data-sink (e.g. a file) can not be written to,
    /// or is not reachable (e.g. a DB on the network).
    fn is_usable(&self, environment: &Environment) -> bool;

    /// Tries to store a list of variable `values`.
    ///
    /// # Errors
    ///
    /// If the underlying data-sink (e.g. a file) can not be written to,
    /// or is not reachable (e.g. a DB on the network).
    /// or innumerable other kinds of problems,
    /// depending on the kind of the sink.
    fn store(
        &self,
        environment: &Environment,
        values: &[(Key, &Variable, &(Confidence, String))],
        // values: Box<dyn Iterator<Item = (Key, &Variable, &(Confidence, String))>>,
    ) -> BoxResult<()>;
}
