use std::path::Path;

use ansi_term::ANSIString;
use git2::Repository;

use color::palette::Palette;
use config::read::read_workspace_file;
use data::status::BranchStatus;
use data::status::DirtyState;
use data::status::RepositoryMethods;


trait BranchStatusPrinting {
    fn describe_sync_status(&self, palette: &Palette) -> ANSIString;
    fn describe_status(&self, palette: &Palette) -> ANSIString;
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
}

fn ellipsisize(s: &str, length: usize) -> String {
    if s.len() >= length {
        format!("{}…", &s[0..(length - 1)]).to_string()
    } else {
        s.to_string()
    }
}

pub fn run() -> Result<(), ::git2::Error> {

    let ws_file_path = Path::new(".projects.gws");
    let ws = read_workspace_file(ws_file_path).unwrap();

    let palette = Palette::default();

    for project in ws.projects {
        println!("{}:", palette.repo.paint(project.path.clone()));

        match Repository::open(
            ws_file_path
                .parent()
                .unwrap()
                .join(&project.path)
                .as_path()
            )
        {
            Ok(repo) => {
                for b in try!(repo.project_status(&project)) {
                    println!(
                        "  {} {} {}",
                        if b.is_head { "*" } else { " " },
                        palette.branch.paint(format!("{: <25}", format!("{} :", ellipsisize(&b.name, 23)))),
                        b.describe_status(&palette)
                    );
                }
            },
            Err(err) => {
                match err.code() {
                    ::git2::ErrorCode::NotFound => println!("{}", palette.missing.paint(format!("    {: <25 } {}", "", "Missing repository"))),
                    _ => println!("{}", palette.error.paint(format!("Failed to open repo: {}", err))),
                }
            }
        }

    }

    Ok(())
}
