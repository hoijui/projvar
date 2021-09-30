// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::environment::Environment;
use crate::var::{Key, Variable};
use std::error::Error;
use std::{env, fmt};

pub struct VarSink;

type BoxResult<T> = Result<T, Box<dyn Error>>;

// fn flush_to_env<'a>(
//     // vars: Box<dyn Iterator<Item=(&String, &String)>>,
//     vars: Iter<'a, (&String, &String)>,
//     overwrite: bool,
// ) {
//     for (key, value) in vars {
//         if overwrite || env::var(&key).is_err() {
//             env::set_var(&key, &value);
//         }
//     }
// }

/// Stores evaluated values (output) into environment variables.
impl super::VarSink for VarSink {
    fn is_usable(&self, _environment: &mut Environment) -> bool {
        true
    }

    fn store(
        &self,
        environment: &mut Environment,
        values: &[(Key, &Variable, String)],
    ) -> BoxResult<()> {
        for (_key, var, value) in values {
            let key = var.key;
            if environment.settings.overwrite.main() || env::var(&key).is_err() {
                env::set_var(&key, &value);
            }
        }
        Ok(())
    }
}

impl fmt::Display for VarSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::any::type_name::<VarSink>())
    }
}
