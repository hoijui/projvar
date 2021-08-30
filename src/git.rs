// SPDX-FileCopyrightText: 2021 Robin Vobruba <hoijui.quaero@gmail.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// use std::process::Command;
// use crate::environment::Environment;
use chrono::DateTime;
use std::convert::TryFrom;
use std::fmt;
use std::path::PathBuf;
// use chrono::Local;
use chrono::NaiveDateTime;
use chrono::Utc;
use clap::lazy_static::lazy_static;
use git2::{self, Repository};
use regex::Regex;
// use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::str;
// use crate::props::version;

type BoxResult<T> = Result<T, Box<dyn Error>>;
// type Git2Result<T> = Result<T, git2::Error>;

/// The default date format.
/// For formatting specifiers, see:
/// <https://docs.rs/chrono/latest/chrono/format/strftime/index.html>
pub const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug)]
pub struct UrlConversionError {
    details: String,
}

impl UrlConversionError {
    fn new(msg: &str) -> UrlConversionError {
        UrlConversionError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for UrlConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for UrlConversionError {
    fn description(&self) -> &str {
        &self.details
    }
}

/// Converts a common git remote URL
/// into a web-ready (http(s)) URL of the project.
///
/// for example:
///
/// `git@github.com:hoijui/kicad-text-injector.git`
/// ->
/// `https://github.com/hoijui/kicad-text-injector`
#[must_use]
pub fn clone_to_web_url(url: &str) -> String {
    lazy_static! {
        static ref R_GIT_AT: Regex = Regex::new(r"^git@").unwrap();
        static ref R_DOT_COM: Regex = Regex::new(r"\.com:").unwrap();
        static ref R_DOT_GIT: Regex = Regex::new(r"\.git$").unwrap();
    }
    let public_url = R_GIT_AT.replace(url, "https://");
    let public_url = R_DOT_COM.replace(&public_url, ".com/");
    let public_url = R_DOT_GIT.replace(&public_url, "");
    public_url.into_owned()
}

/// Converts a common git repo web-host URL
/// into the URL of where to find hosted CI output
/// (commonly known as "pages" URL).
///
/// NOTE: This will likely only work for github.com and gitlab.com!
///
/// for example:
///
/// <https://gitlab.com/OSEGermany/OHS-3105/> -> <https://osegermany.gitlab.io/OHS-3105/>
/// <https://github.com/hoijui/escher> -> <https://hoijui.github.io/escher/>
///
/// # Errors
///
/// Failed fetching/generating the Web URL.
///
/// Failed generating the "pages" URL,
/// likely because the remote is neither "github.com" nor "gitlab.com".
pub fn web_to_build_hosting_url(web_url: &str) -> BoxResult<String> {
    lazy_static! {
        static ref R_WEB_URL: Regex = Regex::new(r"(?P<protocol>[0-9a-zA-Z_-]+)://(?P<server>[0-9a-zA-Z_-]+)\.com/(?P<user>[^/]+)/(?P<project>[^/]+)/?").unwrap();
    }
    // let build_hosting_url = R_WEB_URL.replace(url, "${protocol}://${user}.${server}.io/${project}/");
    let build_hosting_url = R_WEB_URL.replace(web_url, |caps: &regex::Captures| {
        format!(
            "{protocol}://{user}.{server}.io/{project}/",
            protocol = &caps[1],
            server = &caps[2],
            user = &caps[3].to_lowercase(),
            project = &caps[4]
        )
    });
    if build_hosting_url == web_url {
        // Err(Box::new(Err("")))
        Err(Box::new(UrlConversionError::new(&format!("Not a supported hosting platform for converting repo web hosting URL to \"pages\" URL: '{}'", web_url))))
    } else {
        Ok(build_hosting_url.into_owned())
    }
}

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
fn _version(repo: &git2::Repository) -> BoxResult<String> {
    // NOTE We might also want '--broken',
    //      which is not possible with git2-rs,
    //      but it is really not important
    Ok(repo
        .describe(git2::DescribeOptions::new().describe_tags())?
        .format(Some(
            git2::DescribeFormatOptions::new()
                .always_use_long_format(true)
                .dirty_suffix("-dirty"),
        ))?)
}

pub struct Repo {
    repo: git2::Repository,
}

impl TryFrom<Option<&str>> for Repo {
    // type Error = Box<&'static str>;
    type Error = git2::Error;
    fn try_from(repo_root: Option<&str>) -> Result<Self, Self::Error> {
        let repo_root = repo_root.unwrap_or(".");
        Ok(Repo {
            repo: Repository::open(repo_root)?,
        })
    }
}

impl TryFrom<Option<&Path>> for Repo {
    // type Error = Box<&'static str>;
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

    fn _branch(&self) -> BoxResult<git2::Branch> {
        // self.repo.head()?.is_branch()
        Ok(git2::Branch::wrap(self.repo.head()?))
    }

    /// Returns the local name of the currently checked-out branch,
    /// if any.
    //
    /// # Errors
    ///
    /// If some git-related magic goes south,
    /// or the branch name is not valid UTF-8.
    pub fn branch(&self) -> BoxResult<Option<String>> {
        Ok(Some(
            self._branch()?
                // .map(|branch|
                // branch
                .name()?
                .ok_or_else(|| git2::Error::from_str("Branch name is not UTF-8 compatible"))?
                .to_owned(),
        ))
    }

    fn _tag(&self) -> BoxResult<Option<git2::Tag>> {
        let head = self.repo.head()?;
        let head_oid = head
            .resolve()?
            .target()
            .ok_or_else(|| git2::Error::from_str("No OID for HEAD"))?;
        let mut tag = None;
        let mut inner_err: Option<BoxResult<Option<git2::Tag>>> = None;
        self.repo.tag_foreach(|id, _name| {
            let cur_tag_res = self.repo.find_tag(id);
            let cur_tag = match cur_tag_res {
                Err(err) => {
                    inner_err = Some(Err(Box::new(err)));
                    return false;
                }
                Ok(cur_tag) => cur_tag,
            };
            if cur_tag.target_id() == head_oid {
                tag = Some(cur_tag);
                false
            } else {
                true
            }
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
    pub fn tag(&self) -> BoxResult<Option<String>> {
        Ok(match self._tag()? {
            Some(tag) => tag.name().map(std::borrow::ToOwned::to_owned),
            None => None,
        })

        // Ok(self._tag()?.map(|tag: git2::Tag| -> BoxResult<Option<String>> {
        //     // .ok_or_else(|| git2::Error::from_str("No tag on HEAD"))?
        //     Ok(Some(
        //         tag.name().map(std::borrow::ToOwned)
        //             .ok_or_else(|| git2::Error::from_str("Tag name is not UTF-8 compatible"))?,
        //     ))
        // })?)
    }

    fn _remote_tracking_branch(&self) -> BoxResult<git2::Branch> {
        Ok(self._branch()?.upstream()?)
    }

    /// The local name of the remote tracking branch.
    //
    /// # Errors
    ///
    /// If some git-related magic goes south,
    /// or the remote name is not valid UTF-8.
    pub fn remote_tracking_branch(&self) -> BoxResult<String> {
        Ok(self
            ._remote_tracking_branch()?
            .name()?
            .ok_or_else(|| git2::Error::from_str("Remote branch name is not UTF-8 compatible"))?
            .to_owned())
    }

    /// Local name of the main remote.
    //
    /// # Errors
    ///
    /// If some git-related magic goes south,
    /// or the reomte name is not valid UTF-8.
    pub fn remote_name(&self) -> BoxResult<String> {
        Ok(self
            .repo
            .branch_remote_name(
                self.repo
                    .resolve_reference_from_short_name(&self.remote_tracking_branch()?)?
                    .name()
                    .ok_or_else(|| {
                        git2::Error::from_str("Remote branch name is not UTF-8 compatible")
                    })?,
            )?
            .as_str()
            .ok_or_else(|| git2::Error::from_str("Remote name is not UTF-8 compatible"))?
            .to_owned())
        // let remote = remote_tracking_branch.name(); // HACK Need to split of the name part, as this is probably origin/master, and we want only origin.
    }

    /// Returns the clone URL of the main remote,
    /// if there is any.
    //
    /// # Errors
    ///
    /// If some git-related magic goes south.
    pub fn remote_clone_url(&self) -> BoxResult<String> {
        Ok(self
            .repo
            .find_remote(&self.remote_name()?)?
            .url()
            .ok_or_else(|| git2::Error::from_str("Remote URL is not UTF-8 compatible"))?
            .to_owned())
    }

    /// Returns the web-URL of the projects repository.
    /// You may not rely 100% on the retunred value,
    /// as it just performes some simple substitution on the primary clone-URL,
    /// which works well for GitHub.com and GitLab.com,
    /// but might not work for other platforms.
    /// Some platforms might not have a web frontend for the repo at all.
    ///
    /// # Errors
    ///
    /// If some git-related magic goes south.
    pub fn remote_web_url(&self) -> BoxResult<String> {
        let clone_url = self.remote_clone_url()?;
        if clone_url.starts_with("https://") {
            Ok(clone_url)
        } else {
            Ok(clone_to_web_url(&clone_url))
        }
    }

    ///  Returns a generated build/CI output hosting
    /// (commonly known as "pages") URL.
    ///
    /// NOTE: This will likely only work for github.com and gitlab.com!
    ///
    /// for example:
    ///
    /// <https://osegermany.gitlab.io/OHS-3105/>
    /// <https://hoijui.github.io/escher/>
    ///
    /// # Errors
    ///
    /// Failed fetching/generating the Web URL.
    ///
    /// Failed generating the "pages" URL,
    /// likely because the remote is neither "github.com" nor "gitlab.com".
    pub fn build_hosting_url(&self) -> BoxResult<String> {
        let web_url = self.remote_clone_url()?;
        web_to_build_hosting_url(&web_url)
    }

    /// Returns the version of the current state of the repo.
    /// This is basically the result of "git describe --tags --all <and-some-more...>".
    ///
    /// # Errors
    ///
    /// If some git-related magic goes south.
    pub fn version(&self) -> BoxResult<String> {
        _version(&self.repo)
    }

    /// Returns the commit-time (not author-time)
    /// of the last commit in the currently checked out history (=> HEAD)
    ///
    /// # Errors
    ///
    /// If some git-related magic goes south.
    pub fn commit_date(&self, date_format: &str) -> BoxResult<String> {
        let head = self.repo.head()?;
        let commit_time_git2 = head.peel_to_commit()?.time();
        let commit_time_chrono = DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(commit_time_git2.seconds(), 0),
            Utc,
        );
        Ok(commit_time_chrono.format(date_format).to_string())
        // date.fromtimestamp(repo.head.ref.commit.committed_date).strftime(date_format)
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom:
    // importing names from outer (for mod tests) scope.
    use super::*;

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
            "https://osegermany.gitlab.io/OHS-3105/"
        );
        assert_eq!(
            web_to_build_hosting_url("https://github.com/hoijui/escher").unwrap(),
            "https://hoijui.github.io/escher/"
        );
    }
}
