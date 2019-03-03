use std::collections::BTreeMap;
use std::path::Path;

use commands::error::Error;
use super::Remote;


#[derive(Debug)]
#[derive(Eq)]
#[derive(Hash)]
#[derive(Ord)]
#[derive(PartialEq)]
#[derive(PartialOrd)]
pub struct Project {
    pub path: String,
    pub main_remote: Remote,
    pub extra_remotes: Vec<Remote>,
}

impl Project {

    pub fn open_repository<P: AsRef<Path>>(&self, working_dir: P) -> Result<git2::Repository, Error> {
        let repo_dir = working_dir.as_ref().join(&self.path);
        if repo_dir.exists() {
            git2::Repository::open(repo_dir)
                .map_err(Error::from)
        } else {
            Err(Error::RepositoryMissing)
        }
    }

    pub fn remotes(&self) -> Vec<&Remote> {
        let mut extras: Vec<&Remote> = self.extra_remotes.iter().collect();
        let mut result = vec![&self.main_remote];
        result.append(&mut extras);
        result
    }

    fn local_branches_internal<'repo>(&self, repo: &'repo git2::Repository) -> Result<Vec<(git2::Branch<'repo>, Option<git2::Branch<'repo>>)>, Error> {
        Ok(
            repo
                .branches(Some(git2::BranchType::Local))?
                .flatten()
                .map(|(branch, _)| {
                    let upstream = branch.upstream().ok();
                    (branch, upstream)
                })
                .collect()
        )
    }

    pub fn current_upstream_heads<'repo>(&self, repo: &'repo git2::Repository) -> Result<BTreeMap<git2::Branch<'repo>, git2::Oid>, Error> {
        self.local_branches_internal(repo)
            .map(|branches|
                     branches
                     .into_iter()
                     .flat_map(|(branch, gupstream)|
                                         gupstream.map(|gupstream| (branch, gupstream))
                     )
                     .map(|(branch, gupstream)|
                                (branch, gupstream.get().peel_to_commit().unwrap().id()),
                     )
                     .collect()
            )
    }
}

#[cfg(test)]
mod tests {
    use super::Project;
    use super::Remote;

    #[test]
    fn remotes_returns_main_remote_and_then_extras() {
        assert_eq!(
            Project {
                path: "foo".to_string(),
                main_remote: Remote {
                    name: "origin".to_string(),
                    url: "git@github.com:foo/boo.git".to_string(),
                },
                extra_remotes: vec![
                    Remote {
                        name: "myone".to_string(),
                        url: "http://coool".to_string(),
                    },
                    Remote {
                        name: "upstream".to_string(),
                        url: "testurl".to_string(),
                    },
                ],
            }.remotes(),
            vec![
                Remote {
                    name: "origin".to_string(),
                    url: "git@github.com:foo/boo.git".to_string(),
                },
                Remote {
                    name: "myone".to_string(),
                    url: "http://coool".to_string(),
                },
                Remote {
                    name: "upstream".to_string(),
                    url: "testurl".to_string(),
                },
            ].iter().collect::<Vec<&Remote>>()
        );
    }
}
