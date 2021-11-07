// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::settings::{Settings, STUB};
use crate::storage::Storage;
use crate::tools::git;
use std::collections::HashMap;
use std::convert::TryFrom;

pub struct Environment<'t> {
    pub settings: &'t Settings,
    /// The input variables, as supplied by the environment,
    /// on the command line or through input files.
    pub vars: HashMap<String, String>,
    /// The output values we evaluated for the project properties we want to know.
    pub output: Storage,
    repo: Option<git::Repo>,
}

impl<'t> Environment<'t> {
    #[must_use]
    pub fn new(settings: &Settings) -> Environment {
        let vars = HashMap::<String, String>::new();
        let output = Storage::new();
        Environment {
            settings,
            vars,
            output,
            repo: None,
        }
    }

    #[must_use]
    pub fn stub() -> Environment<'static> {
        Self::new(&STUB)
    }

    pub fn repo(&mut self) -> Option<&git::Repo> {
        if self.repo.is_none() {
            self.repo = git::Repo::try_from(self.settings.repo_path.as_deref()).ok();
        }
        self.repo.as_ref()
    }
}
