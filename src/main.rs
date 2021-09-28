// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

extern crate clap;
extern crate enum_map;
extern crate log;

use clap::lazy_static::lazy_static;
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

const A_S_PROJECT_ROOT: char = 'C';
const A_L_PROJECT_ROOT: &str = "project-root";
const A_S_VARIABLE: char = 'D';
const A_L_VARIABLE: &str = "variable";
const A_S_VARIABLES_FILE: char = 'I';
const A_L_VARIABLES_FILE: &str = "variables-file";
const A_S_NO_ENV_IN: char = 'x';
const A_L_NO_ENV_IN: &str = "no-env-in";
const A_S_ENV_OUT: char = 'e';
const A_L_ENV_OUT: &str = "env-out";
const A_S_FILE_OUT: char = 'f';
const A_L_FILE_OUT: &str = "file-out";
const A_S_VERBOSE: char = 'v';
const A_L_VERBOSE: &str = "verbose";
const A_S_LOG_LEVEL: char = 'F';
const A_L_LOG_LEVEL: &str = "log-level";
const A_S_QUIET: char = 'q';
const A_L_QUIET: &str = "quiet";
const A_S_FAIL_ON_MISSING_VALUE: char = 'f';
const A_L_FAIL_ON_MISSING_VALUE: &str = "fail";
const A_S_REQUIRE_ALL: char = 'a';
const A_L_REQUIRE_ALL: &str = "all";
const A_S_REQUIRE: char = 'R';
const A_L_REQUIRE: &str = "require";
const A_S_REQUIRE_NOT: char = 'N';
const A_L_REQUIRE_NOT: &str = "require-not";
const A_S_DRY: char = 'd';
const A_L_DRY: &str = "dry";
const A_S_OVERWRITE: char = 'o';
const A_L_OVERWRITE: &str = "overwrite";
const A_S_LIST: char = 'l';
const A_L_LIST: &str = "list";
const A_S_LOG_FILE: char = 'L';
const A_L_LOG_FILE: &str = "log-file";
const A_S_DATE_FORMAT: char = 'T';
const A_L_DATE_FORMAT: &str = "date-format";

fn arg_project_root() -> Arg<'static> {
    Arg::new(A_L_PROJECT_ROOT)
        .about("The root dir of the project")
        .long_about("The root directory of the project, mainly used for SCM (e.g. git) information gathering.")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_name("DIR")
        .value_hint(ValueHint::DirPath)
        .short(A_S_PROJECT_ROOT)
        .long(A_L_PROJECT_ROOT)
        .multiple_occurrences(false)
        .required(false)
        .default_value(".")
}

fn arg_variable() -> Arg<'static> {
    Arg::new(A_L_VARIABLE)
        .about("A key-value pair to be used as input")
        .long_about("A key-value pair (aka a variable) to be used as input, as it it was specified as an environment variable. Value provided with this take precedense over environment variables - they overwrite them. See -I,--variable-file for supplying a lot of such pairs at once.")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_name("KEY=VALUE")
        .value_hint(ValueHint::Other)
        .validator(var::is_key_value_str_valid)
        .short(A_S_VARIABLE)
        .long(A_L_VARIABLE)
        .multiple_occurrences(true)
        .required(false)
}

fn arg_variables_file() -> Arg<'static> {
    Arg::new(A_L_VARIABLES_FILE)
        .about("A file containing KEY=VALUE pairs")
        .long_about("A file containing KEY=VALUE pairs, one per line (BASH style). Empty lines, and those startign with \"#\" or \"//\" are ignored. See -D,--variable for specifying one pair at a time.")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_name("FILE")
        .value_hint(ValueHint::FilePath)
        .short(A_S_VARIABLES_FILE)
        .long(A_L_VARIABLES_FILE)
        .multiple_occurrences(true)
        .required(false)
        .default_missing_value("-")
}

fn arg_no_env_in() -> Arg<'static> {
    Arg::new(A_L_NO_ENV_IN)
        .about("Do not read environment variables")
        .long_about("Disable the use of environment variables as input")
        .takes_value(false)
        .short(A_S_NO_ENV_IN)
        .long(A_L_NO_ENV_IN)
        .multiple_occurrences(false)
        .required(false)
}

fn arg_env_out() -> Arg<'static> {
    Arg::new(A_L_ENV_OUT)
        .about("Write resulting values directy into the environment") // TODO Check: is that even possible? As in, the values remaining in the environment after the ned of the process?
        .takes_value(false)
        .short(A_S_ENV_OUT)
        .long(A_L_ENV_OUT)
        .multiple_occurrences(false)
        .required(false)
}

fn arg_out_file() -> Arg<'static> {
    Arg::new(A_L_FILE_OUT)
        .about("Write variables into this file")
        .long_about("Write evaluated values into a file, one KEY-VALUE pair per line (BASH syntax). Note that \"-\" has no special meaning here; it does not mean stdout, but rather the file \"./-\".")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_name("FILE")
        .value_hint(ValueHint::FilePath)
        .short(A_S_FILE_OUT)
        .long(A_L_FILE_OUT)
        .multiple_occurrences(true)
        .required(false)
}

fn arg_verbose() -> Arg<'static> {
    Arg::new(A_L_VERBOSE)
        .about("More verbose log output")
        .long_about("More verbose log output; useful for debugging. See -L,--log-level for more fine-graine control.")
        .takes_value(false)
        .short(A_S_VERBOSE)
        .long(A_L_VERBOSE)
        .multiple_occurrences(true)
        .required(false)
}

fn arg_log_level() -> Arg<'static> {
    Arg::new(A_L_LOG_LEVEL)
        .about("Set the log-level")
        .takes_value(false)
        .possible_values(settings::Verbosity::VARIANTS)
        .short(A_S_LOG_LEVEL)
        .long(A_L_LOG_LEVEL)
        .multiple_occurrences(true)
        .required(false)
        .conflicts_with(A_L_VERBOSE)
}

fn arg_quiet() -> Arg<'static> {
    Arg::new(A_L_QUIET)
        .about("No logging to stdout (only stderr)")
        .long_about("Supresses all log-output to stdout, and only shows errors on stderr (see -L,--log-level to also disable those). This does not affect the log level for the log-file.")
        .takes_value(false)
        .short(A_S_QUIET)
        .long(A_L_QUIET)
        .multiple_occurrences(true)
        .required(false)
        .conflicts_with(A_L_VERBOSE)
}

fn arg_fail() -> Arg<'static> {
    Arg::new(A_L_FAIL_ON_MISSING_VALUE)
        .about("Fail if a required value is missing")
        .long_about("Fail if no value is available for any of the required properties (see --all,--require,--require-not)")
        .takes_value(false)
        .short(A_S_FAIL_ON_MISSING_VALUE)
        .long(A_L_FAIL_ON_MISSING_VALUE)
        .multiple_occurrences(false)
        .required(false)
}

fn arg_require_all() -> Arg<'static> {
    Arg::new(A_L_REQUIRE_ALL)
        .about("Marks all properties as required")
        .long_about("Marks all properties as required. See --fail,--require,--require-not.")
        .takes_value(false)
        .short(A_S_REQUIRE_ALL)
        .long(A_L_REQUIRE_ALL)
        .multiple_occurrences(false)
        .required(false)
        .requires(A_L_FAIL_ON_MISSING_VALUE)
        .conflicts_with(A_L_REQUIRE)
}

fn arg_require() -> Arg<'static> {
    Arg::new(A_L_REQUIRE)
        .about("Mark a propery as required")
        .long_about(r#"Mark a propery as required. You may use the property name (e.g. "Name") or the variable key (e.g. "PROJECT_NAME"); See --list for all possible keys. If at least one such option is present, the default required values list is cleared (see --fail,--all,--require-not)."#)
        .takes_value(true)
        .forbid_empty_values(true)
        .value_name("KEY")
        .value_hint(ValueHint::Other)
        .short(A_S_REQUIRE)
        .long(A_L_REQUIRE)
        .multiple_occurrences(true)
        .required(false)
        .requires(A_L_FAIL_ON_MISSING_VALUE)
        .conflicts_with(A_L_REQUIRE_NOT)
        .conflicts_with(A_L_REQUIRE_ALL)
}

fn arg_require_not() -> Arg<'static> {
    Arg::new(A_L_REQUIRE_NOT)
        .about("Mark a property as not required")
        .long_about("A key of a variable whose value is *not* required. For example PROJECT_NAME (see --list for all possible keys). Can be used either on the base of the default requried list or all (see --fail,--all,--require)")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_name("KEY")
        .value_hint(ValueHint::Other)
        .short(A_S_REQUIRE_NOT)
        .long(A_L_REQUIRE_NOT)
        .multiple_occurrences(true)
        .required(false)
        .requires(A_L_FAIL_ON_MISSING_VALUE)
        .conflicts_with(A_L_REQUIRE)
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
    Arg::new(A_L_DRY)
        .about("Do not write any files or set any environment variables")
        .long_about("Set Whether to skip the actual setting of environment variables.")
        .takes_value(false)
        .short(A_S_DRY)
        .long(A_L_DRY)
        .multiple_occurrences(false)
        .required(false)
}

fn arg_overwrite() -> Arg<'static> {
    Arg::new(A_L_OVERWRITE)
        .about("Whether to overwrite already set values in the output.")
        .takes_value(true)
        .possible_values(settings::Overwrite::VARIANTS) //iter().map(|ovr| &*format!("{:?}", ovr)).collect())
        .short(A_S_OVERWRITE)
        .long(A_L_OVERWRITE)
        .multiple_occurrences(false)
        .default_value(settings::Overwrite::All.into())
        .required(false)
        .conflicts_with(A_L_DRY)
}

fn arg_list() -> Arg<'static> {
    Arg::new(A_L_LIST)
        .about("Show all properties and their keys")
        .long_about("Prints a list of all the environment variables that are potentially set by this tool onto stdout and exits.")
        .takes_value(false)
        .short(A_S_LIST)
        .long(A_L_LIST)
        .multiple_occurrences(false)
        .required(false)
}

fn arg_log_file() -> Arg<'static> {
    lazy_static! {
        static ref LOG_FILE_NAME: String = format!("{}.log.txt", crate_name!());
    }
    Arg::new(A_L_LOG_FILE)
        .about("Write log output to a file")
        .long_about("Writes a detailed log to the specifed file.")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_hint(ValueHint::FilePath)
        .short(A_S_LOG_FILE)
        .long(A_L_LOG_FILE)
        .multiple_occurrences(false)
        .required(false)
        .default_missing_value(&LOG_FILE_NAME)
}

fn arg_date_format() -> Arg<'static> {
    Arg::new(A_L_DATE_FORMAT)
        .about("Date format for generated dates")
        .long_about("Date format string for generated (vs supplied) dates. For details, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html")
        .takes_value(true)
        .forbid_empty_values(true)
        .value_hint(ValueHint::Other)
        .short(A_S_DATE_FORMAT)
        .long(A_L_DATE_FORMAT)
        .multiple_occurrences(false)
        .default_value(tools::git::DATE_FORMAT)
        .required(false)
}

lazy_static! {
    static ref ARGS: [Arg<'static>; 18] =
    [
        arg_project_root(),
        arg_variable(),
        arg_variables_file(),
        arg_no_env_in(),
        arg_env_out(),
        arg_out_file(),
        arg_verbose(),
        arg_log_level(),
        arg_quiet(),
        arg_fail(),
        arg_require_all(),
        arg_require(),
        arg_require_not(),
        // .arg(arg_set_all(),
        arg_dry(),
        arg_overwrite(),
        arg_list(),
        arg_log_file(),
        arg_date_format(),
    ];
}

fn arg_matcher() -> App<'static> {
    // App::new("Project Variables")
    let app = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .license(crate_license!())
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::UnifiedHelpMessage)
        .bin_name("osh")
        .args(ARGS.into_iter());
    // This makes sure that we have no duplicate short- or long-flags,
    // as App would not store duplicates, while the slice does.
    // NOTE: We add 2 for "--help" and "--version",
    // whihc are generate dautomatically.
    assert_eq!(app.get_arguments().count(), ARGS.len() + 2);
    app
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

fn repo_path<'a>(args: &'a ArgMatches) -> PathBuf {
    let repo_path: Option<&str> = args.value_of("project-root");
    let repo_path_str = repo_path.unwrap_or(".");
    let repo_path = PathBuf::from(repo_path_str);
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

    let sources = sources(&args, &repo_path);

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
            log::trace!("Adding variable from CLI: '{}' ...", var);
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
