use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;

use super::common::RepositoryCommand;
use super::error::Error;
use super::status::Status;
use crate::config::data::Project;
use crate::data::status::RepositoryStatus;

pub struct Fetch {
    pub status_command: Status,
}

struct FetchedProject {
    pub updated_branch_names: BTreeSet<String>,
}

fn do_fetch_remote<'repo>(
    project: &Project,
    repo: &'repo git2::Repository,
    remote: &mut git2::Remote,
) -> Result<BTreeSet<git2::Branch<'repo>>, Error> {
    let heads_before: BTreeMap<git2::Branch, git2::Oid> = project.current_upstream_heads(repo)?;

    let refspec_strings: Vec<String> = remote
        .refspecs()
        .flat_map(|rs| rs.str().map(String::from))
        .collect();

    remote.fetch(
        &refspec_strings.iter().map(|s| &**s).collect::<Vec<&str>>(),
        None,
        None,
    )?;

    let heads_after: BTreeMap<git2::Branch, git2::Oid> = project.current_upstream_heads(repo)?;

    let updated_branches: BTreeSet<git2::Branch> = heads_after
        .into_iter()
        .filter(|(k, v_after)| {
            heads_before
                .get(k)
                .map(|v_before| v_before != v_after)
                .unwrap_or(false)
        })
        .map(|(k, _)| k)
        .collect();

    Ok(updated_branches)
}

fn do_fetch<'repo>(project: &Project, repo: &git2::Repository) -> FetchedProject {
    FetchedProject {
        updated_branch_names: project
            .remotes()
            .into_iter()
            .flat_map(
                |remote_config| match repo.find_remote(&remote_config.name) {
                    Ok(mut remote) => do_fetch_remote(project, &repo, &mut remote)
                        .unwrap_or(BTreeSet::new())
                        .into_iter(),
                    Err(_) => BTreeSet::new().into_iter(),
                },
            )
            .flat_map(|branch| branch.name().ok().and_then(|n| n).map(String::from))
            .collect(),
    }
}

fn augment_project_status_report<'proj, 'repo, 'result>(
    status: RepositoryStatus,
    result: FetchedProject,
) -> Result<RepositoryStatus, Error> {
    let updated = result.updated_branch_names;
    Ok(status
        .into_iter()
        .map(|mut branch_status| {
            branch_status.upstream_fetched = updated
                .iter()
                .any(|upd_name| &branch_status.name == upd_name);
            branch_status
        })
        .collect())
}

impl RepositoryCommand for Fetch {
    fn only_changes(&self) -> bool {
        self.status_command.only_changes()
    }

    fn project_args(&self) -> &HashSet<String> {
        self.status_command.project_args()
    }

    fn run_project(
        &self,
        project: &Project,
        repository: &git2::Repository,
    ) -> Result<RepositoryStatus, Error> {
        self.status_command
            .run_project(project, repository)
            .and_then(|project_status| {
                let fetch_result = do_fetch(project, repository);
                augment_project_status_report(project_status, fetch_result)
            })
    }
}
