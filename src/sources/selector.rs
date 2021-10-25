// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::environment::Environment;
use crate::validator;
use crate::var::Confidence;
use crate::var::Key;

use super::Hierarchy;
use super::RetrieveRes;

use std::convert::TryFrom;

/// Does not source any new values,
/// but selects out of the previously sourced ones those with highest validity
/// (= the return of validation function),
/// and if the validity is equal,
/// those with higher confidence.
pub struct VarSource;

fn to_confidence(validity: &validator::Result) -> u8 {
    match validity {
        Ok(warning_opt) => match warning_opt {
            None => 255,
            Some(warning) => match warning {
                validator::Warning::Missing => 150,
                validator::Warning::SuboptimalValue { msg: _, value: _ } => 200,
                validator::Warning::Unknown { value: _ } => 140,
            },
        },
        Err(error) => match error {
            validator::Error::Missing => 40,
            validator::Error::AlmostUsableValue { msg: _, value: _ } => 100,
            validator::Error::BadValue { msg: _, value: _ } => 50,
            validator::Error::IO(_) => 30,
        },
    }
}

fn source_index_to_confidence(source_index: usize) -> u8 {
    u8::try_from(source_index).unwrap_or_else(|_err| {
        log::warn!("Sorting during value selection has a small chance to be imprecise because more then {} sources (at least {}) are in use.", u8::MAX, source_index + 1);
        u8::MAX
    })
}

fn confidence_combiner(
    most_important: u8,
    highly_important: u8,
    less_important: u8,
    least_important: u8,
) -> u32 {
    let mut conf_comb: u32 = 0;
    for conf in [
        most_important,
        highly_important,
        less_important,
        least_important,
    ] {
        conf_comb = (conf_comb * u32::from(u8::MAX)) + u32::from(conf);
    }
    conf_comb
}

fn valor(validity: &validator::Result, confidence: Confidence, source_index: usize) -> u32 {
    confidence_combiner(
        0,
        to_confidence(validity),
        confidence,
        source_index_to_confidence(source_index),
    )
}

impl super::VarSource for VarSource {
    fn is_usable(&self, _environment: &mut Environment) -> bool {
        true
    }

    fn hierarchy(&self) -> Hierarchy {
        Hierarchy::EvenHigher
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<VarSource>()
    }

    fn properties(&self) -> &Vec<String> {
        &super::NO_PROPS
    }

    fn retrieve(&self, environment: &mut Environment, key: Key) -> RetrieveRes {
        let values = &environment.output.get_all(key);
        Ok(match values {
            Some(values) => {
                let mut enriched_values = vec![];
                for (src_index, (confidence, value)) in (*values).clone() {
                    let specific_validator = validator::get(key);
                    let validity = specific_validator(environment, &value);
                    enriched_values.push((src_index, (confidence, value), validity));
                }
                enriched_values.sort_by_cached_key(|entry| valor(&entry.2, entry.1 .0, entry.0));
                enriched_values.last().map(|entry| entry.1.clone())
            }
            None => None,
        })
    }
}
