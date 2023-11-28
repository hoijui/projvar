// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use clap::ValueEnum;
/// This deals with things related to different git hosting providers,
/// both the actual hosters (github.com, gitlab.com, bitbucket.org, ...),
/// as well as the software (gitlab, gitea, ...).
use std::str;
use strum_macros::{EnumString, EnumVariantNames, IntoStaticStr};
use url::Host;

use crate::constants;

use super::git::TransferProtocol;

#[derive(Debug, EnumString, EnumVariantNames, IntoStaticStr, PartialEq, Eq, Clone, Copy)]
pub enum PublicSite {
    /// <https://github.com> - commercial, free OS hosting, software is proprietary
    GitHubCom,
    /// <https://gitlab.com> - commercial, free OS hosting, software is OSS
    GitLabCom,
    /// <https://bitbucket.org> - commercial, free OS hosting, software is proprietary
    BitBucketOrg,
    /// <https://git.sr.ht> - free OS hosting, software is OSS: SourceHut
    SourceHut,
    /// <https://codeberg.org> - hosts only OS, software is OSS: Gitea
    CodeBergOrg,
    /// <https://repo.or.cz> - hosts only OS, software is OSS: Girocco
    RepoOrCz,
    /// <https://sourceforge.net> - hosts only OS, software is OSS: Allura
    RocketGitCom,
    /// <https://rocketgit.com> - hosts only OS, software is OSS: RocketGit
    SourceForgeNet,
    /// NOTE: The rust masters said, this is better then Option<None>!
    Unknown,
}

impl PublicSite {
    #[must_use]
    pub fn from_hosting_domain(host: &Host<&str>) -> Self {
        if let Host::Domain(domain) = host {
            let domain_parts = domain.split('.').collect::<Vec<&str>>();
            let main_domain = domain_parts[domain_parts.len() - 2..].join(".");
            match main_domain.as_str() {
                constants::DS_GIT_HUB_IO_SUFIX => Self::GitHubCom,
                constants::DS_GIT_LAB_IO_SUFIX => Self::GitLabCom,
                _ => Self::Unknown, // TODO implement the rest, where applicable (BitBucket does not have a hosting site, for example)
            }
        } else {
            Self::Unknown
        }
    }

    #[must_use]
    pub fn from_hosting_domain_option(host: Option<&Host<&str>>) -> Self {
        host.map_or(Self::Unknown, Self::from_hosting_domain)
    }
}

impl Default for PublicSite {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<Host<&str>> for PublicSite {
    fn from(host: Host<&str>) -> Self {
        match host {
            Host::Domain(
                constants::D_GIT_HUB_COM
                | constants::DS_GIT_HUB_IO_SUFIX
                | constants::D_GIT_HUB_COM_RAW,
            ) => Self::GitHubCom,
            Host::Domain(constants::D_GIT_LAB_COM | constants::DS_GIT_LAB_IO_SUFIX) => {
                Self::GitLabCom
            }
            Host::Domain(constants::D_BIT_BUCKET_ORG) => Self::BitBucketOrg,
            Host::Domain(constants::D_GIT_SOURCE_HUT) => Self::SourceHut,
            Host::Domain(constants::D_REPO_OR_CZ) => Self::RepoOrCz,
            Host::Domain(
                constants::D_ROCKET_GIT_COM
                | constants::D_SSH_ROCKET_GIT_COM
                | constants::D_GIT_ROCKET_GIT_COM,
            ) => Self::RocketGitCom,
            Host::Domain(constants::D_CODE_BERG_ORG | constants::DS_CODE_BERG_PAGE) => {
                Self::CodeBergOrg
            }
            Host::Domain(constants::D_SOURCE_FORGE_NET | constants::DS_SOURCE_FORGE_IO) => {
                Self::SourceForgeNet
            }
            Host::Domain(_) | Host::Ipv4(_) | Host::Ipv6(_) => Self::Unknown,
        }
    }
}

impl From<Option<Host<&str>>> for PublicSite {
    fn from(host: Option<Host<&str>>) -> Self {
        host.map_or_else(|| Self::Unknown, Self::from)
    }
}

#[derive(
    Debug, ValueEnum, EnumString, EnumVariantNames, IntoStaticStr, PartialEq, Eq, Clone, Copy,
)]
pub enum HostingType {
    /// <https://github.com> - proprietary
    GitHub,
    /// <https://about.gitlab.com> - OSS
    GitLab,
    /// <https://bitbucket.org> - proprietary
    BitBucket,
    /// <https://sr.ht/~sircmpwn/sourcehut> - OSS - LowTech (no JS) hacker tool, (almost) suckless style
    SourceHut,
    /// <https://gitea.io> - OSS
    Gitea,
    /// <https://repo.or.cz/girocco.git> - OSS
    Girocco,
    /// <https://rocketgit.com> - OSS
    RocketGit,
    /// <https://allura.apache.org> - OSS
    Allura,
    /// NOTE: The rust masters said, this is better then Option<None>!
    Unknown,
}

impl HostingType {
    #[must_use]
    pub const fn is_oss(self) -> bool {
        match self {
            Self::GitHub | Self::BitBucket | Self::Unknown => false,
            Self::GitLab
            | Self::SourceHut
            | Self::Gitea
            | Self::Girocco
            | Self::RocketGit
            | Self::Allura => true,
        }
    }

    #[must_use]
    pub const fn supports_clone_url(self, protocol: TransferProtocol) -> bool {
        match protocol {
            TransferProtocol::Https | TransferProtocol::Ssh => true,
            TransferProtocol::Git => match self {
                Self::Girocco | Self::RocketGit => true,
                Self::GitHub
                | Self::BitBucket
                | Self::Unknown
                | Self::GitLab
                | Self::SourceHut
                | Self::Gitea
                | Self::Allura => false,
            },
        }
    }

    #[must_use]
    pub const fn def_ssh_user(self) -> &'static str {
        match self {
            Self::GitHub | Self::GitLab | Self::BitBucket | Self::SourceHut => "git@",
            Self::Gitea | Self::Girocco | Self::Allura | Self::Unknown => "",
            Self::RocketGit => "rocketgit@",
        }
    }
}

impl Default for HostingType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<PublicSite> for HostingType {
    fn from(site: PublicSite) -> Self {
        match site {
            PublicSite::GitHubCom => Self::GitHub,
            PublicSite::GitLabCom => Self::GitLab,
            PublicSite::BitBucketOrg => Self::BitBucket,
            PublicSite::SourceHut => Self::SourceHut,
            PublicSite::RepoOrCz => Self::Girocco,
            PublicSite::RocketGitCom => Self::RocketGit,
            PublicSite::CodeBergOrg => Self::Gitea,
            PublicSite::SourceForgeNet => Self::Allura,
            PublicSite::Unknown => Self::Unknown,
        }
    }
}
