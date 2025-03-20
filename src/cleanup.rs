// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::LazyLock;

use crate::{environment::Environment, sources::ConfVal};
use regex::Regex;

pub fn version(_environment: &mut Environment, value: &str) -> Option<String> {
    static R_V_PREFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[vV][.]?[ \t]*").unwrap());
    let stripped_value = R_V_PREFIX.replace(value, "");
    if stripped_value == value {
        None
    } else {
        Some(stripped_value.into_owned())
    }
}

pub fn conf_version(environment: &mut Environment, conf_val: ConfVal) -> ConfVal {
    match version(environment, &conf_val.1) {
        Some(cleaner_val) => (conf_val.0, cleaner_val),
        None => conf_val,
    }
}

// macro_rules! version {
//     (environment: &mut Environment, conf_val: &(Confidence, String)) => {
//         {match bare_version(environment, &conf_val.1) {
//             Some(cleaner_val) => (conf_val.0, cleaner_val),
//             None => conf_val
//         }}
//     }
// }
// pub(crate) use version;
