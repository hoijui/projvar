// SPDX-FileCopyrightText: 2022-2023 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::environment::Environment;
use crate::{storage, BoxResult};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub struct VarSink {
    pub file: PathBuf,
}

/// Extends the first map with the keys and values of the second one.
/// In other words:
/// The resulting map will contain all keys of both maps.
/// If one key appears in both maps,
/// the value of the first one is used.
fn extend<K: std::cmp::Eq + std::hash::Hash, V>(
    first: HashMap<K, V>,
    second: HashMap<K, V>,
) -> HashMap<K, V> {
    second.into_iter().chain(first).collect()
}

/// Stores evaluated values (output) into a JSON file.
impl super::VarSink for VarSink {
    fn is_usable(&self, _environment: &Environment) -> bool {
        true
    }

    fn store(&self, environment: &Environment, values: &[storage::Value]) -> BoxResult<()> {
        log::trace!(
            "Reading previous values from JSON file (if it exists): '{}' ...",
            self.file.display()
        );
        let previous_vars: HashMap<String, String> = if self.file.exists() {
            let mut content = String::new();
            cli_utils::create_input_reader(Some(&self.file))?.read_to_string(&mut content)?;
            serde_json::from_str(&content)?
        } else {
            HashMap::new()
        };

        log::trace!("Combine new/generated vars with previous ones (if any) ...");
        let new_values: HashMap<String, String> = values
            .iter()
            .map(|(_, var, (_, val))| (var.key_raw().to_string(), val.clone()))
            .collect();
        let combined_values: HashMap<String, String> = if environment.settings.overwrite.main() {
            extend(new_values, previous_vars)
        } else {
            extend(previous_vars, new_values)
        };

        log::trace!("Convert combined vars to JSON ...");
        let json = serde_json::to_string(&combined_values)?;

        log::trace!(
            "Write combined vars to JSON file: '{}' ...",
            self.file.display()
        );
        let mut file = File::create(self.file.as_path())?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }
}

impl fmt::Display for VarSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}(file: {})",
            std::any::type_name::<Self>(),
            self.file.as_path().to_str().ok_or(fmt::Error {})?
        )
    }
}
