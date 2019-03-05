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

pub fn format_branch_line(
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

trait BranchStatusPrinting {
    fn describe_sync_status(&self, palette: &Palette) -> ANSIString;
    fn describe_status(&self, palette: &Palette) -> ANSIString;
    fn describe_full(&self, palette: &Palette) -> String;
}

impl BranchStatusPrinting for BranchStatus {
    fn describe_sync_status(&self, palette: &Palette) -> ANSIString {
        match &self.upstream_name {
            Some(upstream_name) => {
                if self.fast_forwarded {
                    palette.cloning.paint("Fast-forwarded")
                } else if self.upstream_fetched {
                    palette.cloning.paint("New upstream commits")
                } else {
                    match self.in_sync {
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

    fn describe_status(&self, palette: &Palette) -> ANSIString {
        if self.is_head {
            match self.dirty {
                DirtyState::Clean => self.describe_sync_status(palette),
                DirtyState::UncommittedChanges => palette
                    .dirty
                    .paint("Dirty (Uncommitted changes)".to_string()),
                DirtyState::UntrackedFiles => {
                    palette.dirty.paint("Dirty (Untracked files)".to_string())
                }
            }
        } else {
            self.describe_sync_status(palette)
        }
    }

    fn describe_full(&self, palette: &Palette) -> String {
        format_branch_line(
            &palette,
            self.is_head,
            &self.name,
            &self.describe_status(&palette),
        )
    }
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
                println!("{}", b.describe_full(&palette));
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
