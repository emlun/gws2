use super::Project;


#[derive(Debug)]
#[derive(PartialEq)]
pub struct Workspace {
    pub projects: Vec<Project>,
}
