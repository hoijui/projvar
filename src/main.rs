// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

extern crate clap;
extern crate enum_map;
extern crate log;
extern crate remain;
extern crate url;

use clap::lazy_static::lazy_static;
use clap::{
    crate_authors, crate_description, crate_license, crate_name, crate_version, App, AppSettings,
    Arg, ArgMatches, ValueHint,
};
use regex::Regex;
use std::collections::HashSet;
// use enumset::EnumSet;
use std::convert::{TryFrom, TryInto};
use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum::VariantNames;

mod constants;
mod environment;
mod logger;
mod process;
pub mod settings;
pub mod sinks;
pub mod sources;
mod std_error;
mod storage;
pub mod tools;
mod validator;
mod value_conversions;
mod var;

use crate::environment::Environment;
use crate::settings::{Settings, Verbosity};
use crate::sinks::VarSink;
use crate::sources::VarSource;
use crate::tools::git_hosting_provs::{self, HostingType};
use crate::var::Key;

type BoxResult<T> = Result<T, Box<dyn Error>>;

fn is_git_repo_root(repo_path: Option<&Path>) -> bool {
    tools::git::Repo::try_from(repo_path).is_ok()
}

const DEFAULT_FILE_OUT: &str = ".projvars.env.txt";

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
const A_S_FILE_OUT: char = 'O';
const A_L_FILE_OUT: &str = "file-out";
const A_S_HOSTING_TYPE: char = 't';
const A_L_HOSTING_TYPE: &str = "hosting-type";
const A_S_VERBOSE: char = 'v';
const A_L_VERBOSE: &str = "verbose";
const A_S_LOG_LEVEL: char = 'F';
const A_L_LOG_LEVEL: &str = "log-level";
const A_S_QUIET: char = 'q';
const A_L_QUIET: &str = "quiet";
const A_S_FAIL_ON_MISSING_VALUE: char = 'f';
const A_L_FAIL_ON_MISSING_VALUE: &str = "fail";
const A_S_REQUIRE_NONE: char = 'n';
const A_L_REQUIRE_NONE: &str = "none";
const A_S_REQUIRE_ALL: char = 'a';
const A_L_REQUIRE_ALL: &str = "all";
const A_S_REQUIRE: char = 'R';
const A_L_REQUIRE: &str = "require";
const A_S_REQUIRE_NOT: char = 'N';
const A_L_REQUIRE_NOT: &str = "require-not";
// const A_S_ONLY_REQUIRED: char = '?';
const A_L_ONLY_REQUIRED: &str = "only-required";
// const A_S_KEY_PREFIX: char = '?';
const A_L_KEY_PREFIX: &str = "key-prefix";
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
const A_S_SHOW_ALL_RETRIEVED: char = 'A';
const A_L_SHOW_ALL_RETRIEVED: &str = "show-all-retrieved";
const A_S_SHOW_PRIMARY_RETRIEVED: char = 'P';
const A_L_SHOW_PRIMARY_RETRIEVED: &str = "show-primary-retrieved";

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
        .about("An input file containing KEY=VALUE pairs")
        .long_about("An input file containing KEY=VALUE pairs, one per line (BASH style). Empty lines, and those starting with \"#\" or \"//\" are ignored. See -D,--variable for specifying one pair at a time.")
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
        .default_value(DEFAULT_FILE_OUT)
        .required(false)
}

fn arg_hosting_type() -> Arg<'static> {
    Arg::new(A_L_HOSTING_TYPE)
        .about("Overrides the hosting type of the primary remote")
        .long_about("As usually most kinds of repo URL property values are derived from the clone URL, it is essential to know how to construct them. Different hosting softwares construct them differently. By default, we try to derive it from the clone URL domain, but if this is not possible, this switch allows to set the hosting software manually.")
        .takes_value(true)
        .forbid_empty_values(true)
        .possible_values(git_hosting_provs::HostingType::VARIANTS)
        .short(A_S_HOSTING_TYPE)
        .long(A_L_HOSTING_TYPE)
        .multiple_occurrences(false)
        .required(false)
        .default_value(HostingType::Unknown.into())
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
        .long_about("Fail if no value is available for any of the required properties (see --all,--none,--require,--require-not)")
        .takes_value(false)
        .short(A_S_FAIL_ON_MISSING_VALUE)
        .long(A_L_FAIL_ON_MISSING_VALUE)
        .multiple_occurrences(false)
        .required(false)
}

fn arg_require_all() -> Arg<'static> {
    Arg::new(A_L_REQUIRE_ALL)
        .about("Marks all properties as required")
        .long_about("Marks all properties as required. See --none,--fail,--require,--require-not.")
        .takes_value(false)
        .short(A_S_REQUIRE_ALL)
        .long(A_L_REQUIRE_ALL)
        .multiple_occurrences(false)
        .required(false)
        // .requires(A_L_FAIL_ON_MISSING_VALUE)
        .conflicts_with(A_L_REQUIRE)
}

fn arg_require_none() -> Arg<'static> {
    Arg::new(A_L_REQUIRE_NONE)
        .about("Marks all properties as *not* required")
        .long_about(
            "Marks all properties as *not* required. See --all,--fail,--require,--require-not.",
        )
        .takes_value(false)
        .short(A_S_REQUIRE_NONE)
        .long(A_L_REQUIRE_NONE)
        .multiple_occurrences(false)
        .required(false)
        // .requires(A_L_FAIL_ON_MISSING_VALUE)
        .conflicts_with(A_L_REQUIRE_NOT)
        .conflicts_with(A_L_REQUIRE_ALL)
}

fn arg_require() -> Arg<'static> {
    Arg::new(A_L_REQUIRE)
        .about("Mark a propery as required")
        .long_about(r#"Mark a propery as required. You may use the property name (e.g. "Name") or the variable key (e.g. "PROJECT_NAME"); See --list for all possible keys. If at least one such option is present, the default required values list is cleared (see --fail,--all,--none,--require-not)."#)
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
        .long_about("A key of a variable whose value is *not* required. For example PROJECT_NAME (see --list for all possible keys). Can be used either on the base of the default requried list or all (see --fail,--all,--none,--require)")
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

fn arg_only_required() -> Arg<'static> {
    Arg::new(A_L_ONLY_REQUIRED)
        .about("Only fetch and output the required values")
        .long_about(
            "Only fetch and output the required values (see --all,--none,--require, --require-not).",
        )
        .takes_value(false)
        // .short(A_S_ONLY_REQUIRED)
        .long(A_L_ONLY_REQUIRED)
        .multiple_occurrences(false)
        .required(false)
}

fn arg_key_prefix() -> Arg<'static> {
    Arg::new(A_L_KEY_PREFIX)
        .about("The key prefix to be used for output")
        .long_about("The key prefix to be used when writing out values in the sinks. For example \"PROJECT_\" -> \"PROJECT_VERSION\", \"PROJECT_NAME\", ...")
        .takes_value(true)
        .forbid_empty_values(false)
        .value_name("STRING")
        .value_hint(ValueHint::Other)
        // .short(A_S_KEY_PREFIX)
        .long(A_L_KEY_PREFIX)
        .multiple_occurrences(false)
        .default_missing_value("")
        .default_value(constants::DEFAULT_KEY_PREFIX)
        .required(false)
}

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

fn arg_show_all_retrieved() -> Arg<'static> {
    Arg::new(A_L_SHOW_ALL_RETRIEVED)
        .about("Shows a table of all values retrieved from sources")
        .long_about("Shows a table (in Markdown syntax) of all properties and the values retrieved for each from each individual source.")
        .takes_value(false)
        .short(A_S_SHOW_ALL_RETRIEVED)
        .long(A_L_SHOW_ALL_RETRIEVED)
        .multiple_occurrences(false)
        .required(false)
}

fn arg_show_primary_retrieved() -> Arg<'static> {
    Arg::new(A_L_SHOW_PRIMARY_RETRIEVED)
        .about("Shows a list of the primary values retrieved from sources")
        .long_about("Shows a list (in Markdown syntax) of all properties and the primary values retrieved for each, accumulated over the sources.")
        .takes_value(false)
        .short(A_S_SHOW_PRIMARY_RETRIEVED)
        .long(A_L_SHOW_PRIMARY_RETRIEVED)
        .multiple_occurrences(false)
        .required(false)
        .conflicts_with(A_L_SHOW_ALL_RETRIEVED)
}

lazy_static! {
    static ref ARGS: [Arg<'static>; 24] = [
        arg_project_root(),
        arg_variable(),
        arg_variables_file(),
        arg_no_env_in(),
        arg_env_out(),
        arg_out_file(),
        arg_hosting_type(),
        arg_verbose(),
        arg_log_level(),
        arg_quiet(),
        arg_fail(),
        arg_require_all(),
        arg_require_none(),
        arg_require(),
        arg_require_not(),
        arg_only_required(),
        arg_key_prefix(),
        arg_dry(),
        arg_overwrite(),
        arg_list(),
        arg_log_file(),
        arg_date_format(),
        arg_show_all_retrieved(),
        arg_show_primary_retrieved(),
    ];
}

fn find_duplicate_short_options() -> Vec<char> {
    let mut short_options: Vec<char> = ARGS.iter().filter_map(clap::Arg::get_short).collect();
    short_options.push('h'); // standard option --help
    short_options.push('V'); // standard option --version
    short_options.sort_unstable();
    let mut duplicate_short_options = HashSet::new();
    let mut last_chr = '&';
    for chr in &short_options {
        if *chr == last_chr {
            duplicate_short_options.insert(*chr);
        }
        last_chr = *chr;
    }
    duplicate_short_options.iter().copied().collect()
}

fn arg_matcher() -> App<'static> {
    let app = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .author(crate_authors!())
        .license(crate_license!())
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::UnifiedHelpMessage)
        .bin_name("osh")
        .args(ARGS.iter());
    let duplicate_short_options = find_duplicate_short_options();
    if !duplicate_short_options.is_empty() {
        panic!(
            "Duplicate argument short options: {:?}",
            duplicate_short_options
        );
    }
    app
}

fn hosting_type(args: &ArgMatches) -> BoxResult<HostingType> {
    let hosting_type = if let Some(hosting_type_str) = args.value_of(A_L_HOSTING_TYPE) {
        HostingType::from_str(hosting_type_str)?
    } else {
        HostingType::default()
    };

    if log::log_enabled!(log::Level::Debug) {
        let hosting_type_str: &str = hosting_type.into();
        log::debug!("Hosting-type setting: {}", hosting_type_str);
    }

    Ok(hosting_type)
}

/// Returns the logging verbosities to be used.
/// The first one is for stdout&stderr,
/// the second one for log-file(s).
fn verbosity(args: &ArgMatches) -> BoxResult<(Verbosity, Verbosity)> {
    let common = if let Some(specified) = args.value_of(A_L_LOG_LEVEL) {
        Verbosity::from_str(specified)?
    } else {
        // Set the default base level
        let level = if cfg!(debug_assertions) {
            Verbosity::Debug
        } else {
            Verbosity::Info
        };
        let num_verbose = args.occurrences_of(A_L_VERBOSE).try_into()?;
        level.up_max(num_verbose)
    };

    let std = if args.is_present(A_L_QUIET) {
        Verbosity::None
    } else {
        common
    };

    Ok((std, common))
}

fn repo_path(args: &ArgMatches) -> PathBuf {
    let repo_path: Option<&str> = args.value_of(A_L_PROJECT_ROOT);
    let repo_path_str = repo_path.unwrap_or(".");
    let repo_path = PathBuf::from(repo_path_str);
    log::debug!("Using repo path '{:?}'.", repo_path);
    repo_path
}

fn date_format(args: &ArgMatches) -> &str {
    let date_format = match args.value_of(A_L_DATE_FORMAT) {
        Some(date_format) => date_format,
        None => tools::git::DATE_FORMAT,
    };
    log::debug!("Using date format '{}'.", date_format);
    date_format
}

fn sources(_args: &ArgMatches, repo_path: &Path) -> Vec<Box<dyn VarSource>> {
    let mut sources: Vec<Box<dyn VarSource>> = vec![];
    if is_git_repo_root(Some(repo_path)) {
        sources.push(Box::new(sources::git::VarSource {}));
    }
    sources.push(Box::new(sources::fs::VarSource {}));
    sources.push(Box::new(sources::bitbucket_ci::VarSource {}));
    sources.push(Box::new(sources::github_ci::VarSource {}));
    sources.push(Box::new(sources::gitlab_ci::VarSource {}));
    sources.push(Box::new(sources::jenkins_ci::VarSource {}));
    sources.push(Box::new(sources::travis_ci::VarSource {}));
    sources.push(Box::new(sources::env::VarSource {}));
    sources.push(Box::new(sources::selector::VarSource {}));
    sources.push(Box::new(sources::deriver::VarSource {}));
    // NOTE We add the deriver a second time,
    //      so it may derive from values created in the first run.
    sources.push(Box::new(sources::deriver::VarSource {}));
    if log::log_enabled!(log::Level::Trace) {
        for source in &sources {
            log::trace!("Registered source {}.", source.display());
        }
    }
    sources
}

fn sinks(args: &ArgMatches) -> BoxResult<Vec<Box<dyn VarSink>>> {
    let mut sinks: Vec<Box<dyn VarSink>> = vec![];
    if args.is_present(A_L_ENV_OUT) {
        sinks.push(Box::new(sinks::env::VarSink {}));
    }
    if args.is_present(A_L_FILE_OUT) {
        if args.occurrences_of(A_L_FILE_OUT) == 0 {
            log::info!("Using the default out file: {}", DEFAULT_FILE_OUT);
            sinks.push(Box::new(sinks::file::VarSink {
                file: PathBuf::from_str(DEFAULT_FILE_OUT)?,
            }));
        } else if let Some(out_files) = args.values_of(A_L_FILE_OUT) {
            for out_file in out_files {
                sinks.push(Box::new(sinks::file::VarSink {
                    file: PathBuf::from_str(out_file)?,
                }));
            }
        }
    }
    if args.is_present(A_L_DRY) {
        sinks.clear();
    } else if sinks.is_empty() {
        log::warn!("No sinks registered! The results of this run will not be stored anywhere.");
    }
    for sink in &sinks {
        log::trace!("Registered sink {}.", sink);
    }
    Ok(sinks)
}

fn required_keys(key_prefix: Option<&str>, args: &ArgMatches) -> BoxResult<HashSet<Key>> {
    let require_all: bool = args.is_present(A_L_REQUIRE_ALL);
    let require_none: bool = args.is_present(A_L_REQUIRE_NONE);
    let mut required_keys = if require_all {
        // EnumSet::<Key>::all()
        // HashSet::<Key>::allj()
        let mut all = HashSet::<Key>::new();
        all.extend(Key::iter());
        all
    } else if require_none {
        HashSet::<Key>::new()
    } else {
        var::default_keys().clone()
    };
    let r_key_prefix_str = format!("^{}", key_prefix.unwrap_or(""));
    let r_key_prefix = Regex::new(&r_key_prefix_str).unwrap();
    if let Some(requires) = args.values_of(A_L_REQUIRE) {
        for require in requires {
            let key = Key::from_name_or_var_key(&r_key_prefix, require)?;
            required_keys.insert(key);
        }
    }
    if let Some(require_nots) = args.values_of(A_L_REQUIRE_NOT) {
        for require_not in require_nots {
            let key = Key::from_name_or_var_key(&r_key_prefix, require_not)?;
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

    let log_file = args.value_of(A_L_LOG_FILE).map(Path::new);
    logger::init(log_file, verbosity);
    // logger::init2(log_file)?;

    if args.is_present(A_L_LIST) {
        let environment = Environment::stub();
        var::list_keys(&environment);
        return Ok(());
    }

    let repo_path = repo_path(&args);
    let date_format = date_format(&args);

    let overwrite = settings::Overwrite::from_str(args.value_of(A_L_OVERWRITE).unwrap())?;
    log::debug!("Overwriting output variable values? -> {:?}", overwrite);

    let sources = sources(&args, &repo_path);

    let sinks = sinks(&args)?;

    let fail_on_missing: bool = args.is_present(A_L_FAIL_ON_MISSING_VALUE);
    let key_prefix = args.value_of(A_L_KEY_PREFIX);
    let required_keys = required_keys(key_prefix, &args)?;
    let show_retrieved: settings::ShowRetrieved = if args.is_present(A_L_SHOW_ALL_RETRIEVED) {
        settings::ShowRetrieved::All
    } else if args.is_present(A_L_SHOW_PRIMARY_RETRIEVED) {
        settings::ShowRetrieved::Primary
    } else {
        settings::ShowRetrieved::No
    };
    let hosting_type = hosting_type(&args)?;
    let only_required = args.is_present(A_L_ONLY_REQUIRED);

    let settings = Settings {
        repo_path: Some(repo_path),
        required_keys,
        date_format: date_format.to_owned(),
        overwrite,
        fail_on: settings::FailOn::from(fail_on_missing),
        show_retrieved,
        hosting_type,
        only_required,
        key_prefix: key_prefix.map(ToOwned::to_owned),
        verbosity,
    };
    log::trace!("Created Settings.");
    let mut environment = Environment::new(&settings /*, sources, sinks*/);
    log::trace!("Created Environment.");

    // fetch environment variables
    if !args.is_present(A_L_NO_ENV_IN) {
        log::trace!("Fetching variables from the environment ...");
        repvar::tools::append_env(&mut environment.vars);
    }
    // fetch variable files
    if let Some(var_files) = args.values_of(A_L_VARIABLES_FILE) {
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
    if let Some(variables) = args.values_of(A_L_VARIABLE) {
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

    process::prepare_project_vars(&mut environment, sources, sinks)
    // Ok(())
}
