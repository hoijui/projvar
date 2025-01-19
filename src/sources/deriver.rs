// SPDX-FileCopyrightText: 2021 - 2023 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use url::Url;

use crate::environment::Environment;
use crate::tools::git::TransferProtocol;
use crate::value_conversions;
use crate::var::Key;

use super::Hierarchy;
use super::RetrieveRes;

macro_rules! overwrite_guard {
    ($environment:ident, $out_key:ident, $fetching:expr) => {
        // Only generates a value if none was sourced so far
        match $environment.output.get($out_key) {
            None => $fetching,
            Some(_) => None,
        }
    };
}

macro_rules! conv_val {
    ($environment:ident, $in_key:ident, $out_key:ident, $conv_fun:ident $(,$extra_arg:expr)*) => {
        overwrite_guard!($environment, $out_key,
                // Does a lot of extracting and reinserting of value and confidence,
                // and mapping of Option to Result
                $environment
                .output
                .get(Key::$in_key)
                .and_then(|(confidence, in_val)| {
                    Some(
                        value_conversions::$conv_fun(in_val, $($extra_arg),*)
                            .map(|val_opt| val_opt.map(|val| (*confidence, val))),
                    )
                })
                .unwrap_or(Ok(None))?
        )
    }
}

macro_rules! conv_val_with_env {
    ($environment:ident, $in_key:ident, $out_key:ident, $conv_fun:ident $(,$extra_arg:expr)*) => {
        overwrite_guard!($environment, $out_key,
                // Does a lot of extracting and reinserting of value and confidence,
                // and mapping of Option to Result
                $environment
                .output
                .get(Key::$in_key)
                .and_then(|(confidence, in_val)| {
                    Some(
                        value_conversions::$conv_fun($environment, in_val, $($extra_arg),*)
                            .map(|val_opt| val_opt.map(|val| (*confidence, val))),
                    )
                })
                .unwrap_or(Ok(None))?
        )
    }
}
// pub(crate) use conv_val_with_env;

/// Does not source any new values,
/// but derives them from other values, already sourced before.
/// For example, it might derive the [`Key::RepoWebUrl`]
/// from the [`Key::RepoCloneUrl`].
pub struct VarSource;

fn name_machine_readable(environment: &Environment) -> RetrieveRes {
    let key = Key::NameMachineReadable;
    let from_name = conv_val_with_env!(environment, Name, key, name_to_machine_readable_name);
    Ok(if from_name.is_some() {
        from_name
    } else {
        conv_val_with_env!(
            environment,
            RepoWebUrl,
            key,
            web_url_to_machine_readable_name
        )
    })
}

fn repo_clone_url_specific(environment: &Environment, protocol: TransferProtocol) -> RetrieveRes {
    let key = protocol.to_clone_url_key();
    let from_web_url =
        conv_val_with_env!(environment, RepoWebUrl, key, web_url_to_clone_url, protocol);
    Ok(match from_web_url {
        Some(_) => from_web_url,
        None => {
            conv_val!(
                environment,
                RepoCloneUrl,
                key,
                clone_url_conversion,
                environment,
                protocol
            )
        }
    }
    .and_then(|conf_and_val| {
        // NOTE We successfully derived the requested clone URL now,
        //      but many hosts do not support all protocols,
        //      so this might be a phony one.
        //      In such a case, we will not return it.
        let url_str = &conf_and_val.1;
        let url = Url::parse(url_str)
            .expect("If we get here, it should always be a valid URL, ensured by previous code.");
        let hosting_type = environment.settings.hosting_type(&url);
        if hosting_type.supports_clone_url(protocol) {
            Some(conf_and_val)
        } else {
            None
        }
    }))
}

impl super::VarSource for VarSource {
    fn is_usable(&self, _environment: &mut Environment) -> bool {
        true
    }

    fn hierarchy(&self) -> Hierarchy {
        Hierarchy::Top
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn properties(&self) -> &Vec<String> {
        &super::NO_PROPS
    }

    #[remain::check]
    fn retrieve(&self, environment: &mut Environment, key: Key) -> RetrieveRes {
        Ok(
            #[remain::sorted]
            match key {
                Key::BuildArch
                | Key::BuildBranch
                | Key::BuildDate
                | Key::BuildNumber
                | Key::BuildOsFamily
                | Key::BuildOs
                | Key::BuildTag
                | Key::Ci
                | Key::License
                | Key::Licenses
                | Key::Version
                | Key::VersionDate => None,
                Key::BuildHostingUrl => {
                    conv_val_with_env!(environment, RepoWebUrl, key, web_url_to_build_hosting_url)
                }
                Key::Name => overwrite_guard!(
                    environment,
                    key,
                    environment.output.get(Key::NameMachineReadable).cloned()
                ),
                Key::NameMachineReadable => name_machine_readable(environment)?,
                Key::RepoCloneUrl => conv_val_with_env!(
                    environment,
                    RepoWebUrl,
                    key,
                    web_url_to_clone_url,
                    TransferProtocol::Https
                ),
                Key::RepoCloneUrlGit => {
                    repo_clone_url_specific(environment, TransferProtocol::Git)?
                }
                Key::RepoCloneUrlHttp => {
                    repo_clone_url_specific(environment, TransferProtocol::Https)?
                }
                Key::RepoCloneUrlSsh => {
                    repo_clone_url_specific(environment, TransferProtocol::Ssh)?
                }
                Key::RepoCommitPrefixUrl => {
                    conv_val_with_env!(environment, RepoWebUrl, key, web_url_to_commit_prefix_url)
                }
                Key::RepoIssuesUrl => {
                    conv_val_with_env!(environment, RepoWebUrl, key, web_url_to_issues_url)
                }
                Key::RepoRawVersionedPrefixUrl => {
                    conv_val_with_env!(environment, RepoWebUrl, key, web_url_to_raw_prefix_url)
                }
                Key::RepoVersionedDirPrefixUrl => {
                    conv_val_with_env!(
                        environment,
                        RepoWebUrl,
                        key,
                        web_url_to_versioned_dir_prefix_url
                    )
                }
                Key::RepoVersionedFilePrefixUrl => conv_val_with_env!(
                    environment,
                    RepoWebUrl,
                    key,
                    web_url_to_versioned_file_prefix_url
                ),
                Key::RepoWebUrl => {
                    let from_http_clone_url =
                        conv_val_with_env!(environment, RepoCloneUrl, key, clone_url_to_web_url);
                    if from_http_clone_url.is_some() {
                        from_http_clone_url
                    } else {
                        conv_val_with_env!(environment, RepoCloneUrlSsh, key, clone_url_to_web_url)
                    }
                }
            },
        )
    }
}
