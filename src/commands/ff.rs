use std::collections::HashSet;

use super::common::RepositoryCommand;
use super::error::Error;
use super::fetch::Fetch;
use crate::config::data::Project;
use crate::crate_info::crate_name;
use crate::data::status::BranchMethods;
use crate::data::status::DirtyState;
use crate::data::status::RepositoryStatus;

pub struct FastForward {
    pub fetch_command: Fetch,
}

fn do_ff(
    repo: &git2::Repository,
    status_report: RepositoryStatus,
) -> Result<RepositoryStatus, Error> {
    status_report
        .into_iter()
        .map(|mut branch_status| {
            if branch_status.dirty == DirtyState::Clean && branch_status.upstream_name.is_some() {
                let branch = repo.find_branch(&branch_status.name, git2::BranchType::Local)?;
                let branch_id = branch.get().peel_to_commit()?.id();
                let upstream = branch.upstream()?;
                let upstream_id: git2::Oid = upstream.get().peel_to_commit()?.id();

                if branch_id != upstream_id {
                    let can_fast_forward = repo.graph_descendant_of(upstream_id, branch_id)?;

                    if can_fast_forward {
                        let reflog_msg = format!(
                            "{prog_name}: Fast-forward {branch_name} to upstream {upstream_name}",
                            prog_name = crate_name(),
                            branch_name = branch.branch_name()?,
                            upstream_name = upstream.branch_name()?
                        );

                        branch
                            .into_reference()
                            .set_target(upstream_id, &reflog_msg)?;

                        if branch_status.is_head {
                            repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))?;
                        }

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
