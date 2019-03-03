use std::collections::HashSet;
use std::path::Path;

use super::common::exit_codes;
use super::common::Command;
use super::error::Error;
use super::fetch::Fetch;
use color::palette::Palette;
use config::data::Workspace;
use crate_info::crate_name;
use data::status::RepositoryStatus;
use data::status::WorkspaceStatus;
use util::git2::WithAncestors;

pub struct FastForward {
    pub fetch_command: Fetch,
}

impl FastForward {
    fn projects(&self) -> &HashSet<String> {
        &self.fetch_command.projects
    }

    pub fn run_command<'ws>(
        &self,
        working_dir: &Path,
        workspace: &'ws Workspace,
    ) -> WorkspaceStatus<'ws> {
        self.fetch_command
            .run_command(working_dir, workspace)
            .into_iter()
            .map(|(project, project_ff_result)| {
                (
                    project,
                    if self.projects().is_empty() || self.projects().contains(&project.path) {
                        project
                            .open_repository(working_dir)
                            .and_then(|repo| project_ff_result.map(|pr| (repo, pr)))
                            .and_then(|(repo, project_status)| do_ff(&repo, project_status))
                    } else {
                        project_ff_result
                    },
                )
            })
            .collect()
    }
}

fn do_ff<'repo>(
    repo: &git2::Repository,
    status_report: RepositoryStatus,
) -> Result<RepositoryStatus, Error> {
    status_report
        .into_iter()
        .map(|mut branch_status| {
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
            Ok(branch_status)
        })
        .collect()
}

impl Command for FastForward {
    fn run<'ws>(
        &self,
        working_dir: &Path,
        workspace: &'ws Workspace,
        palette: &Palette,
    ) -> Result<i32, Error> {
        let status_report = self.run_command(working_dir, workspace);

        for (project, report_result) in status_report {
            self.fetch_command
                .status_command
                .print_output(project, &report_result, palette)
        }

        Ok(exit_codes::OK)
    }
}
