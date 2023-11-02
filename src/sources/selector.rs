// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

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

fn source_index_to_confidence(source_index: usize) -> u8 {
    u8::try_from(source_index).unwrap_or_else(|_err| {
        log::warn!(
            "Sorting during value selection has a small chance to be imprecise, \
            because more then {} sources (at least {}) are in use.",
            u8::MAX,
            source_index + 1
        );
        u8::MAX
    })
}

fn valor(validity: &validator::Result, confidence: Confidence, source_index: usize) -> [u8; 4] {
    let res_confs = validator::res_to_confidences(validity);
    [
        res_confs[0],
        res_confs[1],
        confidence,
        source_index_to_confidence(source_index),
    ]
}

impl super::VarSource for VarSource {
    fn is_usable(&self, _environment: &mut Environment) -> bool {
        true
    }

    fn hierarchy(&self) -> Hierarchy {
        Hierarchy::EvenHigher
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
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
                enriched_values.sort_by_cached_key(|entry| {
                    let valor = valor(&entry.2, entry.1 .0, entry.0);
                    log::trace!("Valor evaluated for {:?} from source {}, value '{}' is {:?}.",
                        key,
                        entry.0,
                        entry.1.1,
                        valor
                    );
                    log::trace!("    ... evaluated from (validity, confidence, source_index): ({:?}, {}, {})",
                        &entry.2,
                        entry.1 .0,
                        entry.0
                    );
                    valor
                });
                enriched_values.last().map(|entry| entry.1.clone())
            }
            None => None,
        })
    }
}
