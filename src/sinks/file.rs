// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::environment::Environment;
use crate::var::{self, Key, Variable};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::LineWriter;
use std::io::Write;
use std::path::PathBuf;

pub struct VarSink {
    pub file: PathBuf,
}

type BoxResult<T> = Result<T, Box<dyn Error>>;

/// Stores evaluated values (output) into a file
/// in a BASH compatible way ("KEY=VALUE\n").
impl super::VarSink for VarSink {
    fn is_usable(&self, _environment: &Environment) -> bool {
        true
    }

    fn store(
        &self,
        environment: &Environment,
        values: &[(Key, &Variable, &String)],
    ) -> BoxResult<()> {
        let previous_vars = if self.file.exists() {
            var::parse_vars_file_reader(repvar::tools::create_input_reader(self.file.to_str())?)?
        } else {
            HashMap::new()
        };

        let file = File::create(self.file.as_path())?;
        let mut file = LineWriter::new(file);
        let mut output_values: Vec<(&str, &&String)> = values
            .iter()
            .map(|(_key, var, value)| (var.key, value))
            .collect();
        output_values.sort();
        for (key, value) in output_values {
            if environment.settings.overwrite.main() || previous_vars.contains_key(key) {
                file.write_fmt(format_args!("{}=\"{}\"\n", key, value))?;
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
