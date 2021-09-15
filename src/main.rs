// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// #[macro_use]
extern crate clap;
// #[macro_use]
extern crate enum_map;
extern crate log;

// use clap::lazy_static::lazy_static;
use clap::{
    crate_authors, crate_description, crate_license, crate_name, crate_version, App, AppSettings,
    Arg, ArgMatches, ValueHint,
};
use std::collections::HashSet;
// use enumset::EnumSet;
use std::convert::{TryFrom, TryInto};
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum::VariantNames;

mod environment;
mod logger;
pub mod settings;
pub mod sinks;
pub mod sources;
pub mod tools;
mod var;
mod vars_preparator;

use crate::environment::Environment;
use crate::settings::{Settings, Verbosity};
use crate::sinks::VarSink;
use crate::sources::VarSource;
use crate::var::Key;

type BoxResult<T> = Result<T, Box<dyn Error>>;

fn is_git_repo_root(repo_path: Option<&Path>) -> bool {
    tools::git::Repo::try_from(repo_path).is_ok()
}

fn arg_project_root() -> Arg<'static> {
    Arg::new("variable")
        .about("The root of the project, mainly used for SCM (e.g. git) information gathering.")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_name("DIR")
        .value_hint(ValueHint::DirPath)
        .short('C')
        .long("project-root")
        .multiple_occurrences(false)
        .required(false)
        .default_value(".")
}

fn arg_variable() -> Arg<'static> {
    Arg::new("variable")
        .about("A variable key-value pair to be used as input")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_name("KEY=VALUE")
        .value_hint(ValueHint::Other)
        .validator(var::is_key_value_str_valid)
        .short('D')
        .long("variable")
        .multiple_occurrences(true)
        .required(false)
}

fn arg_variables_file() -> Arg<'static> {
    Arg::new("variables-file")
        .about("A file containing KEY=VALUE pairs, one per line (BASH style). Empty lines, and those startign wiht \"#\" or \"//\" are ignored.")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_name("FILE")
        .value_hint(ValueHint::FilePath)
        .short('I')
        .long("variables-file")
        .multiple_occurrences(true)
        .required(false)
        .default_missing_value("-")
}

fn arg_no_env_in() -> Arg<'static> {
    Arg::new("no-env-in")
        .about("Disable the use of environment variables as input")
        .takes_value(false)
        .short('x')
        .long("no-env-in")
        .multiple_occurrences(false)
        .required(false)
}

fn arg_env_out() -> Arg<'static> {
    Arg::new("environment-out")
        .about("Write resulting values directy into the environment") // TODO Check: is that even possible? As in, the values remaining in the environment after the ned of the process?
        .takes_value(false)
        .short('e')
        .long("env-out")
        .multiple_occurrences(false)
        .required(false)
}

fn arg_out_file() -> Arg<'static> {
    Arg::new("out-file")
        .about("Writes the variables out thi this file, one KEY-VALUE pair per line (BASH style).")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_name("FILE")
        .value_hint(ValueHint::FilePath)
        .short('F')
        .long("out-file")
        .multiple_occurrences(true)
        .required(false)
}

fn arg_verbose() -> Arg<'static> {
    Arg::new("verbose")
        .about("More verbose output (useful for debugging)")
        .takes_value(false)
        .short('v')
        .long("verbose")
        .multiple_occurrences(true)
        .required(false)
}

fn arg_log_level() -> Arg<'static> {
    Arg::new("log-level")
        .about("Set the log-level to use")
        .takes_value(false)
        .possible_values(settings::Verbosity::VARIANTS)
        .short('v')
        .long("verbose")
        .multiple_occurrences(true)
        .required(false)
        .conflicts_with("verbose")
}

fn arg_quiet() -> Arg<'static> {
    Arg::new("quiet")
        .about("Supresses all log-output to stdout, and only shows errors on stderr (see --log-level to also disable those). This does not affect the log level for the log-file.")
        .takes_value(false)
        .short('q')
        .long("quiet")
        .multiple_occurrences(true)
        .required(false)
        .conflicts_with("verbose")
}

fn arg_fail() -> Arg<'static> {
    Arg::new("fail-on-missing-values")
        .about("Fail if no value is available for any of the required properties (see --all,--require,--require-not)")
        .takes_value(false)
        .short('f')
        .long("fail")
        .multiple_occurrences(false)
        .required(false)
}

fn arg_require_all() -> Arg<'static> {
    Arg::new("require-all")
        .about("Fail if any property failed to resovle to a value (see --fail,--require,--require-not)")
        .takes_value(false)
        .about("Write evaluated values into a file (using BASH syntax). Note: \"-\" has no special meaning here; it does not mean stdout, but rather the file \"./-\".")
        .long("all")
        .multiple_occurrences(false)
        .required(false)
        .requires("fail-on-missing-values")
        .conflicts_with("require")
}

fn arg_require() -> Arg<'static> {
    Arg::new("require")
        .about("A key of a variable whose value is required. For example PROJECT_NAME (see --list for all possible keys). If at least one such option is present, the default required values list is cleared (see --fail,--all,--require-not)")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_name("KEY")
        .value_hint(ValueHint::Other)
        .short('R')
        .long("require")
        .multiple_occurrences(true)
        .required(false)
        .requires("fail-on-missing-values")
        .conflicts_with("require-not")
        .conflicts_with("require-all")
}

fn arg_require_not() -> Arg<'static> {
    Arg::new("require-not")
        .about("A key of a variable whose value is *not* required. For example PROJECT_NAME (see --list for all possible keys). Can be used either on the base of the default requried list or all (see --fail,--all,--require)")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_name("KEY")
        .value_hint(ValueHint::Other)
        .short('N')
        .long("require-not")
        .multiple_occurrences(true)
        .required(false)
        .requires("fail-on-missing-values")
        .conflicts_with("require")
}

// fn arg_set_all() -> Arg<'static> {
//     Arg::new("set-all")
//         .about("Set all associated keys of all variables (e.g. GITHUB_REF, CI_COMMIT_BRANCH, ...), not just the primary one for each (e.g. BUILD_BRANCH).")
//         .takes_value(false)
//         .short('a')
//         .long("set-all")
//         .multiple_occurrences(false)
//         .required(false)
// }

fn arg_dry() -> Arg<'static> {
    Arg::new("dry")
        .about("Set Whether to skip the actual setting of environment variables.")
        .takes_value(false)
        .short('d')
        .long("dry")
        .multiple_occurrences(false)
        .required(false)
}

fn arg_overwrite() -> Arg<'static> {
    Arg::new("overwrite")
        .about("TODO.") // TODO
        .takes_value(true)
        .possible_values(settings::Overwrite::VARIANTS) //iter().map(|ovr| &*format!("{:?}", ovr)).collect())
        // .short('O')
        .long("overwrite")
        .multiple_occurrences(false)
        .default_value(settings::Overwrite::All.into())
        .required(false)
        .conflicts_with("dry")
}

fn arg_list() -> Arg<'static> {
    Arg::new("list")
        .about("Prints a list of all the environment variables that are potentially set by this tool onto stdout and exits.")
        .takes_value(false)
        .short('l')
        .long("list")
        .multiple_occurrences(false)
        .required(false)
}

fn arg_log_file() -> Arg<'static> {
    Arg::new("log-file")
        .about("Writes a detailed log to the specifed file.")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_hint(ValueHint::FilePath)
        .short('L')
        .long("log-file")
        .multiple_occurrences(false)
        .required(false)
        .default_missing_value("projvar.log.txt")
}

fn arg_date_format() -> Arg<'static> {
    Arg::new("date-format")
        .about("Date format string for generated (vs supplied) dates. For details, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_hint(ValueHint::Other)
        .short('T')
        .long("date-format")
        .multiple_occurrences(false)
        .default_value(tools::git::DATE_FORMAT)
        .required(false)
}

fn arg_matcher() -> App<'static> {
    // App::new("Project Variables")
    App::new(crate_name!())
        // .about("Ensures that certain specific, project and build related environment variables are set.")
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .license(crate_license!())
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::UnifiedHelpMessage)
        .arg(arg_project_root())
        .arg(arg_variable())
        .arg(arg_variables_file())
        .arg(arg_no_env_in())
        .arg(arg_env_out())
        .arg(arg_out_file())
        .arg(arg_verbose())
        .arg(arg_log_level())
        .arg(arg_quiet())
        .arg(arg_fail())
        .arg(arg_require_all())
        .arg(arg_require())
        .arg(arg_require_not())
        // .arg(arg_set_all())
        .arg(arg_dry())
        .arg(arg_overwrite())
        .arg(arg_list())
        .arg(arg_log_file())
        .arg(arg_date_format())
}

/// Returns the logging verbosities to be used.
/// The first one is for stdout&stderr,
/// the second one for log-file(s).
fn verbosity(args: &ArgMatches) -> BoxResult<(Verbosity, Verbosity)> {
    let common = if let Some(specified) = args.value_of("log-level") {
        Verbosity::from_str(specified)?
    } else {
        // Set the default base level
        let level = if cfg!(debug_assertions) {
            Verbosity::Debug
        } else {
            Verbosity::Info
        };
        let num_verbose = args.occurrences_of("verbose").try_into()?;
        level.up_max(num_verbose)
    };

    let std = if args.is_present("quiet") {
        Verbosity::None
    } else {
        common
    };

    Ok((std, common))
}

fn repo_path<'a>(args: &'a ArgMatches) -> &'a Path {
    let repo_path: Option<&'static str> = args.value_of("project-root");
    let repo_path_str = repo_path.unwrap_or(".");
    let repo_path = Path::new(repo_path_str);
    log::debug!("Using repo path '{:?}'.", repo_path);
    repo_path
}

fn date_format(args: &ArgMatches) -> &str {
    let date_format = match args.value_of("date-format") {
        Some(date_format) => date_format,
        None => tools::git::DATE_FORMAT,
    };
    log::debug!("Using date format '{}'.", date_format);
    date_format
}

fn storage_mode(args: &ArgMatches) -> settings::StorageMode {
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
    storage
}

fn sources(_args: &ArgMatches, repo_path: &Path) -> Vec<Box<dyn VarSource>> {
    let mut sources: Vec<Box<dyn VarSource>> = vec![];
    if is_git_repo_root(Some(repo_path)) {
        sources.push(Box::new(sources::git::VarSource {}));
    }
    sources.push(Box::new(sources::bitbucket_ci::VarSource {}));
    sources.push(Box::new(sources::github_ci::VarSource {}));
    sources.push(Box::new(sources::gitlab_ci::VarSource {}));
    sources.push(Box::new(sources::jenkins_ci::VarSource {}));
    sources.push(Box::new(sources::travis_ci::VarSource {}));
    for source in &sources {
        log::trace!("Registered source {}.", source);
    }
    sources
}

fn sinks(args: &ArgMatches) -> BoxResult<Vec<Box<dyn VarSink>>> {
    let mut sinks: Vec<Box<dyn VarSink>> = vec![];
    if args.is_present("environment-out") {
        sinks.push(Box::new(sinks::env::VarSink {}));
    }
    if let Some(out_files) = args.values_of("out-file") {
        for out_file in out_files {
            sinks.push(Box::new(sinks::file::VarSink {
                file: PathBuf::from_str(out_file)?,
            }));
        }
    }
    if args.is_present("dry") {
        sinks.clear();
    }
    for sink in &sinks {
        log::trace!("Registered sink {}.", sink);
    }
    Ok(sinks)
}

fn required_keys(args: &ArgMatches) -> BoxResult<HashSet<Key>> {
    let require_all: bool = args.is_present("require-all");
    let mut required_keys = if require_all {
        // EnumSet::<Key>::all()
        // HashSet::<Key>::allj()
        let mut all = HashSet::<Key>::new();
        all.extend(Key::iter());
        all
    } else {
        var::default_keys().clone()
    };
    if let Some(requires) = args.values_of("require") {
        for require in requires {
            let key = Key::from_str(require)?;
            required_keys.insert(key);
        }
    }
    if let Some(require_nots) = args.values_of("require") {
        for require_not in require_nots {
            let key = Key::from_str(require_not)?;
            required_keys.remove(&key);
        }
    }
    // make imutable
    let required_keys = required_keys;
    for required_key in &required_keys {
        log::trace!("Registered required key {:?}.", required_key);
    }

    Ok(required_keys)
}

fn main() -> BoxResult<()> {
    let args = arg_matcher().get_matches();

    let verbosity = verbosity(&args)?;

    let log_file = args.value_of("log-file").map(Path::new);
    logger::init(log_file, verbosity);
    // logger::init2(log_file)?;

    if args.is_present("list") {
        var::list_keys(verbosity.1 >= Verbosity::Info);
        return Ok(());
    }

    // let set_all: bool = args.is_present("set-all");

    let repo_path = repo_path(&args);
    let date_format = date_format(&args);
    let storage = storage_mode(&args);

    let overwrite = settings::Overwrite::from_str(args.value_of("overwrite").unwrap())?;
    log::debug!("Overwriting output variable values? -> {:?}", overwrite);

    let sources = sources(&args, repo_path);

    let sinks = sinks(&args)?;

    let fail_on_missing: bool = args.is_present("fail-on-missing-values");
    let required_keys = required_keys(&args)?;

    let settings = Settings {
        repo_path: Some(repo_path),
        date_format: date_format.to_owned(),
        // to_set: settings::VarsToSet::from(set_all),
        to_set: settings::VarsToSet::Primary,
        overwrite,
        fail_on: settings::FailOn::from(fail_on_missing),
        storage,
        verbosity,
    };
    log::trace!("Created Settings.");
    let mut environment = Environment::new(&settings /*, sources, sinks*/);
    log::trace!("Created Environment.");

    // fetch environment variables
    if !args.is_present("no-env-in") {
        log::trace!("Fetching variables from the environment ...");
        repvar::tools::append_env(&mut environment.vars);
    }
    // fetch variable files
    if let Some(var_files) = args.values_of("variables-file") {
        for var_file in var_files {
            if var_file == "-" {
                log::trace!("Fetching variables from stdin ...");
            } else {
                log::trace!("Fetching variables from file '{}' ...", var_file);
            }
            let mut reader = repvar::tools::create_input_reader(Some(var_file))?;
            environment
                .vars
                .extend(var::parse_vars_file_reader(&mut reader)?);
        }
    }
    // insert CLI supplied variables values
    if let Some(variables) = args.values_of("variable") {
        for var in variables {
            log::trace!("Adding variables from CLI '{}' ...", var);
            let (key, value) = var::parse_key_value_str(var)?;
            environment.vars.insert(key.to_owned(), value.to_owned());
        }
    }

    // // enlist variables provided on the CLI
    // if args.occurrences_of("variable") > 0 {
    //     for kvp in args
    //         .values_of_t::<repvar::key_value::Pair>("variable")
    //         .unwrap_or_else(|e| e.exit())
    //     {
    //         vars.insert(kvp.key, kvp.value);
    //     }
    // }

    vars_preparator::prepare_project_vars(&mut environment, sources, sinks)
    // Ok(())
}
