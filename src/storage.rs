// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::error::Error;
// use std::fmt::Display;
use std::fmt;

use crate::var::Key;

pub type Result<T> = std::result::Result<T, VarError>;
pub trait ClonableError: Error + Clone {}

#[derive(Debug, Clone)]
enum VarErrorType {
    Get,
    Set,
}

#[derive(Debug, Clone)]
pub struct VarError {
    key: String,
    kind: VarErrorType,
    // source: Option<env::VarError>,
}

impl Error for VarError {
    //     fn source(&self) -> Option<&(dyn Error + 'static)> {
    //         self.source.map_or_else(|| None, |val| Some(val))
    //     }
}

impl fmt::Display for VarError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            VarErrorType::Get => write!(f, "No value stored for key '{}'", self.key),
            VarErrorType::Set => write!(f, "Unable to store value for key '{}'", self.key),
        }
    }
}

pub trait Storage<K> {
    // /// This will be called before any of the other methods.
    // fn init(&mut self) {}

    // /// This will be called after any of the other methods.
    // fn finalize(&mut self) {}

    /// Returns the value associated to a specific key,
    /// if it is in store.
    ///
    /// # Errors
    ///
    /// If there was any problem accessing the underlying storage.
    fn get(&self, key: &K) -> Result<Option<String>>;

    /// Sets the value for a specific key.
    ///
    /// # Errors
    ///
    /// If there was any problem accessing the underlying storage.
    fn set(&mut self, key: &K, value: &str) -> Result<()>;
}

pub struct Env {}

impl Env {
    #[must_use]
    pub fn new() -> Env {
        Env {}
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

// impl Storage for Env {
//     fn get(&self, key: &Key) -> Result<Option<String>> {
//         env::var(key).map_or_else(
//             |err| {
//                 match err {
//                     env::VarError::NotPresent => Ok(None),
//                     env::VarError::NotUnicode(_) => Err(VarError {
//                         key: key.to_string(),
//                         kind: VarErrorType::Get, /*, source: Some(err)*/
//                     }),
//                 }
//             },
//             |val| Ok(Some(val)),
//         )
//     }

//     fn set(&mut self, key: &Key, value: &str) -> Result<()> {
//         env::set_var(key, value);
//         Ok(())
//     }
// }

pub struct InMemory {
    pub vars: HashMap<Key, String>,
}

impl InMemory {
    #[must_use]
    pub fn new() -> InMemory {
        InMemory {
            vars: HashMap::<Key, String>::new(),
        }
    }

    // pub fn load_env(&mut self) {
    //     repvar::tools::append_env(&mut self.vars);
    // }

    // pub fn store_to_env(&self) {
    //     repvar::tools::flush_to_env(&self.vars, true);
    // }
}

impl Default for InMemory {
    fn default() -> Self {
        Self::new()
    }
}

// impl Storage for InMemory {
//     // fn init(&mut self) {
//     //     self.load_env();
//     // }

//     // fn finalize(&mut self) {
//     //     self.store_to_env();
//     // }

//     fn get(&self, key: &Key) -> Result<Option<String>> {
//         Ok(self.vars.get(key).map(std::borrow::ToOwned::to_owned))
//     }

//     fn set(&mut self, key: &Key, value: &str) -> Result<()> {
//         self.vars.insert(*key, value.to_string());
//         Ok(())
//     }
// }
