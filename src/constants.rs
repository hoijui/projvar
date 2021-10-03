// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use clap::lazy_static::lazy_static;
use url::Host;

pub const S_GIT_HUB_IO_SUFIX: &str = ".github.io";
pub const S_GIT_LAB_IO_SUFIX: &str = ".gitlab.io";
pub const S_GIT_HUB_COM: &str = "github.com";
pub const S_GIT_HUB_COM_RAW: &str = "raw.githubusercontent.com";
pub const S_GIT_LAB_COM: &str = "gitlab.com";
pub const S_BIT_BUCKET_ORG: &str = "bitbucket.org";

pub const D_GIT_HUB_COM: Host<&'static str> = Host::Domain(S_GIT_HUB_COM);
pub const D_GIT_HUB_COM_RAW: Host<&'static str> = Host::Domain(S_GIT_HUB_COM_RAW);
pub const D_GIT_LAB_COM: Host<&'static str> = Host::Domain(S_GIT_LAB_COM);
pub const D_BIT_BUCKET_ORG: Host<&'static str> = Host::Domain(S_BIT_BUCKET_ORG);

lazy_static! {
    pub static ref SPDX_IDENTS: Vec<&'static str> = ["CC0-1.0", "GPL-3.0-or-later", "GPL-3.0", "GPL-2.0-or-later", "GPL-2.0", "AGPL-3.0-or-later", "AGPL-3.0"].to_vec(); // TODO HACK ...
    // TODO use an SPDX repo as submodule that contains the list of supported license idenfiers and compare against them
    // TODO see: https://github.com/spdx/license-list-XML/issues/1335
}

pub const VALID_OS_FAMILIES: &[&str] = &["linux", "unix", "bsd", "osx", "windows"]; // TODO
pub const VALID_ARCHS: &[&str] = &["x86", "x86_64", "arm", "arm64"]; // TODO
