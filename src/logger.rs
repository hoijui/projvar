// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

extern crate simplelog;
// extern crate flexi_logger;

use std::fs::File;
use std::path::Path;

use crate::settings::Verbosity;
use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, SharedLogger, TermLogger, TerminalMode,
    WriteLogger,
};
// use flexi_logger::{Duplicate, Logger};

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
    let mut loggers: Vec<Box<(dyn SharedLogger + 'static)>> = vec![TermLogger::new(
        // LevelFilter::Info,
        verbosity_to_level(verbosity.0),
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )];
    if let Some(file_path) = file {
        loggers.push(WriteLogger::new(
            verbosity_to_level(verbosity.1),
            Config::default(),
            File::create(file_path).unwrap(),
        ));
    };
    CombinedLogger::init(loggers).unwrap();
    log::debug!("Logging activated.");
    if let Some(file_path) = file {
        log::info!("Logging to file '{:?}'.", file_path);
    }
}

// pub fn init2(file: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {

//     Logger::try_with_env_or_str("info")?
//         .log_to_stdout()
//         // .log_to_stderr()
//         // .duplicate_to_stderr(Duplicate::Warn)
//         .duplicate_to_stderr(Duplicate::Info)
//         .start()?;
//     Ok(())
// }
