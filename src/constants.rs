// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::lazy_static::lazy_static;

pub const DEFAULT_KEY_PREFIX: &str = "PROJECT_";

pub const D_GIT_HUB_COM: &str = "github.com";
pub const D_GIT_HUB_COM_RAW: &str = "raw.githubusercontent.com";
pub const DS_GIT_HUB_IO_SUFIX: &str = "github.io";

pub const D_GIT_LAB_COM: &str = "gitlab.com";
pub const DS_GIT_LAB_IO_SUFIX: &str = "gitlab.io";

pub const D_BIT_BUCKET_ORG: &str = "bitbucket.org";

pub const D_GIT_SOURCE_HUT: &str = "git.sr.ht";

pub const D_REPO_OR_CZ: &str = "repo.or.cz";

pub const D_CODE_BERG_ORG: &str = "codeberg.org";

pub const D_SOURCE_FORGE_NET: &str = "sourceforge.net";

lazy_static! {
    pub static ref SPDX_IDENTS: Vec<&'static str> = ["CC0-1.0", "GPL-3.0-or-later", "GPL-3.0", "GPL-2.0-or-later", "GPL-2.0", "AGPL-3.0-or-later", "AGPL-3.0"].to_vec(); // TODO HACK ...
    // TODO use an SPDX repo as submodule that contains the list of supported license idenfiers and compare against them
    // TODO see: https://github.com/spdx/license-list-XML/issues/1335
}

pub const VALID_OS_FAMILIES: &[&str] = &["linux", "unix", "bsd", "osx", "windows"]; // TODO
pub const VALID_ARCHS: &[&str] = &["x86", "x86_64", "arm", "arm64"]; // TODO
