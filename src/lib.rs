// SPDX-FileCopyrightText: 2021 - 2023 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

mod cleanup;
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

use git_version::git_version;

pub const VERSION: &str = git_version!(cargo_prefix = "", fallback = "unknown");
