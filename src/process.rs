// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::environment::Environment;
use crate::settings::FailOn;
use crate::sinks::VarSink;
use crate::sources::VarSource;
use crate::var::Key;
use crate::{validator, BoxResult};
use std::cmp::Ordering;
use std::fs;
use strum::IntoEnumIterator;

/// Reports the raw values retrieved from the sources, if requested
fn report_retrieved(environment: &Environment, sources: &[Box<dyn VarSource>]) -> BoxResult<()> {
    let retrieved = match &environment.settings.show_retrieved {
        crate::settings::ShowRetrieved::No => (None, None),
        crate::settings::ShowRetrieved::Primary(target) => (
            Some(environment.output.to_list(environment)),
            target.as_ref(),
        ),
        crate::settings::ShowRetrieved::All(target) => (
            Some(environment.output.to_table(environment, sources)),
            target.as_ref(),
        ),
    };
    if let (Some(retr_content), target) = retrieved {
        match target {
            None => {
                log::info!("Raw, Retrieved values from sources:\n\n{}", retr_content,);
            }
            Some(path) => {
                fs::write(path, retr_content)?;
            }
        }
    }
    Ok(())
}

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
pub fn run(
    environment: &mut Environment,
    mut sources: Vec<Box<dyn VarSource>>,
    sinks: Vec<Box<dyn VarSink>>,
) -> BoxResult<()> {
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
                let rated_value = source.retrieve(environment, key)?;
                if let Some((confidence, value)) = rated_value {
                    log::trace!("\tFetched {:?}='{}'", key, value);
                    environment.output.add(key, source_index, confidence, value);
                }
            }
        }
    }

    report_retrieved(environment, &sources)?;

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
                        log::debug!("Validation result for key '{:?}': {:?}", key, validity);
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

    let values = environment.output.get_wrapup();
    if log::log_enabled!(log::Level::Trace) {
        log::trace!("Evaluated variables ...");
        for (key, variable, (confidence, value)) in &values {
            let sinking = if environment.settings.only_required
                && !environment.settings.required_keys.contains(key)
            {
                "output"
            } else {
                "!output"
            };
            log::trace!(
                "\t{:?}:{}:{}:{}='{}' ",
                key,
                variable.key(environment),
                confidence,
                sinking,
                &value
            );
        }
    }

    let sink_values = if environment.settings.only_required {
        values
            .into_iter()
            .filter(|val| environment.settings.required_keys.contains(&val.0))
            .collect()
    } else {
        values
    };

    for ref sink in sinks {
        log::trace!("Checking if sink {} is usable ...", sink);
        if sink.is_usable(environment) {
            log::trace!("Storing to sink {} ...", sink);
            sink.store(environment, &sink_values)?;
        }
    }

    log::trace!("Done.");

    Ok(())
}
