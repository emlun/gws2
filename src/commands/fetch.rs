use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;


use color::palette::Palette;
use config::data::Project;
use config::data::Workspace;
use super::common::Command;
use super::common::exit_codes;
use super::common::format_message_line;
use super::common::format_project_header;


pub struct Fetch {
  pub projects: HashSet<String>,
}

fn get_current_heads(repo: &git2::Repository) -> Result<HashMap<String, git2::Oid>, git2::Error> {
  Ok(
    repo
      .branches(Some(git2::BranchType::Remote))?
      .flatten()
      .map(|(branch, _)|
          (
            String::from(branch.name().unwrap().unwrap()),
            branch.get().peel_to_commit().unwrap().id(),
          )
      )
      .collect()
  )
}

fn do_fetch(project: &Project, repo: git2::Repository, palette: &Palette) -> Result<(), git2::Error> {
  let mut all_heads_before: HashMap<String, git2::Oid> = HashMap::new();
  let mut all_heads_after: HashMap<String, git2::Oid> = HashMap::new();

  for remote_config in project.remotes() {
    match repo.find_remote(&remote_config.name) {
      Ok(mut remote) => {
        all_heads_before.extend(get_current_heads(&repo)?);

        let refspec_strings: Vec<String> = remote
          .refspecs()
          .flat_map(|rs| rs.str().map(String::from))
          .collect();

        remote.fetch(
          &refspec_strings
            .iter()
            .map(|s| &**s)
            .collect::<Vec<&str>>()
          ,
          None,
          None
        )?;

        all_heads_after.extend(get_current_heads(&repo)?);
      },
      Err(_) => {
        eprintln!("Remote {} not found in repository.", remote_config.name);
      }
    }
  }

  if all_heads_before != all_heads_after {
    println!("{}", palette.cloning.paint(format_message_line("Fetched from origin")));
  } else {
    println!("{}", palette.clean.paint(format_message_line("Clean")));
  }

  Ok(())
}

impl Command for Fetch {
  fn run(&self, working_dir: &Path, workspace: &Workspace, palette: &Palette) -> Result<i32, ::git2::Error> {
    for project in workspace.projects.iter()
      .filter(|proj|
              self.projects.contains(&proj.path)
      )
    {
      println!("{}", format_project_header(&project, &palette));

      match project.open_repository(working_dir) {
        Some(Ok(repo)) => {
          do_fetch(&project, repo, palette)?;
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
