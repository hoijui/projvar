// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

extern crate simplelog;

use std::fs::File;
use std::path::Path;

use crate::settings::Verbosity;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, SharedLogger, TermLogger,
    TerminalMode, WriteLogger,
};

fn verbosity_to_level(verbosity: Verbosity) -> LevelFilter {
    match verbosity {
        Verbosity::None => LevelFilter::Off,
        Verbosity::Errors => LevelFilter::Error,
        Verbosity::Warnings => LevelFilter::Warn,
        Verbosity::Info => LevelFilter::Info,
        Verbosity::Debug => LevelFilter::Debug,
        Verbosity::Trace => LevelFilter::Trace,
    }
}

pub fn init(file: Option<&Path>, verbosity: (Verbosity, Verbosity)) {
    // only show the log-level (no time, no source-code location, ...)
    let config = ConfigBuilder::new()
        .set_max_level(LevelFilter::Error)
        .set_time_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_target_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .build();
    let mut loggers: Vec<Box<(dyn SharedLogger + 'static)>> = vec![TermLogger::new(
        // LevelFilter::Info,
        verbosity_to_level(verbosity.0),
        config.clone(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )];
    if let Some(file_path) = file {
        loggers.push(WriteLogger::new(
            verbosity_to_level(verbosity.1),
            config,
            File::create(file_path).unwrap(),
        ));
    };
    CombinedLogger::init(loggers).unwrap();
    log::debug!("Logging activated.");
    if let Some(file_path) = file {
        log::info!("Logging to file '{:?}'.", file_path);
    }
}
