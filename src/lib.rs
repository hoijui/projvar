// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

extern crate clap;
extern crate enum_map;
extern crate log;
extern crate remain;
extern crate url;

mod constants;
pub mod environment;
mod license;
pub mod process;
pub mod settings;
pub mod sinks;
pub mod sources;
mod std_error;
mod storage;
pub mod tools;
pub mod validator;
pub mod value_conversions;
pub mod var;
