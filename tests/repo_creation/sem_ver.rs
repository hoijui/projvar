// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use cmd_lib::run_cmd;

use super::RepoCreationError;

/// This makes sure a sem-versioned tag is checked out.
pub fn create(repo_dir: &Path) -> Result<(), RepoCreationError> {
    super::default::create(repo_dir)?;
    let repo_dir_str = repo_dir.display();
    run_cmd! (
        cd "$repo_dir_str";
        git tag -a -m "This is release 0.0.1" "0.0.1";
        git checkout "0.0.1";
    )
    .map_err(|err| RepoCreationError::Initializing {
        dir: repo_dir.display().to_string(),
        source: err,
    })?;

    Ok(())
}
