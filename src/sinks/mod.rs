// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod env;
pub mod file;

// use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use crate::environment::Environment;
use crate::var::{Key, Variable};

type BoxResult<T> = Result<T, Box<dyn Error>>;

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
        values: &[(Key, &Variable, &String)],
        // values: Iter<'a, (&Key, &Variable, String)>,
    ) -> BoxResult<()>;
}
