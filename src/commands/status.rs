use std::collections::HashSet;

use super::common::RepositoryCommand;
use super::error::Error;
use config::data::Project;
use data::status::project_status;
use data::status::RepositoryStatus;

pub struct Status {
    pub only_changes: bool,
    pub projects: HashSet<String>,
}

impl RepositoryCommand for Status {
    fn only_changes(&self) -> bool {
        self.only_changes
    }

    fn project_args(&self) -> &HashSet<String> {
        &self.projects
    }

    fn run_project(
        &self,
        project: &Project,
        repository: &git2::Repository,
    ) -> Result<RepositoryStatus, Error> {
        project_status(project, repository)
    }
}
