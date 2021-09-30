// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::environment::Environment;
use crate::settings::FailOn;
use crate::sinks::VarSink;
use crate::sources::VarSource;
use crate::validator;
use crate::var::{self, Key, Variable};
use std::cmp::Ordering;
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
    mut sources: Vec<Box<dyn VarSource>>,
    sinks: Vec<Box<dyn VarSink>>,
) -> BoxResult<()> {
    // sources.sort_unstable_by_key(|s| (s.hierarchy(), s.type_name(), s.properties().clone()));
    sources.sort_unstable_by(|s1, s2| {
        let o_hierarchy = s1.hierarchy().cmp(&s2.hierarchy());
        if let Ordering::Equal = o_hierarchy {
            let o_type = s1.type_name().cmp(s2.type_name());
            if let Ordering::Equal = o_type {
                let o_props = s1.properties().cmp(s2.properties());
                o_props
            } else {
                o_type
            }
        } else {
            o_hierarchy
        }
    });

    for source in sources {
        if source.is_usable(environment) {
            log::trace!("Trying to fetch from source {} ...", source.display());
            for key in Key::iter() {
                let value = source.retrieve(environment, key)?;
                if let Some(value) = value {
                    log::trace!("\tFetched {:?}='{}'", key, value);
                    environment.output.insert(key, value);
                }
            }
        }
    }

    log::trace!("Validate each variables precense and value ...");
    let output = environment.output.clone();
    for key in Key::iter() {
        let required = environment.settings.required_keys.contains(&key);
        match output.get(&key) {
            Some(value) => {
                log::trace!("Validating value for key '{:?}': '{}'", key, value);
                validator::get(key)(environment, value)?;
            }
            None => {
                if required {
                    log::warn!("Missing value for required key '{:?}'", key);
                    match environment.settings.fail_on {
                        FailOn::AnyMissingValue => return Err(validator::Error::Missing.into()), // TODO Should/could/is this be handled in the validator already?
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
                let key = *key_value.0;
                let variable = var::get(*key_value.0);
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
