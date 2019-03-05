pub mod exit_codes;

use std::path::Path;

use ansi_term::ANSIString;

use super::error::Error;
use color::palette::Palette;
use config::data::Project;
use config::data::Workspace;
use data::status::BranchStatus;
use data::status::DirtyState;
use data::status::RepositoryStatus;

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
    fn run(
        &self,
        working_dir: &Path,
        workspace: &Workspace,
        palette: &Palette,
    ) -> Result<i32, Error>;
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
