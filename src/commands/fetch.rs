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

enum FetchProjectResult {
  Ok(HashSet<String>),
  Git2Error(git2::Error),
  RepositoryMissing,
}

struct FetchResult<'ws> {
  pub results: HashMap<&'ws Project, FetchProjectResult>,
}
impl<'ws> FetchResult<'ws> {
  fn new() -> FetchResult<'ws> {
    FetchResult { results: HashMap::new() }
  }
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

fn do_fetch(project: &Project, repo: git2::Repository, palette: &Palette) -> Result<HashSet<String>, git2::Error> {
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

  let updated_names: HashSet<String> = all_heads_after
    .into_iter()
    .filter(|(k, v_after)|
            all_heads_before.get(k).map(|v_before| v_before != v_after).unwrap_or(false)
    )
    .map(|(k, _)| k)
    .collect();

  Ok(updated_names)
}

fn print_output(workspace: &Workspace, palette: &Palette, result: FetchResult) {
  for project in &workspace.projects {
    println!("{}", format_project_header(&project, &palette));

    match result.results.get(&project).unwrap() {
      FetchProjectResult::Ok(updated) => {
        if updated.is_empty() {
          println!("{}", palette.clean.paint(format_message_line("Clean")));
        } else {
          println!("{}", palette.cloning.paint(format_message_line("Fetched from origin")));
        }
      },
      FetchProjectResult::Git2Error(err) => {
        eprintln!("Failed to open repository: {}", err);
        println!("{}", palette.error.paint(format_message_line("Error")));
      },
      FetchProjectResult::RepositoryMissing => {
        println!("{}", palette.missing.paint(format_message_line("Missing repository")));
      }
    }
  }
}

impl Command for Fetch {
  fn run(&self, working_dir: &Path, workspace: &Workspace, palette: &Palette) -> Result<i32, ::git2::Error> {
    let mut result = FetchResult::new();

    result.results = workspace.projects.iter()
      .map(|project|
           (project, match project.open_repository(working_dir) {
             Some(Ok(repo)) =>
               do_fetch(&project, repo, palette)
                 .map(FetchProjectResult::Ok)
                 .unwrap_or_else(FetchProjectResult::Git2Error),
             Some(Err(err)) =>
               FetchProjectResult::Git2Error(err),
             None =>
               FetchProjectResult::RepositoryMissing,
           })
      )
      .collect();

    print_output(workspace, palette, result);

    Ok(
      exit_codes::OK
    )
  }
}
