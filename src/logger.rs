// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

extern crate simplelog;
// extern crate flexi_logger;

use std::fs::File;
use std::path::Path;

use simplelog::{
    ColorChoice, CombinedLogger, Config, LevelFilter, SharedLogger, TermLogger, TerminalMode,
    WriteLogger,
};
// use flexi_logger::{Duplicate, Logger};

pub fn init(file: Option<&Path>) {
    let mut loggers: Vec<Box<(dyn SharedLogger + 'static)>> = vec![TermLogger::new(
        // LevelFilter::Info,
        LevelFilter::Trace,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )];
    if let Some(file_path) = file {
        loggers.push(WriteLogger::new(
            LevelFilter::Trace,
            Config::default(),
            File::create(file_path).unwrap(),
        ));
    };
    CombinedLogger::init(loggers).unwrap();
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
