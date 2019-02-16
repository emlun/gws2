use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::path::Path;

use color::palette::Palette;
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

struct FetchedProject<'repo> {
  pub repo: &'repo ::git2::Repository,
  pub local_branches: BTreeSet<git2::Branch<'repo>>,
  pub updated_branches: BTreeSet<git2::Branch<'repo>>,
}

fn do_fetch_remote<'repo>(
  project: &Project,
  repo: &'repo git2::Repository,
  remote: &mut git2::Remote
) -> Result<BTreeSet<git2::Branch<'repo>>, Error> {
  let heads_before: BTreeMap<git2::Branch, git2::Oid> =
    project.current_upstream_heads(repo)?;

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

  let heads_after: BTreeMap<git2::Branch, git2::Oid> =
    project.current_upstream_heads(repo)?;

  let updated_branches: BTreeSet<git2::Branch> = heads_after
    .into_iter()
    .filter(|(k, v_after)|
            heads_before.get(k)
            .map(|v_before| v_before != v_after)
            .unwrap_or(false)
    )
    .map(|(k, _)| k)
    .collect();

  Ok(updated_branches)
}

fn do_fetch<'repo>(
  project: &Project,
  repo: &'repo git2::Repository
) -> FetchedProject<'repo> {
  FetchedProject {
    repo: repo,
    local_branches: project.local_branches(&repo).unwrap_or_else(|_| BTreeSet::new()),
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

fn print_output(
  workspace: &Workspace,
  palette: &Palette,
  result: BTreeMap<&Project, Result<FetchedProject, &Error>>
) {
  for project in &workspace.projects {
    println!("{}", format_project_header(&project, &palette));

    match result.get(&project).unwrap() {
      Ok(project_result) => {
        for branch in &project_result.local_branches {
          let msg =
            if project_result.updated_branches.contains(branch) {
              palette.cloning.paint("New upstream commits")
            } else {
              palette.clean.paint("No update")
            };
          println!("{}", format_branch_line(
            palette,
            false,
            match branch.name() {
              Ok(Some(name)) => name,
              _ => "<Unprintable name>",
            },
            &msg
          ));
        }
      },
      Err(Error::Git2Error(err)) => {
        eprintln!("Failed to open repository: {}", err);
        println!("{}", palette.error.paint(format_message_line("Error")));
      },
      Err(Error::RepositoryMissing) => {
        println!("{}", palette.missing.paint(format_message_line("Missing repository")));
      },
      Err(_) => unreachable!(),
    }
  }
}

impl Command for Fetch {
  fn run<'ws>(&self, working_dir: &Path, workspace: &'ws Workspace, palette: &Palette) -> Result<i32, Error> {
    let repos: BTreeMap<&Project, Result<git2::Repository, Error>> = workspace.projects.iter()
      .map(|project|
           (
             project,
             project.open_repository(working_dir)
           )
      )
      .collect();

    let results = repos.iter()
      .map(|(project, repo)|
           (
             *project,
             repo.as_ref().map(|repo| do_fetch(&project, &repo))
           )
      )
      .collect();

    print_output(workspace, palette, results);

    Ok(
      exit_codes::OK
    )
  }
}
