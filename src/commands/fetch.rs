use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::path::Path;

use color::palette::Palette;
use config::data::Branch;
use config::data::Project;
use config::data::Workspace;
use super::common::Command;
use super::common::exit_codes;
use super::common::format_branch_line;
use super::common::format_message_line;
use super::common::format_project_header;
use super::error::Error;


pub struct Fetch {
  pub projects: HashSet<String>,
}

struct FetchedProject {
  pub branches: BTreeSet<String>,
  pub updated_branches: BTreeSet<String>,
}

fn do_fetch_remote(project: &Project, repo: &git2::Repository, remote: &mut git2::Remote) -> Result<BTreeSet<String>, Error> {
  let heads_before: BTreeMap<Branch, git2::Oid> = project.current_upstream_heads(repo)?;

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

  let heads_after: BTreeMap<Branch, git2::Oid> = project.current_upstream_heads(repo)?;

  let updated_branches: BTreeSet<String> = heads_after
    .into_iter()
    .filter(|(k, v_after)|
            heads_before.get(k).map(|v_before| v_before != v_after).unwrap_or(false)
    )
    .map(|(k, _)| k)
    .flat_map(|b| b.name)
    .collect();

  Ok(updated_branches)
}

fn do_fetch<'proj>(project: &'proj Project, repo: &git2::Repository) -> FetchedProject {
  FetchedProject {
    branches: project.local_branches(repo).unwrap().into_iter().flat_map(|b| b.name).collect(),
    updated_branches: project.remotes()
      .into_iter()
      .flat_map(|remote_config|
           match repo.find_remote(&remote_config.name) {
             Ok(mut remote) =>
               do_fetch_remote(project, &repo, &mut remote)
               .unwrap_or(BTreeSet::new())
               .into_iter()
               ,
             Err(_) =>
               BTreeSet::new().into_iter(),
           }
      )
      .collect()
  }
}

fn print_output(workspace: &Workspace, palette: &Palette, result: BTreeMap<&Project, Result<FetchedProject, Error>>) {
  for project in &workspace.projects {
    println!("{}", format_project_header(&project, &palette));

    match result.get(&project).unwrap() {
      Ok(project_result) => {
        for branch in &project_result.branches {
          let msg =
            if project_result.updated_branches.contains(branch) {
              palette.cloning.paint("New upstream commits")
            } else {
              palette.clean.paint("No update")
            };
          println!("{}", format_branch_line(palette, false, branch, &msg));
        }
      },
      Err(Error::Git2Error(err)) => {
        eprintln!("Failed to open repository: {}", err);
        println!("{}", palette.error.paint(format_message_line("Error")));
      },
      Err(Error::RepositoryMissing) => {
        println!("{}", palette.missing.paint(format_message_line("Missing repository")));
      }
    }
  }
}

impl Command for Fetch {
  fn run(&self, working_dir: &Path, workspace: &Workspace, palette: &Palette) -> Result<i32, Error> {
    let results = workspace.projects.iter()
      .map(|project|
           (project, match project.open_repository(working_dir) {
             Some(Ok(repo)) =>
               Ok(do_fetch(&project, &repo)),
             Some(Err(err)) =>
               Err(Error::Git2Error(err)),
             None =>
               Err(Error::RepositoryMissing),
           })
      )
      .collect();

    print_output(workspace, palette, results);

    Ok(
      exit_codes::OK
    )
  }
}
