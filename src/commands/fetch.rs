use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::path::Path;

use ansi_term::ANSIString;

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

struct ProjectStatusReport<'repo, 'result> {
  branch_statuses: BTreeMap<git2::Branch<'repo>, ANSIString<'result>>,
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
) -> Result<FetchedProject<'repo>, Error> {
  project
    .local_branches(&repo)
    .map(|local_branches|
         FetchedProject {
           repo: repo,
           local_branches: local_branches,
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
    )
}

fn make_project_status_report<'proj, 'repo, 'result>(
  result: FetchedProject<'repo>,
  palette: &Palette,
) -> ProjectStatusReport<'repo, 'result> {
  let branches = result.local_branches;
  let updated = result.updated_branches;
  ProjectStatusReport {
    branch_statuses: branches
      .into_iter()
      .map(|branch| {
        let msg =
          if updated.contains(&branch) {
            palette.cloning.paint("New upstream commits")
          } else {
            palette.clean.paint("No update")
          };
        (branch, msg)
      })
      .collect()
  }
}

fn print_output(
  project: &Project,
  report: &Result<ProjectStatusReport, Error>,
  palette: &Palette,
) {
  println!("{}", format_project_header(project, palette));

  match report {
    Ok(project_result) => {
      for (branch, branch_status) in &project_result.branch_statuses {
        println!("{}", format_branch_line(
          palette,
          false,
          match branch.name() {
            Ok(Some(name)) => name,
            _ => "<Unprintable name>",
          },
          &branch_status
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

    for (project, repo_result) in repos {
      match repo_result {
        Ok(repo) => {
          let fetch_result = do_fetch(project, &repo);
          let report = fetch_result
            .map(|result|
                 make_project_status_report(result, palette)
            );
          print_output(project, &report, palette);
        },
        e @ Err(_) => {
          print_output(project, &e.map(|_| unreachable!()), palette);
        },
      }
    }

    Ok(
      exit_codes::OK
    )
  }
}
