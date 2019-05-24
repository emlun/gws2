pub mod exit_codes;

use std::collections::HashSet;
use std::path::Path;

use ansi_term::ANSIString;

use super::error::Error;
use color::palette::Palette;
use config::data::Project;
use config::data::Workspace;
use data::status::BranchStatus;
use data::status::DirtyState;
use data::status::RepositoryStatus;
use data::status::WorkspaceStatus;

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
                Err(_) => exit_codes::STATUS_PROJECT_FAILED,
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

fn format_branch_line(
    palette: &Palette,
    is_head: bool,
    name: &str,
    description: &ANSIString,
) -> String {
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

fn describe_status<'a>(status: &'a BranchStatus, palette: &Palette) -> ANSIString<'a> {
    if status.is_head {
        match status.dirty {
            DirtyState::Clean => describe_sync_status(status, palette),
            DirtyState::UncommittedChanges => palette
                .dirty
                .paint("Dirty (Uncommitted changes)".to_string()),
            DirtyState::UntrackedFiles => {
                palette.dirty.paint("Dirty (Untracked files)".to_string())
            }
        }
    } else {
        describe_sync_status(status, palette)
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
