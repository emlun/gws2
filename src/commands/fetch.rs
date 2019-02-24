use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::path::Path;

use color::palette::Palette;
use config::data::Project;
use config::data::Workspace;
use data::status::RepositoryStatus;
use data::status::WorkspaceStatus;
use super::common::Command;
use super::common::exit_codes;
use super::error::Error;
use super::status::Status;


pub struct Fetch {
  pub status_command: Status,
  pub projects: HashSet<String>,
}

impl Fetch {
  pub fn run_command<'ws>(
    &self,
    working_dir: &Path,
    workspace: &'ws Workspace,
  ) -> WorkspaceStatus<'ws> {
    self
    .status_command.make_report(working_dir, workspace)
    .into_iter()
    .map(|(project, project_status_result)|
         (
           project,
           project
             .open_repository(working_dir)
             .and_then(|repo| project_status_result.map(|pr| (repo, pr)))
             .and_then(|(repo, project_status)| {
               let fetch_result = do_fetch(project, &repo);
               augment_project_status_report(project_status, fetch_result)
             }),
         )
    )
    .collect()
  }
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

fn augment_project_status_report<'proj, 'repo, 'result>(
  status: RepositoryStatus,
  result: FetchedProject,
) -> Result<RepositoryStatus, Error> {
  let updated = result.updated_branch_names;
  Ok(
    status
      .into_iter()
      .map(|mut branch_status| {
        branch_status.upstream_fetched =
          updated
          .iter()
          .any(|upd_name| &branch_status.name == upd_name);
        branch_status
      })
      .collect()
  )
}

impl Command for Fetch {
  fn run<'ws>(
    &self,
    working_dir: &Path,
    workspace: &'ws Workspace,
    palette: &Palette,
  ) -> Result<i32, Error> {
    let status_report = self.run_command(working_dir, workspace);

    for (project, report_result) in status_report {
      self.status_command.print_output(project, &report_result, palette)
    }

    Ok(
      exit_codes::OK
    )
  }
}
