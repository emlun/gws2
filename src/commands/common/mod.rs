pub mod exit_codes;

use std::collections::HashSet;
use std::path::Path;

use ansi_term::ANSIString;

use super::error::Error;
use crate::color::palette::Palette;
use crate::config::data::Project;
use crate::config::data::Workspace;
use crate::data::status::BranchStatus;
use crate::data::status::DirtyState;
use crate::data::status::RepositoryStatus;
use crate::data::status::WorkspaceStatus;

pub enum Command {
    DirectoryCommand(Box<DirectoryCommand>),
    RepositoryCommand(Box<RepositoryCommand>),
}

pub trait DirectoryCommand {
    fn run(
        &self,
        working_dir: &Path,
        workspace: &Workspace,
        palette: &Palette,
    ) -> Result<i32, Error>;
}

pub trait RepositoryCommand {
    fn only_changes(&self) -> bool;

    fn project_args(&self) -> &HashSet<String>;

    fn run(
        &self,
        working_dir: &Path,
        workspace: &Workspace,
        palette: &Palette,
    ) -> Result<i32, Error> {
        let reports = self.make_report_and_maybe_print(working_dir, workspace, Some(palette));

        let exit_code = reports
            .iter()
            .map(|(_, project_result)| match project_result {
                Ok(_) => exit_codes::OK,
                Err(Error::RepositoryMissing) => exit_codes::OK,
                Err(_) => exit_codes::INTERNAL_ERROR,
            })
            .fold(exit_codes::OK, |exit_code, next_code| {
                if next_code != exit_codes::OK {
                    next_code
                } else {
                    exit_code
                }
            });
        Ok(exit_code)
    }

    fn make_report_and_maybe_print<'ws>(
        &self,
        working_dir: &Path,
        workspace: &'ws Workspace,
        palette: Option<&Palette>,
    ) -> WorkspaceStatus<'ws> {
        workspace
            .projects
            .iter()
            .filter(|project| {
                self.project_args().is_empty() || self.project_args().contains(&project.path)
            })
            .map(|project| {
                let project_path: &Path = &working_dir.join(&project.path);
                (
                    project,
                    if project_path.exists() {
                        git2::Repository::open(project_path)
                            .map_err(Error::from)
                            .and_then(|repo| self.run_project(project, &repo))
                    } else {
                        Err(Error::RepositoryMissing)
                    },
                )
            })
            .filter(|(_, status_result)| {
                self.only_changes() == false
                    || status_result
                        .as_ref()
                        .map(|status| {
                            status.iter().any(|b| !b.is_clean())
                                || status.iter().all(|b| b.upstream_name == None)
                        })
                        .unwrap_or(false)
            })
            .map(|(project, status)| {
                for p in palette {
                    print_status(project, &status, p);
                }
                (project, status)
            })
            .collect()
    }

    fn make_report<'ws>(
        &self,
        working_dir: &Path,
        workspace: &'ws Workspace,
    ) -> WorkspaceStatus<'ws> {
        self.make_report_and_maybe_print(working_dir, workspace, None)
    }

    fn run_project(
        &self,
        project: &Project,
        repository: &git2::Repository,
    ) -> Result<RepositoryStatus, Error>;
}

fn ellipsisize(s: &str, length: usize) -> String {
    if s.len() >= length {
        format!("{}…", &s[0..(length - 1)])
    } else {
        s.to_string()
    }
}

fn format_branch_line(palette: &Palette, is_head: bool, name: &str, description: &str) -> String {
    format!(
        "  {} {} {}",
        if is_head { "*" } else { " " },
        palette
            .branch
            .paint(format!("{: <25}", format!("{} :", ellipsisize(name, 23)))),
        description
    )
}

pub fn format_message_line(message: &str) -> String {
    format!("{: <30 }{}", "", message)
}

pub fn format_project_header(project: &Project, palette: &Palette) -> String {
    format!("{}:", palette.repo.paint(project.path.clone()))
}

fn describe_sync_status<'a>(status: &'a BranchStatus, palette: &Palette) -> ANSIString<'a> {
    match &status.upstream_name {
        Some(upstream_name) => {
            if status.fast_forwarded {
                palette.cloning.paint("Fast-forwarded")
            } else if status.upstream_fetched {
                palette.cloning.paint("New upstream commits")
            } else {
                match status.in_sync {
                    Some(true) => palette.clean.paint("Clean".to_string()),
                    Some(false) => palette
                        .dirty
                        .paint(format!("Not in sync with {}", upstream_name)),
                    None => palette
                        .missing
                        .paint(format!("No remote branch {}", upstream_name)),
                }
            }
        }
        None => palette.missing.paint("No upstream set"),
    }
}

fn describe_status(status: &BranchStatus, palette: &Palette) -> String {
    if status.is_head {
        let fetch_prefix = if status.upstream_fetched {
            format!("{} - ", palette.cloning.paint("New upstream commits"))
        } else {
            "".to_string()
        };

        match status.dirty {
            DirtyState::Clean => describe_sync_status(status, palette).to_string(),
            DirtyState::UncommittedChanges => format!(
                "{}{}",
                fetch_prefix,
                palette
                    .dirty
                    .paint("Dirty (Uncommitted changes)".to_string()),
            ),
            DirtyState::UntrackedFiles => format!(
                "{}{}",
                fetch_prefix,
                palette.dirty.paint("Dirty (Untracked files)".to_string()),
            ),
        }
    } else {
        describe_sync_status(status, palette).to_string()
    }
}

fn describe_full(status: &BranchStatus, palette: &Palette) -> String {
    format_branch_line(
        &palette,
        status.is_head,
        &status.name,
        &describe_status(status, &palette),
    )
}

pub fn print_status(
    project: &Project,
    project_status: &Result<RepositoryStatus, Error>,
    palette: &Palette,
) {
    println!("{}", format_project_header(project, palette));

    match project_status {
        Ok(status) => {
            for b in status {
                println!("{}", describe_full(b, &palette));
            }
        }
        Err(Error::RepositoryMissing) => {
            println!(
                "{}",
                palette
                    .missing
                    .paint(format_message_line("Missing repository"))
            );
        }
        Err(Error::Git2Error(err)) => {
            eprintln!("Failed to open repository: {}", err);
            println!("{}", palette.error.paint(format_message_line("Error")));
        }
        Err(err) => {
            eprintln!("Failed to list branches: {}", err);
            println!(
                "{}",
                palette
                    .error
                    .paint(format!("Failed to compute status: {}", err))
            );
        }
    }
}

pub fn get_repobuilder<'a>() -> git2::build::RepoBuilder<'a> {
    use git2::CredentialType;

    let mut result = git2::build::RepoBuilder::new();
    let mut fopts = git2::FetchOptions::new();
    let mut callbacks = git2::RemoteCallbacks::new();
    let mut tried_ssh = false;
    let mut tried_password = false;

    callbacks.credentials(
        move |url: &str, username: Option<&str>, allowed: CredentialType| {
            let mut cred_helper = git2::CredentialHelper::new(url);
            cred_helper.config(&git2::Config::open_default()?);

            if allowed.contains(CredentialType::SSH_KEY) && !tried_ssh {
                let user: String = username
                    .map(&str::to_string)
                    .or_else(|| cred_helper.username.clone())
                    .unwrap_or("git".to_string());
                tried_ssh = true;
                git2::Cred::ssh_key_from_agent(&user)
            } else if allowed.contains(CredentialType::USER_PASS_PLAINTEXT) && !tried_password {
                let res =
                    git2::Cred::credential_helper(&git2::Config::open_default()?, url, username);
                tried_password = true;
                res
            } else {
                git2::Cred::default()
            }
        },
    );
    fopts.remote_callbacks(callbacks);
    result.fetch_options(fopts);
    result
}

// Copied from git2::Repository
//
// Copyright (c) 2014 Alex Crichton
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
pub fn update_submodules(repo: &git2::Repository) -> Result<(), git2::Error> {
    fn add_subrepos(
        repo: &git2::Repository,
        list: &mut Vec<git2::Repository>,
    ) -> Result<(), git2::Error> {
        for mut subm in repo.submodules()? {
            subm.update(true, None)?;
            list.push(subm.open()?);
        }
        Ok(())
    }

    let mut repos = Vec::new();
    add_subrepos(repo, &mut repos)?;
    while let Some(repo) = repos.pop() {
        add_subrepos(&repo, &mut repos)?;
    }
    Ok(())
}
