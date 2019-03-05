use std::path::Path;

use super::common::exit_codes;
use super::common::print_status;
use super::common::RepositoryCommand;
use super::error::Error;
use color::palette::Palette;
use config::data::Workspace;
use data::status::project_status;
use data::status::WorkspaceStatus;

pub struct Status {
    pub only_changes: bool,
}

impl Status {
    pub fn make_report<'ws>(
        &self,
        working_dir: &Path,
        workspace: &'ws Workspace,
    ) -> WorkspaceStatus<'ws> {
        workspace
            .projects
            .iter()
            .map(|project| (project, project_status(project, working_dir)))
            .filter(|(_, status_result)| {
                self.only_changes == false
                    || status_result
                        .as_ref()
                        .map(|status| status.iter().any(|b| !b.is_clean()))
                        .unwrap_or(false)
            })
            .collect()
    }
}

impl RepositoryCommand for Status {
    fn run(
        &self,
        working_dir: &Path,
        workspace: &Workspace,
        palette: &Palette,
    ) -> Result<i32, Error> {
        let report = self.make_report(working_dir, workspace);

        for (project, project_result) in &report {
            print_status(project, project_result, palette);
        }

        let exit_code = report
            .values()
            .map(|project_result| match project_result {
                Ok(_) => exit_codes::OK,
                Err(Error::RepositoryMissing) => exit_codes::OK,
                Err(_) => exit_codes::STATUS_PROJECT_FAILED,
            })
            .fold(exit_codes::OK, |exit_code, next_code| {
                if next_code != exit_codes::OK {
                    next_code
                } else {
                    exit_code
                }
            });
        Ok(exit_code)
    }
}
