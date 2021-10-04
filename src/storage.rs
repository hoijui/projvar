// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;

use crate::{
    sources::VarSource,
    var::{self, Key, Variable},
};

/// Stores the property values gathered from all the sources.
#[derive(Clone)]
pub struct Storage {
    // key_values: HashMap<Key, Vec<(usize, String)>>,
    key_values: HashMap<Key, HashMap<usize, String>>,
    key_primary: HashMap<Key, String>,
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
    pub fn to_table(&self, sources: &Vec<Box<dyn VarSource>>) -> String {
        let mut table = Vec::with_capacity(self.key_values.len() * 7 + 1); // because the loob below adds 7 strings for each entry
        table.push("| Property | Env-Key | ");
        let mut source_ids = Vec::with_capacity(sources.len());
        for source in sources {
            source_ids.push(source.display());
        }
        for source_index in 0..sources.len() {
            table.push(&source_ids[source_index]);
            table.push(" |");
        }
        for (key, values) in &self.key_values {
            let variable = var::get(*key);
            table.push("| ");
            table.push(key.into());
            table.push(" | ");
            table.push(variable.key);
            table.push(" | ");
            for source_index in 0..sources.len() {
                table.push(values.get(&source_index).map_or("", |v| &v));
                table.push("\n");
            }
        }
        table.concat()
    }

    /// Creates a list of all the keys,
    /// containing the currently stored values.
    /// It will be created in markdown format.
    pub fn to_list(&self) -> String {
        let values = self.get_wrapup();
        let mut list = Vec::with_capacity(values.len() * 7); // because the loob below adds 7 strings for each entry
        for (key, variable, value) in values {
            list.push("* ");
            list.push(key.into());
            list.push(" - ");
            list.push(variable.key);
            list.push(" - ");
            list.push(value);
            list.push("\n");
        }
        list.concat()
    }

    /// Returns the primary value associated to a specific key,
    /// if it is in store.
    pub fn get(&self, key: Key) -> Option<&String> {
        // The last entry contains the value of the source
        // with the highest `sources::Hierarchy`
        // that provided a value at all.
        self.key_primary.get(&key)
        // .and_then(|entry| entry.last().map(|entry| &entry.1))
    }

    /// Returns all value by any source
    /// which is associated to the provided key.
    pub fn get_all(&self, key: Key) -> Option<&HashMap<usize, String>> {
        self.key_values.get(&key)
    }

    /// Builds a list of all the keys with associated values,
    /// their variable meta-data and the primary value.
    pub fn get_wrapup(&self) -> Vec<(Key, &'static Variable, &String)> {
        self.key_primary
            .iter()
            .map(|key_value| {
                let key = *key_value.0;
                let variable = var::get(*key_value.0);
                let value = key_value.1;
                (key, variable, value)
            })
            .collect()
    }

    /// Adds the value found for a specific key by a certain source.
    pub fn add(&mut self, key: Key, source_index: usize, value: String) {
        // ... PUH! :O
        // This returns the Vec for key,
        // or creates, inserts and returns a new one,
        // if none is present yet.
        // See: <https://stackoverflow.com/a/41418147>
        (*self.key_values.entry(key).or_insert_with(HashMap::new)).insert(source_index, value);
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
