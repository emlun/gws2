use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::path::Path;

use color::palette::Palette;
use config::data::Project;
use config::data::Workspace;
use data::status::ProjectStatusMethods;
use data::status::RepositoryStatus;
use data::status::WorkspaceStatus;
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
  pub updated_branch_names: BTreeSet<String>,
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
  repo: & git2::Repository
) -> FetchedProject {
    FetchedProject {
      updated_branch_names: project.remotes()
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
        .flat_map(|branch|
                  branch.name()
                  .ok()
                  .and_then(|n| n)
                  .map(String::from)
        )
        .collect()
    }
}

fn make_project_status_report<'proj, 'repo, 'result>(
  working_dir: &Path,
  project: &Project,
  result: FetchedProject,
) -> Result<RepositoryStatus, Error> {
  let updated = result.updated_branch_names;

  let status = project.status(working_dir)?;

  Ok(
    status
      .into_iter()
      .map(|mut branch_status| {
        branch_status.upstream_fetched =
          updated.iter()
          .any(|upd_name| &branch_status.name == upd_name);
        branch_status
      })
      .collect()
  )
}

fn print_output(
  project: &Project,
  project_status: &Result<RepositoryStatus, Error>,
  palette: &Palette,
) {
  println!("{}", format_project_header(project, palette));

  match project_status {
    Ok(project_status) => {
      for branch_status in project_status {
        let msg =
          if branch_status.upstream_fetched {
            palette.cloning.paint("New upstream commits")
          } else {
            palette.clean.paint("No update")
          };

        println!("{}", format_branch_line(
          palette,
          false,
          &branch_status.name,
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
    Err(err) => {
      eprintln!("Failed to list branches: {}", err);
      println!("{}", palette.error.paint("Failed to list branches"));
    },
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

    let status_report: WorkspaceStatus =
      repos
      .into_iter()
      .map(|(project, repo_result)|
           (
             project,
             repo_result.and_then(|repo| {
               let fetch_result = do_fetch(project, &repo);
               make_project_status_report(working_dir, project, fetch_result)
             }),
           )
      )
      .collect();

    for (project, report_result) in status_report {
      print_output(project, &report_result, palette)
    }

    Ok(
      exit_codes::OK
    )
  }
}
