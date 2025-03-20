// SPDX-FileCopyrightText: 2021-2023 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

extern crate clap;
extern crate enum_map;
extern crate human_panic;
extern crate log;
extern crate remain;
extern crate url;

use clap::builder::ValueParser;
use clap::{command, value_parser, Arg, ArgAction, ArgMatches, Command, ValueHint};
use cli_utils::BoxResult;
use const_format::formatcp;
use regex::Regex;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::LazyLock;
use strum::IntoEnumIterator;

mod cleanup;
mod constants;
mod environment;
mod license;
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
use crate::tools::git_hosting_provs::{self, HostingType};
use crate::var::Key;

pub const A_L_VERSION: &str = "version";
pub const A_S_VERSION: char = 'V';
const A_S_PROJECT_ROOT: char = 'C';
const A_L_PROJECT_ROOT: &str = "project-root";
const A_L_RAW_PANIC: &str = "raw-panic";
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
const A_S_DATE_FORMAT: char = 'T';
const A_L_DATE_FORMAT: &str = "date-format";
const A_S_SHOW_ALL_RETRIEVED: char = 'A';
const A_L_SHOW_ALL_RETRIEVED: &str = "show-all-retrieved";
const A_S_SHOW_PRIMARY_RETRIEVED: char = 'P';
const A_L_SHOW_PRIMARY_RETRIEVED: &str = "show-primary-retrieved";

fn arg_version() -> Arg {
    Arg::new(A_L_VERSION)
        .help(formatcp!("Print version information and exit. May be combined with -{A_S_QUIET},--{A_L_QUIET}, to really only output the version string."))
        .short(A_S_VERSION)
        .long(A_L_VERSION)
        .action(ArgAction::SetTrue)
}

fn arg_project_root() -> Arg {
    Arg::new(A_L_PROJECT_ROOT)
        .help("The root dir of the project")
        .long_help(
            "The root directory of the project, \
            mainly used for SCM (e.g. git) information gathering.",
        )
        .num_args(1)
        .value_parser(value_parser!(std::path::PathBuf))
        .value_name("DIR")
        .value_hint(ValueHint::DirPath)
        .short(A_S_PROJECT_ROOT)
        .long(A_L_PROJECT_ROOT)
        .action(ArgAction::Set)
        .required(false)
        .default_value(".")
}

fn arg_raw_panic() -> Arg {
    Arg::new(A_L_RAW_PANIC)
        .help("Use rusts native panic handling, if one occurs.")
        .long_help(
            "Do not wrap rusts native panic handling functionality \
            in a more end-user-friendly way. \
            This is particularly useful when running on CI.",
        )
        .action(ArgAction::SetTrue)
        .long(A_L_RAW_PANIC)
}

fn arg_variable() -> Arg {
    Arg::new(A_L_VARIABLE)
        .help("A key-value pair to be used as input")
        .long_help(formatcp!(
            "A key-value pair (aka a variable) to be used as input, \
            as it it was specified as an environment variable. \
            Values provided with this take precedence over environment variables - \
            they overwrite them. \
            See -{A_S_VARIABLES_FILE},--{A_L_VARIABLES_FILE} for supplying a lot of such pairs at once.",
        ))
        .num_args(1)
        .value_name("KEY=VALUE")
        .value_hint(ValueHint::Other)
        .value_parser(ValueParser::new(var::parse_key_value_str))
        .short(A_S_VARIABLE)
        .long(A_L_VARIABLE)
        .action(ArgAction::Append)
        .required(false)
}

fn arg_variables_file() -> Arg {
    Arg::new(A_L_VARIABLES_FILE)
        .help("An input file containing KEY=VALUE pairs")
        .long_help(formatcp!(
            "An input file containing KEY=VALUE pairs, one per line (BASH style). \
            Empty lines, and those starting with \"#\" or \"//\" are ignored. \
            See -{A_S_VARIABLE},--{A_L_VARIABLE} for specifying one pair at a time.",
        ))
        .num_args(1)
        .value_parser(value_parser!(std::path::PathBuf))
        .value_name("FILE")
        .value_hint(ValueHint::FilePath)
        .short(A_S_VARIABLES_FILE)
        .long(A_L_VARIABLES_FILE)
        .action(ArgAction::Append)
        .required(false)
        .default_missing_value("-")
}

fn arg_no_env_in() -> Arg {
    Arg::new(A_L_NO_ENV_IN)
        .help("Do not read environment variables")
        .long_help("Disable the use of environment variables as input")
        .action(ArgAction::SetTrue)
        .short(A_S_NO_ENV_IN)
        .long(A_L_NO_ENV_IN)
        .required(false)
}

fn arg_env_out() -> Arg {
    Arg::new(A_L_ENV_OUT)
        .help("Write resulting values directly into the environment") // TODO Check: is that even possible? As in, the values remaining in the environment after the end of the process?
        .action(ArgAction::SetTrue)
        .short(A_S_ENV_OUT)
        .long(A_L_ENV_OUT)
        .required(false)
}

fn arg_out_file() -> Arg {
    Arg::new(A_L_FILE_OUT)
        .help("Write variables into this file; .env or .json")
        .long_help(
            "Write evaluated values into a file. \
            Two file formats are supported: \
            * ENV: one KEY=VALUE pair per line (BASH syntax) \
            * JSON: a dictionary of KEY: \"value\" \
            You can choose which format is used by the file-extension.
            Note that \"-\" has no special meaning here; \
            it does not mean stdout, but rather the file \"./-\".",
        )
        .num_args(1)
        .value_parser(value_parser!(std::path::PathBuf))
        .value_name("FILE")
        .value_hint(ValueHint::FilePath)
        .short(A_S_FILE_OUT)
        .long(A_L_FILE_OUT)
        .action(ArgAction::Set)
        .default_value(sinks::DEFAULT_FILE_OUT)
        .required(false)
}

fn arg_hosting_type() -> Arg {
    Arg::new(A_L_HOSTING_TYPE)
        .help("Overrides the hosting type of the primary remote")
        .long_help(
            "As usually most kinds of repo URL property values are derived from the clone URL, \
            it is essential to know how to construct them. \
            Different hosting software construct them differently. \
            By default, we try to derive it from the clone URL domain, \
            but if this is not possible, \
            this switch allows to set the hosting software manually.",
        )
        .num_args(1)
        .value_parser(value_parser!(git_hosting_provs::HostingType))
        .short(A_S_HOSTING_TYPE)
        .long(A_L_HOSTING_TYPE)
        .action(ArgAction::Set)
        .required(false)
}

fn arg_verbose() -> Arg {
    Arg::new(A_L_VERBOSE)
        .help("More verbose log output")
        .long_help(formatcp!(
            "More verbose log output; useful for debugging. \
            See -{A_S_LOG_LEVEL},--{A_L_LOG_LEVEL} for more fine-grained control.",
        ))
        .short(A_S_VERBOSE)
        .long(A_L_VERBOSE)
        .action(ArgAction::Count)
        .required(false)
}

fn arg_log_level() -> Arg {
    Arg::new(A_L_LOG_LEVEL)
        .help("Set the log-level")
        .value_parser(value_parser!(settings::Verbosity))
        .short(A_S_LOG_LEVEL)
        .long(A_L_LOG_LEVEL)
        .action(ArgAction::Set)
        .required(false)
        .conflicts_with(A_L_VERBOSE)
        .conflicts_with(A_L_QUIET)
}

fn arg_quiet() -> Arg {
    Arg::new(A_L_QUIET)
        .help("Minimize or suppress output to stdout")
        .long_help(formatcp!(
            "Minimize or suppress output to stdout, \
and only shows log output on stderr. \
See -{A_S_LOG_LEVEL},--{A_L_LOG_LEVEL} to also disable the later. \
This does not affect the log level for the log-file.",
        ))
        .action(ArgAction::SetTrue)
        .short(A_S_QUIET)
        .long(A_L_QUIET)
        .required(false)
        .conflicts_with(A_L_VERBOSE)
}

fn arg_fail() -> Arg {
    Arg::new(A_L_FAIL_ON_MISSING_VALUE)
        .help("Fail if a required value is missing")
        .long_help(formatcp!(
            "Fail if no value is available for any of the required properties. \
            See --{A_L_REQUIRE_ALL}, --{A_L_REQUIRE_NONE}, --{A_L_REQUIRE}, --{A_L_REQUIRE_NOT}.",
        ))
        .action(ArgAction::SetTrue)
        .short(A_S_FAIL_ON_MISSING_VALUE)
        .long(A_L_FAIL_ON_MISSING_VALUE)
        .required(false)
}

fn arg_require_all() -> Arg {
    Arg::new(A_L_REQUIRE_ALL)
        .help("Marks all properties as required")
        .long_help(formatcp!(
            "Marks all properties as required. \
            See --{A_L_REQUIRE_NONE}, --{A_L_FAIL_ON_MISSING_VALUE}, --{A_L_REQUIRE}, --{A_L_REQUIRE_NOT}.",
        ))
        .action(ArgAction::SetTrue)
        .short(A_S_REQUIRE_ALL)
        .long(A_L_REQUIRE_ALL)
        .required(false)
        // .requires(A_L_FAIL_ON_MISSING_VALUE)
        .conflicts_with(A_L_REQUIRE)
}

fn arg_require_none() -> Arg {
    Arg::new(A_L_REQUIRE_NONE)
        .help("Marks all properties as *not* required")
        .long_help(formatcp!(
            "Marks all properties as *not* required. \
            See --{A_L_REQUIRE_ALL}, --{A_L_FAIL_ON_MISSING_VALUE}, --{A_L_REQUIRE}, --{A_L_REQUIRE_NOT}.",
        ))
        .action(ArgAction::SetTrue)
        .short(A_S_REQUIRE_NONE)
        .long(A_L_REQUIRE_NONE)
        .required(false)
        // .requires(A_L_FAIL_ON_MISSING_VALUE)
        .conflicts_with(A_L_REQUIRE_NOT)
        .conflicts_with(A_L_REQUIRE_ALL)
}

fn arg_require() -> Arg {
    #![allow(clippy::needless_raw_string_hashes)]
    Arg::new(A_L_REQUIRE)
        .help("Mark a property as required")
        .long_help(formatcp!(
            r#"Mark a property as required. \
            You may use the property name (e.g. "Name") \
            or the variable key (e.g. "PROJECT_NAME"); \
            See --{A_L_LIST} for all possible keys. \
            If at least one such option is present, \
            the default required values list is cleared. \
            See --{A_L_FAIL_ON_MISSING_VALUE}, --{A_L_REQUIRE_ALL}, --{A_L_REQUIRE_NONE}, --{A_L_REQUIRE_NOT}."#,
        ))
        .num_args(1)
        .value_parser(clap::builder::NonEmptyStringValueParser::new()) // TODO Maybe parse into Key directly here already?
        .value_name("KEY")
        .value_hint(ValueHint::Other)
        .short(A_S_REQUIRE)
        .long(A_L_REQUIRE)
        .action(ArgAction::Append)
        .required(false)
        .requires(A_L_FAIL_ON_MISSING_VALUE)
        .conflicts_with(A_L_REQUIRE_NOT)
        .conflicts_with(A_L_REQUIRE_ALL)
}

fn arg_require_not() -> Arg {
    Arg::new(A_L_REQUIRE_NOT)
        .help("Mark a property as not required")
        .long_help(formatcp!(
            "A key of a variable whose value is *not* required. \
            For example PROJECT_NAME (see --{A_L_LIST} for all possible keys). \
            Can be used either on the base of the default required list \
            or all. \
            See --{A_L_FAIL_ON_MISSING_VALUE}, --{A_L_REQUIRE_ALL}, --{A_L_REQUIRE_NONE}, --{A_L_REQUIRE}.",
        ))
        .num_args(1)
        .value_parser(clap::builder::NonEmptyStringValueParser::new()) // TODO Maybe parse into Key directly here already?
        .value_name("KEY")
        .value_hint(ValueHint::Other)
        .short(A_S_REQUIRE_NOT)
        .long(A_L_REQUIRE_NOT)
        .action(ArgAction::Append)
        .required(false)
        .requires(A_L_FAIL_ON_MISSING_VALUE)
        .conflicts_with(A_L_REQUIRE)
}

fn arg_only_required() -> Arg {
    Arg::new(A_L_ONLY_REQUIRED)
        .help("Only output the required values")
        .long_help(formatcp!(
            "Only output the required values. \
            See --{A_L_REQUIRE_ALL}, --{A_L_REQUIRE_NONE}, --{A_L_REQUIRE}, --{A_L_REQUIRE_NOT}.",
        ))
        .action(ArgAction::SetTrue)
        // .short(A_S_ONLY_REQUIRED)
        .long(A_L_ONLY_REQUIRED)
        .required(false)
}

fn arg_key_prefix() -> Arg {
    Arg::new(A_L_KEY_PREFIX)
        .help("The key prefix to be used for output")
        .long_help(
            "The key prefix to be used when writing out values in the sinks. \
            For example \"PROJECT_\" -> \"PROJECT_VERSION\", \"PROJECT_NAME\", ...",
        )
        .num_args(1)
        .value_name("STRING")
        .value_parser(clap::builder::StringValueParser::new()) // TODO Maybe check for illegal chars directly here?
        .value_hint(ValueHint::Other)
        // .short(A_S_KEY_PREFIX)
        .long(A_L_KEY_PREFIX)
        .action(ArgAction::Set)
        .default_missing_value("")
        .default_value(constants::DEFAULT_KEY_PREFIX)
        .required(false)
}

fn arg_dry() -> Arg {
    Arg::new(A_L_DRY)
        .help("Do not write any files or set any environment variables")
        .long_help("Set Whether to skip the actual setting of environment variables.")
        .action(ArgAction::SetTrue)
        .short(A_S_DRY)
        .long(A_L_DRY)
        .required(false)
}

fn arg_overwrite() -> Arg {
    Arg::new(A_L_OVERWRITE)
        .help("Whether to overwrite already set values in the output.")
        .num_args(1)
        .value_parser(value_parser!(settings::Overwrite))
        .short(A_S_OVERWRITE)
        .long(A_L_OVERWRITE)
        .action(ArgAction::Set)
        .required(false)
        .conflicts_with(A_L_DRY)
}

fn arg_list() -> Arg {
    Arg::new(A_L_LIST)
        .help("Show all properties and their keys")
        .long_help(
            "Prints a list of all the environment variables \
            that are potentially set by this tool onto stdout and exits.",
        )
        .action(ArgAction::SetTrue)
        .short(A_S_LIST)
        .long(A_L_LIST)
        .required(false)
}

fn arg_date_format() -> Arg {
    Arg::new(A_L_DATE_FORMAT)
        .help("Date format for generated dates")
        .long_help(
            "Date format string for generated (vs supplied) dates. \
            For details, see https://docs.rs/chrono/latest/chrono/format/strftime/index.html",
        )
        .num_args(1)
        .value_parser(clap::builder::NonEmptyStringValueParser::new()) // TODO Maybe parse directly into a date format?
        .value_hint(ValueHint::Other)
        .short(A_S_DATE_FORMAT)
        .long(A_L_DATE_FORMAT)
        .action(ArgAction::Set)
        .default_value(tools::git::DATE_FORMAT)
        .required(false)
}

fn arg_show_all_retrieved() -> Arg {
    Arg::new(A_L_SHOW_ALL_RETRIEVED)
        .help("Shows a table of all values retrieved from sources")
        .long_help(
            "Shows a table (in Markdown syntax) of all properties and the values \
            retrieved for each from each individual source. \
            Writes to log(Info), if no target file is given as argument.",
        )
        .num_args(0..=1)
        .value_hint(ValueHint::FilePath)
        .value_name("MD-FILE")
        .value_parser(value_parser!(std::path::PathBuf))
        .short(A_S_SHOW_ALL_RETRIEVED)
        .long(A_L_SHOW_ALL_RETRIEVED)
        .action(ArgAction::Set)
        .required(false)
}

fn arg_show_primary_retrieved() -> Arg {
    Arg::new(A_L_SHOW_PRIMARY_RETRIEVED)
        .help("Shows a list of the primary values retrieved from sources")
        .long_help(
            "Shows a list (in Markdown syntax) of all properties \
            and the primary values retrieved for each, \
            accumulated over the sources. \
            Writes to log(Info), if no target file is given as argument.",
        )
        .num_args(0..=1)
        .value_hint(ValueHint::FilePath)
        .value_name("MD-FILE")
        .value_parser(value_parser!(std::path::PathBuf))
        .short(A_S_SHOW_PRIMARY_RETRIEVED)
        .long(A_L_SHOW_PRIMARY_RETRIEVED)
        .action(ArgAction::Set)
        .required(false)
        .conflicts_with(A_L_SHOW_ALL_RETRIEVED)
}

static ARGS: LazyLock<[Arg; 25]> = LazyLock::new(|| {
    [
        arg_version(),
        arg_project_root(),
        arg_raw_panic(),
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
        arg_date_format(),
        arg_show_all_retrieved(),
        arg_show_primary_retrieved(),
    ]
});

fn find_duplicate_short_options() -> Vec<char> {
    let mut short_options: Vec<char> = ARGS.iter().filter_map(clap::Arg::get_short).collect();
    // standard option --help
    short_options.push('h');
    // standard option --version
    // short_options.push('V'); // NOTE We handle this manually now
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

fn arg_matcher() -> Command {
    let app = command!()
        .bin_name(clap::crate_name!())
        .help_expected(true)
        .disable_version_flag(true)
        .args(ARGS.iter());
    let duplicate_short_options = find_duplicate_short_options();
    assert!(
        duplicate_short_options.is_empty(),
        "Duplicate argument short options: {duplicate_short_options:?}"
    );
    app
}

fn hosting_type(args: &ArgMatches) -> HostingType {
    let hosting_type = args
        .get_one::<HostingType>(A_L_HOSTING_TYPE)
        .copied()
        .unwrap_or_default();

    if log::log_enabled!(log::Level::Debug) {
        let hosting_type_str: &str = hosting_type.into();
        log::debug!("Hosting-type setting: {}", hosting_type_str);
    }

    hosting_type
}

fn overwrite(args: &ArgMatches) -> settings::Overwrite {
    let overwrite = args
        .get_one::<settings::Overwrite>(A_L_OVERWRITE)
        .copied()
        .unwrap_or_default();

    if log::log_enabled!(log::Level::Debug) {
        let overwrite_str: &str = overwrite.into();
        log::debug!("Overwriting output variable values? -> {}", overwrite_str);
    }

    overwrite
}

/// Returns the logging verbosity to be used.
/// We only log to stderr;
/// if the user wants to log anywhere else,
/// they have to redirect from there.
/// We are simple enough to not having to worry about
/// complex logging schemes.
/// ... right? :/
fn verbosity(args: &ArgMatches) -> Verbosity {
    if args.get_flag(A_L_QUIET) {
        Verbosity::None
    } else if let Some(specified) = args.get_one::<Verbosity>(A_L_LOG_LEVEL).copied() {
        specified
    } else {
        // Set the default base level
        let level = if cfg!(debug_assertions) {
            Verbosity::Debug
        } else {
            Verbosity::Info
        };
        let num_verbose = *args.get_one::<u8>(A_L_VERBOSE).unwrap_or(&0);
        level.up_max(num_verbose)
    }
}

fn repo_path(args: &ArgMatches) -> PathBuf {
    let repo_path = args
        .get_one::<PathBuf>(A_L_PROJECT_ROOT)
        .cloned()
        .unwrap_or_else(PathBuf::new);
    log::debug!("Using repo path {:#?}.", &repo_path);
    repo_path
}

fn date_format(args: &ArgMatches) -> &str {
    let date_format = match args.get_one::<String>(A_L_DATE_FORMAT) {
        Some(date_format) => date_format,
        None => tools::git::DATE_FORMAT,
    };
    log::debug!("Using date format '{}'.", date_format);
    date_format
}

fn sinks_cli(args: &ArgMatches) -> Vec<Box<dyn VarSink>> {
    let env_out = args.get_flag(A_L_ENV_OUT);
    let dry = args.get_flag(A_L_DRY);

    let mut default_out_file = true;
    let mut additional_out_files = vec![];
    if let Some(out_files) = args.get_many::<PathBuf>(A_L_FILE_OUT) {
        for out_file in out_files {
            additional_out_files.push(out_file.into());
            default_out_file = false;
        }
    }

    sinks::cli_list(env_out, dry, default_out_file, additional_out_files)
}

fn required_keys(key_prefix: Option<String>, args: &ArgMatches) -> BoxResult<HashSet<Key>> {
    let require_all: bool = args.get_flag(A_L_REQUIRE_ALL);
    let require_none: bool = args.get_flag(A_L_REQUIRE_NONE);
    let mut required_keys = if require_all {
        let mut all = HashSet::<Key>::new();
        all.extend(Key::iter());
        all
    } else if require_none {
        HashSet::<Key>::new()
    } else {
        var::default_keys().clone()
    };
    let r_key_prefix_str = format!("^{}", key_prefix.unwrap_or_default());
    let r_key_prefix = Regex::new(&r_key_prefix_str).unwrap();
    if let Some(requires) = args.get_many::<String>(A_L_REQUIRE) {
        for require in requires {
            let key = Key::from_name_or_var_key(&r_key_prefix, require)?;
            required_keys.insert(key);
        }
    }
    if let Some(require_nots) = args.get_many::<String>(A_L_REQUIRE_NOT) {
        for require_not in require_nots {
            let key = Key::from_name_or_var_key(&r_key_prefix, require_not)?;
            required_keys.remove(&key);
        }
    }
    // make immutable
    let required_keys = required_keys;
    if log::log_enabled!(log::Level::Trace) {
        for required_key in &required_keys {
            log::trace!("Registered required key {:?}.", required_key);
        }
    }

    Ok(required_keys)
}

fn print_version_and_exit(quiet: bool) {
    #![allow(clippy::print_stdout)]

    if !quiet {
        print!("{} ", clap::crate_name!());
    }
    println!("{}", projvar::VERSION);
    std::process::exit(0);
}

fn main() -> BoxResult<()> {
    let log_filter_reload_handle = logger::setup_logging()?;
    let initial_verbosity = if cfg!(debug_assertions) {
        Verbosity::Debug
    } else {
        Verbosity::Info
    };
    logger::set_log_level(&log_filter_reload_handle, initial_verbosity)?;

    let args = arg_matcher().get_matches();

    if !args.get_flag(A_L_RAW_PANIC) {
        human_panic::setup_panic!();
    }

    let quiet = args.get_flag(A_L_QUIET);

    let version = args.get_flag(A_L_VERSION);
    if version {
        print_version_and_exit(quiet);
    }

    let verbosity = verbosity(&args);
    logger::set_log_level(&log_filter_reload_handle, verbosity)?;

    if args.get_flag(A_L_LIST) {
        let environment = Environment::stub();
        let list = var::list_keys(&environment);
        log::info!("{}", list);
        return Ok(());
    }

    let repo_path = repo_path(&args);
    let date_format = date_format(&args);

    let overwrite = overwrite(&args);

    log::trace!("Collecting sources ...");
    let sources = sources::default_list(&repo_path);

    log::trace!("Collecting sinks ...");
    let sinks = sinks_cli(&args);

    log::trace!("Collecting more settings ...");
    let fail_on_missing = args.get_flag(A_L_FAIL_ON_MISSING_VALUE);
    let key_prefix = args.get_one::<String>(A_L_KEY_PREFIX).cloned();
    log::trace!("Collecting required keys ...");
    let required_keys = required_keys(key_prefix.clone(), &args)?;
    log::trace!("Collecting setting 'show-retrieved?' ...");
    let show_retrieved: settings::ShowRetrieved = if args.contains_id(A_L_SHOW_ALL_RETRIEVED) {
        settings::ShowRetrieved::All(
            args.get_one::<PathBuf>(A_L_SHOW_ALL_RETRIEVED)
                .map(std::convert::Into::into),
        )
    } else if args.contains_id(A_L_SHOW_PRIMARY_RETRIEVED) {
        settings::ShowRetrieved::Primary(
            args.get_one::<PathBuf>(A_L_SHOW_PRIMARY_RETRIEVED)
                .map(std::convert::Into::into),
        )
    } else {
        settings::ShowRetrieved::No
    };
    log::trace!("Collecting yet more settings ...");
    let hosting_type = hosting_type(&args);
    let only_required = args.get_flag(A_L_ONLY_REQUIRED);

    let settings = Settings {
        repo_path: Some(repo_path),
        required_keys,
        date_format: date_format.to_owned(),
        overwrite,
        fail_on: settings::FailOn::from(fail_on_missing),
        show_retrieved,
        hosting_type,
        only_required,
        key_prefix,
        verbosity,
    };
    log::trace!("Created Settings.");
    let mut environment = Environment::new(settings);
    log::trace!("Created Environment.");

    // fetch environment variables
    if !args.get_flag(A_L_NO_ENV_IN) {
        log::trace!("Fetching variables from the environment ...");
        repvar::tools::append_env(&mut environment.vars);
    }
    // fetch variables files
    if let Some(var_files) = args.get_many::<PathBuf>(A_L_VARIABLES_FILE) {
        for var_file in var_files.cloned() {
            if var_file.to_string_lossy() == "-" {
                log::trace!("Fetching variables from stdin ...");
            } else {
                log::trace!("Fetching variables from file '{}' ...", var_file.display());
            }
            let mut reader = cli_utils::create_input_reader(Some(var_file))?;
            environment
                .vars
                .extend(var::parse_vars_file_reader(&mut reader)?);
        }
    }
    // insert CLI supplied variables values
    if let Some(variables) = args.get_many::<(String, String)>(A_L_VARIABLE) {
        for (key, value) in variables {
            log::trace!("Adding variable from CLI: {}='{}' ...", key, value);
            environment.vars.insert(key.clone(), value.clone());
        }
    }

    process::run(&mut environment, sources, sinks)
    // Ok(())
}
