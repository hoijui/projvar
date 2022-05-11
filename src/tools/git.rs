// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use git2::{self, Repository};
use lazy_static::lazy_static;
use regex::Regex;
use std::convert::TryFrom;
use std::path::Path;
use std::path::PathBuf;
use std::str;
use thiserror::Error;

/// This enumerates all possible errors returned by this module.
/// Represents all other cases of `std::io::Error`.
#[derive(Error, Debug)]
#[error("Git2 lib error: {from} - {message}")]
pub struct Error {
    from: git2::Error,
    message: String,
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Error {
            from: git2::Error::from_str("PLACEHOLDER"),
            message: String::from(message),
        }
    }
}

/// The default date format.
/// For formatting specifiers, see:
/// <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
pub const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// Checks whether a given version string is a git dirty version.
/// Dirty means, there are uncommitted changes.
#[must_use]
pub fn is_git_dirty_version(vers: &str) -> bool {
    lazy_static! {
        // static ref R_DIRTY_VERSION: Regex = Regex::new(r"^.+(-broken)?-dirty(-.+)?$").unwrap();
        static ref R_DIRTY_VERSION: Regex = Regex::new(r"^[^-].+(-broken)?-dirty(-.+)?$").unwrap();
    }
    R_DIRTY_VERSION.is_match(vers)
}

///
/// "--tags", "--dirty", "--broken", "--always"
fn _version(repo: &git2::Repository) -> Result<String, Error> {
    // NOTE We might also want '--broken',
    //      which is not possible with git2-rs,
    //      but it is really not important
    repo.describe(
        git2::DescribeOptions::new()
            .pattern("*[0-9]*.[0-9]*.[0-9]*")
            .describe_tags(),
    )
    .map_err(|from| Error {
        from,
        message: String::from("Failed to describe the HEAD revision version"),
    })?
    .format(Some(
        git2::DescribeFormatOptions::new()
            .always_use_long_format(false)
            .dirty_suffix("-dirty"),
    ))
    .map_err(|from| Error {
        from,
        message: String::from("Failed to format the HEAD revision version"),
    })
}

pub struct Repo {
    repo: git2::Repository,
}

impl TryFrom<Option<&str>> for Repo {
    type Error = git2::Error;
    fn try_from(repo_root: Option<&str>) -> Result<Self, Self::Error> {
        let repo_root = repo_root.unwrap_or(".");
        Ok(Repo {
            repo: Repository::open(repo_root)?,
        })
    }
}

impl TryFrom<Option<&Path>> for Repo {
    type Error = git2::Error;
    fn try_from(repo_root: Option<&Path>) -> Result<Self, Self::Error> {
        let repo_root = repo_root.unwrap_or_else(|| Path::new("."));
        Ok(Repo {
            repo: Repository::open(repo_root)?,
        })
    }
}

impl Repo {
    // pub fn new(repo_root: Option<&str>) -> BoxResult<Repo> {
    //     let repo_root = repo_root.unwrap_or(".");
    //     Ok(Repo {
    //         repo: Repository::open(repo_root)?,
    //     })
    // }

    // pub fn new(repo_root: Option<&str>) -> BoxResult<Repo> {
    //     let repo_root = repo_root.unwrap_or(".");
    //     Ok(Repo {
    //         repo: Repository::open(repo_root)?,
    //     })
    // }

    #[must_use]
    pub fn inner(&self) -> &git2::Repository {
        &self.repo
    }

    /// Returns the path to the local repo.
    ///
    /// # Panics
    ///
    /// Should never happen
    #[must_use]
    pub fn local_path(&self) -> PathBuf {
        let path = self.repo.path().canonicalize().unwrap(); // We want this to panic, as it should never happen
        match path.file_name() {
            Some(file_name) => {
                if file_name.to_str().unwrap() == ".git" {
                    // This panics if not valid UTF-8
                    path.parent().unwrap().to_path_buf() // As we already know the parent is called ".git", this could never panic
                } else {
                    // let path_str = path as &str;
                    // (path.as_ref() as &Path).clone()
                    // Path::new(path_str)
                    path
                }
            }
            None => {
                // There is no file_name in the path, so it must be the root of the file-system
                Path::new("/").to_path_buf()
            }
        }
    }

    /// Returns the path to the local repo as string.
    ///
    /// # Panics
    ///
    /// Should never happen
    #[must_use]
    pub fn local_path_str(&self) -> String {
        self.local_path().to_str().unwrap().to_owned() // Can never hapen, as we already know fro mwithin local_path(), that it is valid UTF-8
    }

    fn _branch(&self) -> Result<git2::Branch, Error> {
        Ok(git2::Branch::wrap(self.repo.head().map_err(|from| {
            Error {
                from,
                message: String::from("Failed to convert HEAD into a branch"),
            }
        })?))
    }

    /// Returns the SHA of the currently checked-out commit,
    /// if any.
    //
    /// # Errors
    ///
    /// If some git-related magic goes south,
    /// or there is no commit.
    pub fn sha(&self) -> Result<Option<String>, Error> {
        let head = self.repo.head().map_err(|from| Error {
            from,
            message: String::from("Failed get repo HEAD for figuring out the SHA1"),
        })?;
        Ok(
            //Some(
            head.resolve()
                .map_err(|from| Error {
                    from,
                    message: String::from("Failed resolving HEAD into a direct reference"),
                })?
                .target()
                .map(|oid| oid.to_string()),
        ) //)
    }

    /// Returns the local name of the currently checked-out branch,
    /// if any.
    //
    /// # Errors
    ///
    /// If some git-related magic goes south,
    /// or the branch name is not valid UTF-8.
    pub fn branch(&self) -> Result<Option<String>, Error> {
        Ok(Some(
            self._branch()?
                .name()
                .map_err(|from| Error {
                    from,
                    message: String::from("Failed fetching name of a branch"),
                })?
                .ok_or_else(|| Error::from("Branch name is not UTF-8 compatible"))?
                .to_owned(),
        ))
    }

    fn _tag(&self) -> Result<Option<String>, Error> {
        let head = self.repo.head().map_err(|from| Error {
            from,
            message: String::from("Failed to get repo HEAD for figuring out the tag"),
        })?;
        let head_oid = head
            .resolve()
            .map_err(|from| Error {
                from,
                message: String::from("Failed resolve HEAD into a reference"),
            })?
            .target()
            .ok_or_else(|| git2::Error::from_str("No OID for HEAD"))
            .map_err(|from| Error {
                from,
                message: String::from("-"),
            })?;
        let mut tag = None;
        let mut inner_err: Option<Result<Option<String>, Error>> = None;
        self.repo
            .tag_foreach(|_id, name| {
                let name_str =
                    String::from_utf8(name.to_vec()).expect("Failed stringifying tag name");
                let cur_tag_res = self.repo.find_reference(&name_str).and_then(|git_ref| {
                    git_ref.target().ok_or_else(|| {
                        git2::Error::from_str("Failed to get tag reference target commit")
                    })
                });
                let cur_tag = match cur_tag_res {
                    Err(from) => {
                        inner_err = Some(Err(Error {
                            from,
                            message: String::from("Failed fetching current tag reference"),
                        }));
                        return false;
                    }
                    Ok(cur_tag) => cur_tag,
                };
                if cur_tag == head_oid {
                    tag = Some(name_str);
                    false
                } else {
                    true
                }
            })
            .map_err(|from| Error {
                from,
                message: String::from("Failed processing tags"),
            })?;
        match inner_err {
            Some(err) => err,
            None => Ok(tag),
        }
    }

    /// Returns the name of the currently checked-out tag,
    /// if any tag points to the current HEAD.
    //
    /// # Errors
    ///
    /// If some git-related magic goes south,
    /// or the tag name is not valid UTF-8.
    pub fn tag(&self) -> Result<Option<String>, Error> {
        self._tag()
    }

    fn _remote_tracking_branch(&self) -> Result<git2::Branch, Error> {
        self._branch()?.upstream().map_err(|from| Error {
            from,
            message: String::from("Failed resolving the remote tracking branch"),
        })
    }

    /// The local name of the remote tracking branch.
    //
    /// # Errors
    ///
    /// If some git-related magic goes south,
    /// or the remote name is not valid UTF-8.
    pub fn remote_tracking_branch(&self) -> Result<String, Error> {
        Ok(self
            ._remote_tracking_branch()?
            .name()
            .map_err(|from| Error {
                from,
                message: String::from("Failed fetching the remote tracking branch name"),
            })?
            .ok_or_else(|| Error::from("Remote tracking branch name is not UTF-8 compatible"))?
            .to_owned())
    }

    /// Local name of the main remote.
    //
    /// # Errors
    ///
    /// If some git-related magic goes south,
    /// or the reomte name is not valid UTF-8.
    pub fn remote_name(&self) -> Result<String, Error> {
        Ok(self
            .repo
            .branch_remote_name(
                self.repo
                    .resolve_reference_from_short_name(&self.remote_tracking_branch()?)
                    .map_err(|from| Error {
                        from,
                        message: String::from(
                            "Failed to resolve referenece from remotetracking branch short name",
                        ),
                    })?
                    .name()
                    .ok_or_else(|| Error::from("Remote branch name is not UTF-8 compatible"))?,
            )
            .map_err(|from| Error {
                from,
                message: String::from("Failed to get branch remote name"),
            })?
            .as_str()
            .ok_or_else(|| Error::from("Remote name is not UTF-8 compatible"))?
            .to_owned())
        // let remote = remote_tracking_branch.name(); // HACK Need to split of the name part, as this is probably origin/master, and we want only origin.
    }

    /// Returns the clone URL of the main remote,
    /// if there is any.
    //
    /// # Errors
    ///
    /// If some git-related magic goes south.
    pub fn remote_clone_url(&self) -> Result<String, Error> {
        Ok(self
            .repo
            .find_remote(&self.remote_name()?)
            .map_err(|from| Error {
                from,
                message: String::from("Failed to find remote name for remote clone URL"),
            })?
            .url()
            .ok_or_else(|| Error::from("Remote URL is not UTF-8 compatible"))?
            .to_owned())
    }

    /// Returns the version of the current state of the repo.
    /// This is basically the result of "git describe --tags --all <and-some-more...>".
    ///
    /// # Errors
    ///
    /// If some git-related magic goes south.
    pub fn version(&self) -> Result<String, Error> {
        _version(&self.repo)
    }

    /// Returns the commit-time (not author-time)
    /// of the last commit in the currently checked out history (=> HEAD)
    ///
    /// # Errors
    ///
    /// If some git-related magic goes south.
    pub fn commit_date(&self, date_format: &str) -> Result<String, Error> {
        let head = self.repo.head().map_err(|from| Error {
            from,
            message: String::from("Failed to get repo HEAD for figuring out the commit date"),
        })?;
        let commit_time_git2 = head
            .peel_to_commit()
            .map_err(|from| Error {
                from,
                message: String::from(
                    "Failed to peal HEAD to commit for figuring out the commit date",
                ),
            })?
            .time();
        let commit_time_chrono = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(commit_time_git2.seconds(), 0),
            Utc,
        );
        Ok(commit_time_chrono.format(date_format).to_string())
        // date.fromtimestamp(repo.head.ref.commit.committed_date).strftime(date_format)
    }
}

/*
#[cfg(test)]
mod tests {
    // Note this useful idiom:
    // importing names from outer (for mod tests) scope.
    use super::*;

    macro_rules! is_that_error {
        ($result:ident,$err_type:ident) => {
            $result.unwrap_err().downcast_ref::<$err_type>().is_some()
        };
    }

    #[test]
    fn test_is_git_dirty_version() {
        assert!(!is_git_dirty_version("0.2.2"));
        assert!(!is_git_dirty_version("0.2.2-0-gbe4cc26"));
        assert!(!is_git_dirty_version("dirty"));
        assert!(!is_git_dirty_version("-dirty"));
        assert!(!is_git_dirty_version("-dirty-broken"));
        assert!(!is_git_dirty_version("-broken-dirty"));
        assert!(is_git_dirty_version("0.2.2-0-gbe4cc26-dirty"));
        assert!(is_git_dirty_version("0.2.2-0-gbe4cc26-dirty-broken"));
    }

    #[test]
    fn test_web_to_build_hosting_url() {
        assert_eq!(
            web_to_build_hosting_url("https://gitlab.com/OSEGermany/OHS-3105/").unwrap(),
            "https://osegermany.gitlab.io/OHS-3105"
        );
        assert_eq!(
            web_to_build_hosting_url("https://github.com/hoijui/escher").unwrap(),
            "https://hoijui.github.io/escher"
        );

        let result = web_to_build_hosting_url("git@github.com:hoijui/escher.git");
        assert!(is_that_error!(result, UrlConversionError));
    }
}
*/
