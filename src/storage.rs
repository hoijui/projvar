// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use clap::lazy_static::lazy_static;
use regex::Regex;
use strum::IntoEnumIterator;

use crate::{
    environment::Environment,
    sources::VarSource,
    var::{self, Confidence, Key, Variable},
};

/// Stores the property values gathered from all the sources.
#[derive(Clone)]
pub struct Storage {
    // key_values: HashMap<Key, Vec<(usize, String)>>,
    key_values: HashMap<Key, HashMap<usize, (Confidence, String)>>,
    key_primary: HashMap<Key, (Confidence, String)>,
}

impl Storage {
    /// Creates a new, empty instance of a storage.
    pub fn new() -> Storage {
        Storage {
            key_values: HashMap::new(),
            key_primary: HashMap::new(),
        }
    }

    /// Creates a table of all the keys (y) and all the sources (x),
    /// containing the currently stored values.
    /// It will be created in markdown format.
    // TODO further specify the markdown flavor in the sentence above.
    pub fn to_table(&self, environment: &Environment, sources: &[Box<dyn VarSource>]) -> String {
        lazy_static! {
            static ref R_COMMON_SOURCE_PREFIX: Regex = Regex::new(r"^projvar::sources::").unwrap();
            static ref R_COMMON_SOURCE_NAME: Regex = Regex::new(r"::VarSource").unwrap();
            static ref R_EMPTY_PROPERTIES: Regex = Regex::new(r"\[\]$").unwrap();
        }
        static HEADER_PREFIX: &str = "| Property | Env-Key |";
        static HEADER_SUFFIX: &str = " Final Value |";
        static SOURCE_NAME_ESTIMATE: usize = 32;
        // "| `Key::name()` | `variable.key` |"
        static CONTENT_LINE_PREFIX_EST: usize = 40;
        // " `$value` |" (this will often be empty)
        static CONTENT_LINE_PART_EST: usize = 10;
        // the estimated size of the table in chars
        let table_chars_estimate = (HEADER_PREFIX.len() + (sources.len() * (3 + SOURCE_NAME_ESTIMATE)) + 1) + // header
            (1 + (sources.len() * 6) + 1) + // header separator
            self.key_values.len() * (CONTENT_LINE_PREFIX_EST + sources.len() * CONTENT_LINE_PART_EST) + 1; // table content
        let mut table = String::with_capacity(table_chars_estimate);

        // header
        table.push_str(HEADER_PREFIX);
        for source in sources {
            let display = source.display();
            let display = R_COMMON_SOURCE_PREFIX.replace(&display, "");
            let display = R_COMMON_SOURCE_NAME.replace(&display, "");
            let display = R_EMPTY_PROPERTIES.replace(&display, "");
            table.push(' ');
            table.push_str(&display);
            table.push_str(" |");
        }
        table.push_str(HEADER_SUFFIX);
        table.push('\n');

        // header separator
        table.push('|');
        for _table_sep_index in 0..(sources.len() + 3) {
            table.push_str(" --- |");
        }
        table.push('\n');

        // table content (`Key::iter()` is sorted)
        for key in Key::iter() {
            let values = self.key_values.get(&key);
            if let Some(values) = values {
                let variable = var::get(key);
                table.push_str("| ");
                table.push_str(key.into());
                table.push_str(" | `");
                table.push_str(&variable.key(environment));
                table.push_str("` |");
                for source_index in 0..sources.len() {
                    table.push(' ');
                    table.push_str(values.get(&source_index).map_or("", |(_c, v)| v));
                    table.push_str(" |");
                }
                table.push_str(" **");
                table.push_str(self.get(key).map_or("", |(_c, v)| v));
                table.push_str("** |");
                table.push('\n');
            }
        }
        log::trace!("Table size (in chars), estimated: {}", table_chars_estimate);
        log::trace!("Table size (in chars), actual:    {}", table.len());
        table
    }

    /// Creates a list of all the keys,
    /// containing the currently stored values.
    /// It will be created in markdown format.
    pub fn to_list(&self, environment: &Environment) -> String {
        let values = self.get_wrapup();
        let mut key_strs: HashMap<Key, String> = HashMap::with_capacity(values.len());
        for (key, variable, _value) in &values {
            let key_str = variable.key(environment);
            key_strs.insert(*key, key_str.as_ref().to_owned());
        }
        // because the loop below adds 7 strings for each entry
        let mut list = Vec::with_capacity(values.len() * 7);
        for (key, _variable, (_confidence, value)) in &values {
            list.push("* ");
            list.push(key.into());
            list.push(" - `");
            list.push(&key_strs[key]);
            list.push("` - ");
            list.push(value);
            list.push("\n");
        }
        list.concat()
    }

    /// Returns the primary value associated to a specific key,
    /// if it is in store.
    pub fn get(&self, key: Key) -> Option<&(Confidence, String)> {
        // The last entry contains the value of the source
        // with the highest `sources::Hierarchy`
        // that provided a value at all.
        self.key_primary.get(&key)
        // .and_then(|entry| entry.last().map(|entry| &entry.1))
    }

    /// Returns all value by any source
    /// which is associated to the provided key.
    pub fn get_all(&self, key: Key) -> Option<&HashMap<usize, (Confidence, String)>> {
        self.key_values.get(&key)
    }

    /// Builds a sorted list of all the keys with associated values,
    /// their variable meta-data and the primary value.
    pub fn get_wrapup(&self) -> Vec<(Key, &'static Variable, &(Confidence, String))> {
        let mut wrapup: Vec<(Key, &'static Variable, &(Confidence, String))> = self
            .key_primary
            .iter()
            .map(|key_value| {
                let key = *key_value.0;
                let variable = var::get(*key_value.0);
                let value = key_value.1;
                (key, variable, value)
            })
            .collect();
        wrapup.sort_unstable_by_key(|entry| entry.0);
        wrapup
    }

    /// Adds the value found for a specific key by a certain source.
    pub fn add(&mut self, key: Key, source_index: usize, confidence: Confidence, value: String) {
        // ... PUH! :O
        // This returns the Vec for key,
        // or creates, inserts and returns a new one,
        // if none is present yet.
        // See: <https://stackoverflow.com/a/41418147>
        (*self.key_values.entry(key).or_insert_with(HashMap::new))
            .insert(source_index, (confidence, value.clone()));
        // here, the last to add, wins (should be the source with the highest hierarchy)
        self.key_primary.insert(key, (confidence, value));
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
