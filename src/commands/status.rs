use ansi_term::ANSIString;

use std::path::Path;

use color::palette::Palette;
use config::data::Project;
use config::data::Workspace;
use data::status::BranchStatus;
use data::status::DirtyState;
use data::status::ProjectStatusMethods;
use super::common::Command;
use super::common::exit_codes;
use super::common::format_branch_line;
use super::common::format_message_line;
use super::common::format_project_header;
use super::error::Error;


trait BranchStatusPrinting {
  fn describe_sync_status(&self, palette: &Palette) -> ANSIString;
  fn describe_status(&self, palette: &Palette) -> ANSIString;
  fn describe_full(&self, palette: &Palette) -> String;
}

impl BranchStatusPrinting for BranchStatus {
  fn describe_sync_status(&self, palette: &Palette) -> ANSIString {
    match self.in_sync {
      Some(true) => palette.clean.paint("Clean".to_string()),
      Some(false) => palette.dirty.paint(format!("Not in sync with {}", self.upstream_name)),
      None => palette.missing.paint(format!("No remote branch {}", self.upstream_name)),
    }
  }

  fn describe_status(&self, palette: &Palette) -> ANSIString {
    if self.is_head {
      match self.dirty {
        DirtyState::Clean => self.describe_sync_status(palette),
        DirtyState::UncommittedChanges => palette.dirty.paint("Dirty (Uncommitted changes)".to_string()),
        DirtyState::UntrackedFiles => palette.dirty.paint("Dirty (Untracked files)".to_string()),
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
      &self.describe_status(&palette)
    )
  }
}

pub struct Status {
  pub only_changes: bool,
}

impl Status {
  fn run_project(&self, working_dir: &Path, project: &Project, palette: &Palette) -> i32 {
    match project.status(working_dir) {
      Some(Ok(status)) => {
        if self.only_changes == false || status.iter()
          .any(|b|
            b.dirty != DirtyState::Clean
              || b.in_sync.unwrap_or(true) == false
          )
        {
          println!("{}", format_project_header(&project, &palette));

          for b in status {
            println!("{}", b.describe_full(&palette));
          }
        }
        exit_codes::OK
      },
      Some(Err(err)) => {
        println!("{}", format_project_header(&project, &palette));
        eprintln!("{}", palette.error.paint(format!("Failed to compute status: {}", err)));
        exit_codes::STATUS_PROJECT_FAILED
      }
      None => {
        if self.only_changes == false {
          println!("{}", format_project_header(&project, &palette));
          println!("{}", palette.missing.paint(format_message_line("Missing repository")));
        }
        exit_codes::OK
      },
    }
  }
}

impl Command for Status {
  fn run(&self, working_dir: &Path, workspace: &Workspace, palette: &Palette) -> Result<i32, Error> {
    Ok(
      workspace.projects
        .iter()
        .map(|project|
          self.run_project(working_dir, project, palette)
        )
        .fold(
          exit_codes::OK,
          |exit_code, next_code| {
            if next_code != exit_codes::OK {
              next_code
            } else {
              exit_code
            }
          }
        )
    )
  }
}
