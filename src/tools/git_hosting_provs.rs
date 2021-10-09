// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

/// This deals with things related to different git hosting providers,
/// both the actual hosters (github.com, gitlab.com, bitbucket.org, ...),
/// as well as the software (gitlab, gitea, ...).
use std::str;
use strum_macros::{EnumString, EnumVariantNames, IntoStaticStr};
use url::Host;

use crate::constants;

#[derive(Debug, EnumString, EnumVariantNames, IntoStaticStr, PartialEq, Eq, Clone, Copy)]
pub enum PublicSite {
    /// https://github.com - commercial, free OS hosting, software is proprietary
    GitHubCom,
    /// https://gitlab.com - commercial, free OS hosting, software is OSS
    GitLabCom,
    /// https://bitbucket.org - commercial, free OS hosting, software is proprietary
    BitBucketOrg,
    /// https://git.sr.ht - free OS hosting, software is OSS: SourceHut
    SourceHut,
    /// https://codeberg.org - hosts only OS, software is OSS: Gitea
    CodeBergOrg,
    /// https://repo.or.cz - hosts only OS, software is OSS: Girocco
    RepoOrCz,
    /// https://sourceforge.net - hosts only OS, software is OSS: Allura
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
            Host::Domain(constants::D_GIT_HUB_COM) => Self::GitHubCom,
            Host::Domain(constants::D_GIT_LAB_COM) => Self::GitLabCom,
            Host::Domain(constants::D_BIT_BUCKET_ORG) => Self::BitBucketOrg,
            Host::Domain(constants::D_GIT_SOURCE_HUT) => Self::SourceHut,
            Host::Domain(constants::D_REPO_OR_CZ) => Self::RepoOrCz,
            Host::Domain(constants::D_CODE_BERG_ORG) => Self::CodeBergOrg,
            Host::Domain(constants::D_SOURCE_FORGE_NET) => Self::SourceForgeNet,
            Host::Domain(_) | Host::Ipv4(_) | Host::Ipv6(_) => Self::Unknown,
        }
    }
}

impl From<Option<Host<&str>>> for PublicSite {
    fn from(host: Option<Host<&str>>) -> Self {
        match host {
            Some(host) => Self::from(host),
            None => Self::Unknown,
        }
    }
}

#[derive(Debug, EnumString, EnumVariantNames, IntoStaticStr, PartialEq, Eq, Clone, Copy)]
pub enum HostingType {
    /// proprietary
    GitHub,
    /// https://about.gitlab.com - OSS
    GitLab,
    /// proprietary
    BitBucket,
    /// https://sr.ht/~sircmpwn/sourcehut - OSS - LowTech (no JS) hacker tool, (almost) suckless style
    SourceHut,
    /// https://gitea.io - OSS
    Gitea,
    /// https://repo.or.cz/girocco.git - OSS
    Girocco,
    /// https://rocketgit.com - OSS
    RocketGit,
    /// https://allura.apache.org - OSS
    Allura,
    /// NOTE: The rust masters said, this is better then Option<None>!
    Unknown,
}

impl HostingType {
    #[must_use]
    pub fn is_oss(self) -> bool {
        match self {
            HostingType::GitHub | HostingType::BitBucket | HostingType::Unknown => false,
            HostingType::GitLab
            | HostingType::SourceHut
            | HostingType::Gitea
            | HostingType::Girocco
            | HostingType::RocketGit
            | HostingType::Allura => true,
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
            PublicSite::CodeBergOrg => Self::Gitea,
            PublicSite::SourceForgeNet => Self::Allura,
            PublicSite::Unknown => Self::Unknown,
        }
    }
}
