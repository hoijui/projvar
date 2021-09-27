// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// use dict::Dict;
// use std::io;
// use std::io::BufRead;
// use std::io::Write;
use crate::settings::{Settings, STUB};
// use crate::sinks::VarSink;
// use crate::sources::VarSource;
use crate::tools::git;
use crate::var::Key;
use std::collections::HashMap;
use std::convert::TryFrom;

// #[derive(Debug)]
// #[derive(TypedBuilder)]
// pub struct Environment<'t, S> where S: storage::Storage {
pub struct Environment<'t> {
    pub settings: &'t Settings,
    // pub vars: Box<dyn storage::Storage>,
    /// The input variables, as supplied by the environment,
    /// on the command line or through input files.
    pub vars: HashMap<String, String>,
    /// The output values we evaluated for the project properties we want to know.
    pub output: HashMap<Key, String>,
    // #[builder(default = false)]
    repo: Option<git::Repo>,
    // #[builder(default = false)]
    // verbose: bool,
    // pub sources: Vec<Box<dyn VarSource>>,
    // pub sinks: Vec<Box<dyn VarSink>>,
}

impl<'t> Environment<'t> {
    #[must_use]
    pub fn new(
        settings: &Settings, /*, sources: Vec<Box<dyn VarSource>>, sinks: Vec<Box<dyn VarSink>>*/
    ) -> Environment {
        // let vars: Box<dyn storage::Storage>  = Box::new(storage::InMemory::new());
        let vars = HashMap::<String, String>::new();
        let output = HashMap::<Key, String>::new();
        // match settings.storage {
        //     StorageMode::Environment => Box::new(storage::Env::new()),
        //     _ => Box::new(storage::InMemory::new()),
        // };
        Environment {
            settings,
            vars,
            output,
            repo: None,
            // sources,
            // sinks,
        }
    }

    #[must_use]
    pub fn stub() -> Environment<'static> {
        Self::new(&STUB)
    }

    pub fn repo(&mut self) -> Option<&git::Repo> {
        if self.repo.is_none() {
            // self.repo : git::Repo = git::Repo::try_from(self.settings.repo_path).ok();
            self.repo = git::Repo::try_from(self.settings.repo_path.as_deref()).ok();
        }
        // match &self.repo {
        //     Some(repo) => Some(repo),
        //     None => None
        // }
        self.repo.as_ref()
    }
}
