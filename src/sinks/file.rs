// SPDX-FileCopyrightText: 2021-2023 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::environment::Environment;
use crate::var::{self, Confidence};
use crate::{storage, BoxResult};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::LineWriter;
use std::io::Write;
use std::path::PathBuf;

pub struct VarSink {
    pub file: PathBuf,
}

/// Stores evaluated values (output) into a file
/// in a BASH compatible way ("KEY=VALUE\n").
impl super::VarSink for VarSink {
    fn is_usable(&self, _environment: &Environment) -> bool {
        true
    }

    fn store(&self, environment: &Environment, values: &[storage::Value]) -> BoxResult<()> {
        log::trace!(
            "Reading previous values from ENV file (if it exists): '{}' ...",
            self.file.display()
        );
        let previous_vars = if self.file.exists() {
            var::parse_vars_file_reader(cli_utils::create_input_reader(Some(&self.file))?)?
        } else {
            HashMap::new()
        };

        log::trace!("Prepare and sort new/generated values ...");
        let mut output_values: Vec<(Cow<str>, &&(Confidence, String))> = values
            .iter()
            .map(|(_key, var, rated_value)| (var.key(environment), rated_value))
            .collect();
        output_values.sort();

        log::trace!(
            "Combine and write combined vars to ENV file: '{}' ...",
            self.file.display()
        );
        let file = File::create(self.file.as_path())?;
        let mut file = LineWriter::new(file);
        for (key, rated_value) in output_values {
            if environment.settings.overwrite.main() || !previous_vars.contains_key(key.as_ref()) {
                file.write_fmt(format_args!("{key}=\"{}\"\n", rated_value.1))?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for VarSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}(file: {})",
            std::any::type_name::<VarSink>(),
            self.file.as_path().to_str().ok_or(fmt::Error {})?
        )
    }
}
