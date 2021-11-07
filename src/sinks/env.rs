// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::environment::Environment;
use crate::var::{Confidence, Key, Variable};
use std::error::Error;
use std::{env, fmt};

pub struct VarSink;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Stores evaluated values (output) into environment variables.
impl super::VarSink for VarSink {
    fn is_usable(&self, _environment: &Environment) -> bool {
        true
    }

    fn store(
        &self,
        environment: &Environment,
        values: &[(Key, &Variable, &(Confidence, String))],
    ) -> BoxResult<()> {
        for (_key, var, rated_value) in values {
            let key = var.key(environment);
            if environment.settings.overwrite.main() || env::var(&*key).is_err() {
                env::set_var(&*key, &rated_value.1);
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
