use std::path::Path;

use ansi_term::ANSIString;

use cli::exit_codes;
use color::palette::Palette;
use config::read::read_workspace_file;
use data::status::BranchStatus;
use data::status::DirtyState;
use data::status::ProjectStatusMethods;
use super::common::print_project_header;


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
        format!(
            "{} {} {}",
            if self.is_head { "*" } else { " " },
            palette.branch.paint(format!("{: <25}", format!("{} :", ellipsisize(&self.name, 23)))),
            self.describe_status(&palette)
        )
    }
}

fn ellipsisize(s: &str, length: usize) -> String {
    if s.len() >= length {
        format!("{}…", &s[0..(length - 1)]).to_string()
    } else {
        s.to_string()
    }
}

pub fn run(palette: &Palette) -> Result<i32, ::git2::Error> {

    let ws_file_path = Path::new(".projects.gws");
    let ws = read_workspace_file(ws_file_path).unwrap();

    let mut exit_code: exit_codes::ExitCode = exit_codes::OK;

    for project in ws.projects {
        print_project_header(&project, &palette);

        match project.status() {
            Ok(Some(status)) => {
                for b in status {
                    println!("  {}", b.describe_full(&palette));
                }
            },
            Ok(None) => {
                println!("{}", palette.missing.paint(format!("    {: <25 } {}", "", "Missing repository")));
            },
            Err(err) => {
                eprintln!("{}", palette.error.paint(format!("Failed to compute status: {}", err)));
                exit_code = exit_codes::STATUS_PROJECT_FAILED;
            }
        }
    }

    Ok(exit_code)
}
