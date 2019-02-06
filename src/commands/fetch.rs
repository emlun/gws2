use std::collections::HashSet;
use std::path::Path;

use git2::Repository;

use color::palette::Palette;
use config::data::Workspace;
use data::status::ProjectStatusMethods;
use super::common::Command;
use super::common::exit_codes;
use super::common::format_message_line;
use super::common::format_project_header;


pub struct Fetch {
  pub projects: HashSet<String>,
}

impl Command for Fetch {
  fn run(&self, working_dir: &Path, workspace: Workspace, palette: &Palette) -> Result<i32, ::git2::Error> {

    for project in workspace.projects.into_iter()
      .filter(|proj|
        self.projects.contains(&proj.path)
      )
    {
      println!("{}", format_project_header(&project, &palette));

      match project.open_repository(working_dir) {
        Some(Ok(repo)) => {
          println!("{}", palette.cloning.paint(format_message_line("Fetching…")));

          for remote_config in project.remotes() {
            match repo.find_remote(&remote_config.name) {
              Ok(mut remote) => {
                let refspecs: Vec<String> = remote
                  .refspecs()
                  .flat_map(|refspec| refspec.str().map(String::from))
                  .collect();
                let rs2: Vec<&str> = refspecs
                  .iter()
                  .map(String::as_str)
                  .collect();

                remote.fetch(
                  &rs2,
                  None,
                  None
                )?;
              },
              Err(err) => {
                eprintln!("Remote {} not found in repository.", remote_config.name);
              }
            }
          }

          println!("{}", palette.clean.paint(format_message_line("Fetched.")));
        },
        Some(Err(err)) => {
          eprintln!("Failed to open repository: {}", err);
          println!("{}", palette.error.paint(format_message_line("Error")));
        },
        None => {
          println!("{}", palette.missing.paint(format_message_line("Missing repository")));
        }
      }
    }

    Ok(
      exit_codes::OK
    )
  }
}
