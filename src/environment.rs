// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::settings::{Settings, STUB};
use crate::storage::Storage;
use crate::tools::git;
use std::collections::HashMap;
use std::convert::TryFrom;

pub struct Environment {
    pub settings: Settings,
    /// The input variables, as supplied by the environment,
    /// on the command line or through input files.
    pub vars: HashMap<String, String>,
    /// The output values we evaluated for the project properties we want to know.
    pub output: Storage,
    pub repo: Option<git::Repo>,
}

impl Environment {
    #[must_use]
    pub fn new(settings: Settings) -> Environment {
        let vars = HashMap::<String, String>::new();
        let output = Storage::new();
        let repo = git::Repo::try_from(settings.repo_path.as_deref()).ok();
        Environment {
            settings,
            vars,
            output,
            repo,
        }
    }

    #[must_use]
    pub fn stub() -> Environment {
        Self::new(STUB.clone())
    }

    #[must_use]
    pub fn repo(&self) -> Option<&git::Repo> {
        // TODO DEPRECATED Just use the repo property directly, instead
        self.repo.as_ref()
    }
}
