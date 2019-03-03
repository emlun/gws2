use ansi_term::ANSIString;

use std::path::Path;

use super::common::exit_codes;
use super::common::format_branch_line;
use super::common::format_message_line;
use super::common::format_project_header;
use super::common::Command;
use super::error::Error;
use color::palette::Palette;
use config::data::Project;
use config::data::Workspace;
use data::status::BranchStatus;
use data::status::DirtyState;
use data::status::ProjectStatusMethods;
use data::status::RepositoryStatus;
use data::status::WorkspaceStatus;

trait BranchStatusPrinting {
    fn describe_sync_status(&self, palette: &Palette) -> ANSIString;
    fn describe_status(&self, palette: &Palette) -> ANSIString;
    fn describe_full(&self, palette: &Palette) -> String;
}

impl BranchStatusPrinting for BranchStatus {
    fn describe_sync_status(&self, palette: &Palette) -> ANSIString {
        match &self.upstream_name {
            Some(upstream_name) => {
                if self.upstream_fetched {
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

pub struct Status {
    pub only_changes: bool,
}

impl Status {
    pub fn print_output(
        &self,
        project: &Project,
        project_status: &Result<RepositoryStatus, Error>,
        palette: &Palette,
    ) {
        println!("{}", format_project_header(project, palette));

        match project_status {
            Ok(status) => {
                if self.only_changes == false
                    || status.iter().any(|b| {
                        b.dirty != DirtyState::Clean
                            || b.in_sync.unwrap_or(true) == false
                            || b.upstream_fetched
                    })
                {
                    for b in status {
                        println!("{}", b.describe_full(&palette));
                    }
                }
            }
            Err(Error::RepositoryMissing) => {
                if self.only_changes == false {
                    println!(
                        "{}",
                        palette
                            .missing
                            .paint(format_message_line("Missing repository"))
                    );
                }
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

    pub fn make_report<'ws>(
        &self,
        working_dir: &Path,
        workspace: &'ws Workspace,
    ) -> WorkspaceStatus<'ws> {
        workspace
            .projects
            .iter()
            .map(|project| (project, project.status(working_dir)))
            .collect()
    }
}

impl Command for Status {
    fn run(
        &self,
        working_dir: &Path,
        workspace: &Workspace,
        palette: &Palette,
    ) -> Result<i32, Error> {
        let report = self.make_report(working_dir, workspace);

        for (project, project_result) in &report {
            self.print_output(project, project_result, palette);
        }

        let exit_code = report
            .values()
            .map(|project_result| match project_result {
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
}
