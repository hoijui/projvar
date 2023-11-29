// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod default;
pub mod sem_ver;
pub mod sem_ver_pref;

#[derive(thiserror::Error, Debug)]
pub enum RepoCreationError {
    #[error("Failed to figure out the cache dir for storing testing repos.")]
    Creating,

    #[error("Failed to (re-)initialize repo dir '{dir}'.")]
    Initializing { dir: String, source: std::io::Error },
}

macro_rules! hash_file {
    ($file:expr) => {{
        let content = include_str!($file);
        const_fnv1a_hash::fnv1a_hash_str_64(content)
    }};
}

macro_rules! create_repo {
    ($creation_fn:path, $creation_code_file:expr) => {{
        let proj_dirs = directories::ProjectDirs::from("org", "oseg", "projvar")
            .ok_or_else(|| crate::repo_creation::RepoCreationError::Creating)?;
        let repo_dir = proj_dirs
            .cache_dir()
            .join("testing")
            .join("repos")
            .join($creation_code_file);

        let code_hash_file = format!("{}.chechsum.txt", repo_dir.display());
        let code_hash_on_fs = std::fs::read_to_string(code_hash_file).ok();
        let code_hash_we_want = crate::repo_creation::hash_file!($creation_code_file);
        if code_hash_on_fs
            .map(|hash_str| hash_str.parse::<u64>().unwrap())
            .filter(|&code_hash_on_fs| code_hash_on_fs == code_hash_we_want)
            .is_none()
        {
            // crate::repo_creation::default::$creation_fn(&repo_dir)?;
            $creation_fn(&repo_dir)?;
        }
        Ok::<_, crate::repo_creation::RepoCreationError>(repo_dir)
    }};
}

macro_rules! create_repo_common {
    ($creation_code_module:ident) => {
        crate::repo_creation::create_repo!(
            crate::repo_creation::$creation_code_module::create,
            concat!("repo_creation/", stringify!($creation_code_module), ".rs")
        )
    };
}

pub(crate) use create_repo;

pub(crate) use hash_file;
