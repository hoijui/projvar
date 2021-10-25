// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::environment::Environment;
use crate::settings::FailOn;
use crate::sinks::VarSink;
use crate::sources::VarSource;
use crate::validator;
use crate::var::Key;
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

    for (source_index, source) in sources.iter().enumerate() {
        if source.is_usable(environment) {
            log::trace!("Trying to fetch from source {} ...", source.display());
            for key in Key::iter() {
                if environment.settings.only_required
                    && !environment.settings.required_keys.contains(&key)
                {
                    log::trace!("\tSkip fetching {:?} because it is not required", key);
                    continue;
                }
                let rated_value = source.retrieve(environment, key)?;
                if let Some((confidence, value)) = rated_value {
                    log::trace!("\tFetched {:?}='{}'", key, value);
                    environment.output.add(key, source_index, confidence, value);
                }
            }
        }
    }

    // Report raw values retrieved from the sources,
    // if requested
    let retrieved_display = match environment.settings.show_retrieved {
        crate::settings::ShowRetrieved::No => None,
        crate::settings::ShowRetrieved::Primary => Some(environment.output.to_list(environment)),
        crate::settings::ShowRetrieved::All => {
            Some(environment.output.to_table(environment, &sources))
        }
    };
    if let Some(retrieved_display) = retrieved_display {
        log::info!(
            "Raw, Retrieved values from sources:\n\n{}",
            retrieved_display
        );
    }

    log::trace!("Validate each variables precense and value ...");
    let output = environment.output.clone();
    for key in Key::iter() {
        let required = environment.settings.required_keys.contains(&key);
        match output.get(key) {
            Some((_confidence, value)) => {
                log::trace!("Validating value for key '{:?}': '{}'", key, value);
                let validation_res = validator::get(key)(environment, value);
                match validation_res {
                    Ok(validity) => {
                        log::info!("Validation result for key '{:?}': {:?}", key, validity);
                    }
                    Err(err) => {
                        log::error!("Validation result for key '{:?}': {:?}", key, err);
                        return Err(Box::new(err));
                    }
                }
            }
            None => {
                if required {
                    log::warn!("Missing value for required key '{:?}'", key);
                    match environment.settings.fail_on {
                        FailOn::AnyMissingValue => return Err(validator::Error::Missing.into()), // TODO Should/could this be handled in the validator already?
                        FailOn::Error => (),
                    }
                } else {
                    log::debug!("Missing value for optional key '{:?}'", key);
                }
            }
        }
    }

    log::trace!("Evaluated variables ...");
    // let values: Vec<(Key, &'static Variable, &String)> = environment.output.get_wrapup();
    let values = environment.output.get_wrapup();
    if log::log_enabled!(log::Level::Trace) {
        for (key, variable, (confidence, value)) in &values {
            log::trace!(
                "\t{:?}:{}:{}='{}' ",
                key,
                variable.key(environment),
                confidence,
                &value
            );
        }
    }

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
