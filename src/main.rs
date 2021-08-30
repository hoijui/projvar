// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;

use crate::environment::Environment;
use crate::settings::Settings;
use clap::{crate_authors, crate_version, App, AppSettings, Arg, ValueHint};
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
// use strum::IntoEnumIterator;
use std::str::FromStr;
use strum::VariantNames;

mod environment;
mod git;
mod logger;
mod settings;
mod storage;
mod var;
mod vars_preparator;

fn arg_matcher() -> App<'static> {
    App::new("Project Variables")
        .about("Ensures that certain specific, project and build related environment variables are set.")
        .version(crate_version!())
        .author(crate_authors!())
        .setting(AppSettings::ColoredHelp)
        // .setting(AppSettings::UnifiedHelpMessage)
        /* .arg( */
        /*     Arg::new("input") */
        /*         .about("the input file to use; '-' for stdin") */
        /*         .takes_value(true) */
        /*         .short('i') */
        /*         .long("input") */
        /*         .multiple_occurrences(false) */
        /*         .default_value("-") */
        /*         .required(true) */
        /* ) */
        .arg(
            Arg::new("output")
                .about("instead of setting environment variables, write values to the specified file (using BASH syntax)")
                .takes_value(true)
                .forbid_empty_values(true)
                .value_hint(ValueHint::FilePath)
                .short('o')
                .long("output")
                .multiple_occurrences(false)
                .required(false)
        )
        .arg(
            Arg::new("variable")
                .about("a variable key-value pair to be used for substitution in the text")
                .takes_value(true)
                .forbid_empty_values(true)
                .value_name("key=value")
                .value_hint(ValueHint::Other)
                .short('D')
                .long("variable")
                .multiple_occurrences(true)
                .required(false)
        )
        // .arg(
        //     Arg::new("environment")
        //         .about("use environment variables for substitution in the text")
        //         .takes_value(false)
        //         .short('e')
        //         .long("env")
        //         .multiple_occurrences(false)
        //         .required(false)
        // )
        .arg(
            Arg::new("verbose")
                .about("more verbose output (useful for debugging)")
                .takes_value(false)
                .short('v')
                .long("verbose")
                .multiple_occurrences(false)
                .required(false)
        )
        .arg(
            Arg::new("fail-on-missing-values")
                .about("fail if no value is available for a variable key found in the input text")
                .takes_value(false)
                .short('f')
                .long("fail-on-missing-values")
                .multiple_occurrences(false)
                .required(false)
        )
        .arg(
            Arg::new("set-all")
                .about("Set all associated keys of all variables (e.g. GITHUB_REF, CI_COMMIT_BRANCH, ...), not just the primary one for each (e.g. BUILD_BRANCH).")
                .takes_value(false)
                .short('a')
                .long("set-all")
                .multiple_occurrences(false)
                .required(false)
        )
        .arg(
            Arg::new("dry")
                .about("Set Whether to skip the actual setting of environment variables.")
                .takes_value(false)
                .short('d')
                .long("dry")
                .multiple_occurrences(false)
                .required(false)
        )
        .arg(
            Arg::new("overwrite")
                .about("TODO.")
                .takes_value(true)
                .possible_values(settings::Overwrite::VARIANTS)//iter().map(|ovr| &*format!("{:?}", ovr)).collect())
                // .short('O')
                .long("overwrite")
                .multiple_occurrences(false)
                .default_value(settings::Overwrite::All.into())
                .required(false)
        )
        .arg(
            Arg::new("list")
                .about("Prints a list of all the environment variables that are potentially set by this tool onto stdout and exits.")
                .takes_value(false)
                .short('l')
                .long("list")
                .multiple_occurrences(false)
                .required(false)
        )
        .arg(
            Arg::new("log-file")
                .about("Writes a detailed log to the specifed file.")
                .takes_value(true)
                .forbid_empty_values(true)
                .short('L')
                .long("log-file")
                .multiple_occurrences(false)
                .required(false)
                .default_missing_value("projvar.log.txt")
        )
        .arg(
            Arg::new("date-format")
                .about("Date format string for generated (vs supplied) dates. For details, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html")
                .takes_value(true)
                .forbid_empty_values(true)
                .value_hint(ValueHint::Other)
                .short('F')
                .long("date-format")
                .multiple_occurrences(false)
                .default_value(git::DATE_FORMAT)
                .required(false)
        )
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = arg_matcher().get_matches();

    let verbose: bool = args.is_present("verbose");

    let log_file = args.value_of("log-file").map(Path::new);
    logger::init(log_file);
    // logger::init2(log_file)?;
    log::debug!("Logging activated."); // HACK

    if args.is_present("list") {
        var::list(verbose);
        return Ok(());
    }

    let fail_on_missing: bool = args.is_present("fail-on-missing-values");
    let set_all: bool = args.is_present("set-all");

    // if verbose {
    //     println!();
    //     if let Some(in_file) = args.value_of("input") {
    //         println!("INPUT: {}", &in_file);
    //     }
    //     if let Some(out_file) = args.value_of("output") {
    //         println!("OUTPUT: {}", &out_file);
    //     }

    //     for (key, value) in &vars {
    //         println!("VARIABLE: {}={}", key, value);
    //     }
    //     println!();
    // }

    let repo_path: Option<&'static str> = Some(".");
    let repo_path_str = repo_path.unwrap_or(".");
    let repo_path = Path::new(repo_path_str);

    let date_format = match args.value_of("date-format") {
        Some(date_format) => date_format,
        None => git::DATE_FORMAT,
    };

    let storage = if args.is_present("dry") {
        settings::StorageMode::Dry
    } else if args.is_present("output") {
        settings::StorageMode::ToFile(
            args.value_of("output")
                .map(|s| Path::new(s).to_owned())
                .unwrap(),
        )
    } else {
        settings::StorageMode::Environment
    };

    let overwrite = settings::Overwrite::from_str(args.value_of("overwrite").unwrap())?;

    let settings = Settings {
        repo_path: Some(repo_path),
        date_format: date_format.to_owned(),
        to_set: settings::VarsToSet::from(set_all),
        overwrite,
        fail_on: settings::FailOn::from(fail_on_missing),
        storage,
        verbosity: settings::Verbosity::from(verbose),
    };
    let mut environment = Environment::new(&settings);

    // // enlist environment variables
    // if args.is_present("environment") {
    //     repvar::tools::append_env(&mut vars);
    // }

    // // enlist variables provided on the CLI
    // if args.occurrences_of("variable") > 0 {
    //     for kvp in args
    //         .values_of_t::<repvar::key_value::Pair>("variable")
    //         .unwrap_or_else(|e| e.exit())
    //     {
    //         vars.insert(kvp.key, kvp.value);
    //     }
    // }

    vars_preparator::prepare_project_vars(&mut environment)
}
