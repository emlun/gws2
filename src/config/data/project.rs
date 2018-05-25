use std::path:: Path;

use git2::Repository;

use super::Remote;


#[derive(Debug)]
#[derive(PartialEq)]
pub struct Project {
    pub path: String,
    pub main_remote: Remote,
    pub extra_remotes: Vec<Remote>,
}

impl Project {

    pub fn open_repository(&self) -> Option<Result<Repository, ::git2::Error>> {
        if Path::new(&self.path).exists() {
            Some(Repository::open(&self.path))
        } else {
            None
        }
    }

}
