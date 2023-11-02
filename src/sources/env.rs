// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::environment::Environment;
use crate::var;
use crate::var::Key;
use crate::var::C_HIGH;

use super::var;
use super::Hierarchy;
use super::RetrieveRes;

/// Sources from environment variables
/// with the same names as the those used for output.
/// We treat this as a way to (almost) preset certain output values,
/// which is both useful for testing
/// and streamlining the process during production use.
pub struct VarSource;

impl super::VarSource for VarSource {
    fn is_usable(&self, _environment: &mut Environment) -> bool {
        true
    }

    fn hierarchy(&self) -> Hierarchy {
        Hierarchy::Higher
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn properties(&self) -> &Vec<String> {
        &super::NO_PROPS
    }

    fn retrieve(&self, environment: &mut Environment, key: Key) -> RetrieveRes {
        Ok(var(environment, &var::get(key).key(environment), C_HIGH))
    }
}
