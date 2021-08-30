// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// use dict::Dict;
// use std::io;
// use std::io::BufRead;
// use std::io::Write;
use crate::git;
use crate::settings::{Settings, StorageMode};
use crate::storage;
use std::convert::TryFrom;

// #[derive(Debug)]
// #[derive(TypedBuilder)]
// pub struct Environment<'t, S> where S: storage::Storage {
pub struct Environment<'t> {
    pub settings: &'t Settings,
    pub vars: Box<dyn storage::Storage>,
    // #[builder(default = false)]
    repo: Option<git::Repo>,
    // #[builder(default = false)]
    // verbose: bool,
}

impl<'t> Environment<'t> {
    #[must_use]
    pub fn new(settings: &Settings) -> Environment {
        let vars: Box<dyn storage::Storage> = match settings.storage {
            StorageMode::Environment => Box::new(storage::Env::new()),
            _ => Box::new(storage::InMemory::new()),
        };
        Environment {
            settings,
            vars,
            repo: None,
        }
    }

    pub fn repo(&mut self) -> Option<&git::Repo> {
        if self.repo.is_none() {
            // self.repo : git::Repo = git::Repo::try_from(self.settings.repo_path).ok();
            self.repo = git::Repo::try_from(self.settings.repo_path).ok();
        }
        // match &self.repo {
        //     Some(repo) => Some(repo),
        //     None => None
        // }
        self.repo.as_ref()
    }
}
