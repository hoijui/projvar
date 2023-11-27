// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::io;

use crate::settings::Verbosity;
use projvar::BoxResult;
use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    fmt,
    prelude::*,
    reload::{self, Handle},
    Registry,
};

const fn verbosity_to_level(verbosity: Verbosity) -> LevelFilter {
    match verbosity {
        Verbosity::None => LevelFilter::OFF,
        Verbosity::Errors => LevelFilter::ERROR,
        Verbosity::Warnings => LevelFilter::WARN,
        Verbosity::Info => LevelFilter::INFO,
        Verbosity::Debug => LevelFilter::DEBUG,
        Verbosity::Trace => LevelFilter::TRACE,
    }
}

/// Sets up logging, with a way to change the log level later on,
/// and with all output going to stderr,
/// as suggested by <https://clig.dev/>.
///
/// # Errors
///
/// If initializing the registry (logger) failed.
pub fn setup_logging() -> BoxResult<Handle<LevelFilter, Registry>> {
    // NOTE It is crucial to first set the lowest log level,
    //      as apparently, any level that is lower then this one
    //      will be ignored when trying to set it later on.
    //      Later though, the level can be changed up and down as desired.
    let level_filter = LevelFilter::TRACE;
    let (filter, reload_handle_filter) = reload::Layer::new(level_filter);

    let l_stderr = fmt::layer().map_writer(move |_| io::stderr);

    let registry = tracing_subscriber::registry().with(filter).with(l_stderr);
    registry.try_init()?;

    Ok(reload_handle_filter)
}

pub fn set_log_level(
    reload_handle: &Handle<LevelFilter, Registry>,
    verbosity: Verbosity,
) -> BoxResult<()> {
    let level_filter = verbosity_to_level(verbosity);
    reload_handle.modify(|filter| *filter = level_filter)?;
    Ok(())
}
