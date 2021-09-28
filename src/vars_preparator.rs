// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::settings::FailOn;
use crate::sinks::VarSink;
use crate::sources::VarSource;
use crate::validator;
use crate::var::{self, Key, Variable};
use crate::{environment::Environment, validator::ValidationError};
use std::error::Error;
use strum::IntoEnumIterator;

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// The main function of this crate,
/// gathering data as good as it can,
/// and making sure it is stored in the appropriate environment variables.
///
/// # Errors
///
/// Reading from the environment fails.
///
/// Any of the alternative methods to come up with a value
/// for a specific key fails.
///
/// Writing to the environment fails.
pub fn prepare_project_vars(
    environment: &mut Environment,
    sources: Vec<Box<dyn VarSource>>,
    sinks: Vec<Box<dyn VarSink>>,
) -> BoxResult<()> {
    for source in sources {
        if source.is_usable(environment) {
            log::trace!("Trying to fetch from source {} ...", source);
            for key in Key::iter() {
                let value = source.retrieve(environment, key.clone())?;
                if let Some(value) = value {
                    log::trace!("\tFetched {:?}='{}'", key, value);
                    environment.output.insert(key, value);
                }
            }
        }
    }

    log::trace!("Validate each variables precense and value ...");
    let output = environment.output.clone();
    for ref key in Key::iter() {
        let required = environment.settings.required_keys.contains(key);
        match output.get(key) {
            Some(value) => {
                log::trace!("Validating value for key '{:?}': '{}'", key, value);
                validator::get(key)(environment, value)?;
            }
            None => {
                if required {
                    log::warn!("Missing value for required key '{:?}'", key);
                    match environment.settings.fail_on {
                        FailOn::AnyMissingValue => Err(ValidationError::Missing)?, // TODO Should/could/is this handled in the validator already?
                        FailOn::Error => (),
                    }
                } else {
                    log::debug!("Missing value for optional key '{:?}'", key);
                }
            }
        }
    }

    log::trace!("Evaluated variables ...");
    let values: Vec<(Key, &'static Variable, String)> = {
        environment
            .output
            .iter()
            .map(|key_value| {
                let key = key_value.0.clone();
                let variable = var::get(key_value.0.clone());
                let value = key_value.1.clone();
                log::trace!("\t{:?}:{}='{}'", key, variable.key, &value);
                (key, variable, value)
            })
            .collect()
    };

    for ref sink in sinks {
        log::trace!("Checking if sink {} is usable ...", sink);
        if sink.is_usable(environment) {
            log::trace!("Storing to sink {} ...", sink);
            sink.store(environment, &values)?;
        }
    }

    log::trace!("Done.");

    Ok(())
}
