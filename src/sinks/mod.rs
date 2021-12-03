// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod env;
pub mod file;

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use crate::environment::Environment;
use crate::{storage, BoxResult};

pub const DEFAULT_FILE_OUT: &str = ".projvars.env.txt";

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
        values: &[storage::Value],
        // values: Box<dyn Iterator<Item = (Key, &Variable, &(Confidence, String))>>,
    ) -> BoxResult<()>;
}

/// Returns a list of sinks commonly used when using this crate as CLI tool
///
/// # Panics
///
/// if [`DEFAULT_FILE_OUT`] fails to be parsed as a valid file-system path
#[must_use]
pub fn cli_list(
    env_out: bool,
    dry: bool,
    default_out_file: bool,
    additional_out_files: Vec<PathBuf>,
) -> Vec<Box<dyn VarSink>> {
    let mut sinks: Vec<Box<dyn VarSink>> = vec![];
    if env_out {
        sinks.push(Box::new(env::VarSink {}));
    }
    if default_out_file {
        log::info!("Using the default out file: {}", DEFAULT_FILE_OUT);
        sinks.push(Box::new(file::VarSink {
            file: PathBuf::from_str(DEFAULT_FILE_OUT).unwrap(),
        }));
    }
    for out_file in additional_out_files {
        sinks.push(Box::new(file::VarSink { file: out_file }));
    }
    if dry {
        sinks.clear();
    } else if sinks.is_empty() {
        log::warn!("No sinks registered! The results of this run will not be stored anywhere.");
    }
    for sink in &sinks {
        log::trace!("Registered sink {}.", sink);
    }
    sinks
}
