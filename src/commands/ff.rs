use std::collections::HashSet;

use super::common::RepositoryCommand;
use super::error::Error;
use super::fetch::Fetch;
use crate::config::data::Project;
use crate::crate_info::crate_name;
use crate::data::status::RepositoryStatus;
use crate::util::git2::WithAncestors;

pub struct FastForward {
    pub fetch_command: Fetch,
}

fn do_ff<'repo>(
    repo: &git2::Repository,
    status_report: RepositoryStatus,
) -> Result<RepositoryStatus, Error> {
    status_report
        .into_iter()
        .map(|mut branch_status| {
            if branch_status.upstream_name.is_some() {
                let branch = repo.find_branch(&branch_status.name, git2::BranchType::Local)?;
                let upstream = branch.upstream()?;
                let upstream_id: git2::Oid = upstream.get().peel_to_commit()?.id();

                if branch.get().peel_to_commit()?.id() != upstream_id {
                    let can_fast_forward = upstream
                        .get()
                        .peel_to_commit()?
                        .is_descendant_of(&branch.get().peel_to_commit()?);

                    if can_fast_forward {
                        let reflog_msg = format!(
                            "{prog_name}: Fast-forward {branch_name} to upstream {upstream_name}",
                            prog_name = crate_name(),
                            branch_name = branch.name()?.unwrap(),
                            upstream_name = upstream.name()?.unwrap()
                        );

                        branch
                            .into_reference()
                            .set_target(upstream_id, &reflog_msg)?;

                        branch_status.fast_forwarded = true;
                    }
                }
            }
            Ok(branch_status)
        })
        .collect()
}

impl RepositoryCommand for FastForward {
    fn only_changes(&self) -> bool {
        self.fetch_command.only_changes()
    }

    fn project_args(&self) -> &HashSet<String> {
        self.fetch_command.project_args()
    }

    fn run_project(
        &self,
        project: &Project,
        repository: &git2::Repository,
    ) -> Result<RepositoryStatus, Error> {
        self.fetch_command
            .run_project(project, repository)
            .and_then(|project_status| do_ff(repository, project_status))
    }
}
