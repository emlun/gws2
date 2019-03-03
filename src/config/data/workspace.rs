use std::collections::BTreeSet;

use super::Project;

#[derive(Debug, PartialEq)]
pub struct Workspace {
    pub projects: BTreeSet<Project>,
}

impl<I: IntoIterator<Item = Project>> From<I> for Workspace {
    fn from(projects: I) -> Self {
        Workspace {
            projects: projects.into_iter().collect(),
        }
    }
}
