// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use cmd_lib::run_cmd;

use super::RepoCreationError;

pub fn create(repo_dir: &Path) -> Result<(), RepoCreationError> {
    let license_text = include_str!("../../LICENSE.txt");
    run_cmd! (
        // Re-creat ethe repo from scratch
        rm -Rf "$repo_dir";
        mkdir -p "$repo_dir";
        cd "$repo_dir";
        git init;

        // Create content
        touch "a.txt";
        mkdir "b"
        touch "b/c.txt";
        echo "$license_text" | tee "LICENSE.txt";
        mkdir -p "LICENSES";
        touch "LICENSES/AGPL-3.0-or-later.txt";
        touch "LICENSES/CC0-1.0.txt";
        touch "LICENSES/Unlicense.txt";

        // Add and commit all content
        git add -A;
        git commit -m "Initial commit";

        // Add a remote (without having to fetch -> tricky!)
        git remote add origin "https://github.com/hoijui/projvar.git";
        git config "branch.master.remote" "origin";
        git config "branch.master.merge" "refs/heads/master";
        mkdir -p ".git/refs/remotes/origin";
        git rev-parse HEAD | tee ".git/refs/remotes/origin/master";
    )
    .map_err(|err| RepoCreationError::Initializing {
        dir: repo_dir.display().to_string(),
        source: err,
    })?;

    Ok(())
}
